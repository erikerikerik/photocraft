//! The `method: "ai"` paths of `select.subject` and `layer.removeBackground` (SPEC amendment
//! 18): submit the provider task `subject_mask` and turn the mask it answers into a selection or a
//! layer mask. The two upstream command files branch on `method` into here, so a weekly sync stays
//! small. The image goes out downscaled by area average (a document is never refused for its size)
//! and the mask comes back at that size, stretched bilinearly to the document area; the classical
//! commands' own apply code then stores it, so both methods leave the same kind of selection or
//! layer mask and undo in one step. No network code lives here.

use std::sync::Arc;
use std::time::Duration;

use photocraft_algo::matting;
use photocraft_algo::segment::Sampler;
use photocraft_algo::selection::Region;
use photocraft_color::PixelFormat;
use photocraft_doc::{Document, Layer, LayerId, LayerMask};
use photocraft_geom::{Rect, Size};
use serde_json::{Value, json};

use crate::ai::{self, AiError, AiOutput, AiProvider, AiRequest, AiTask};
use crate::cutout_cmds;
use crate::jobs::{self, JobCtx};
use crate::smartselect_cmds::{self, with_doc_sampler};
use crate::{EngineError, Result, Session};

/// How long to wait between provider polls on a job worker, as Generative Fill does.
const POLL_EVERY: Duration = Duration::from_millis(100);

/// The sentence for a manager that is running but has no subject model. Amendment 18 has PhotoCraft
/// check `ready` in `/v1/tasks` itself, so this arrives before a job is ever posted.
const NOT_INSTALLED: &str = "The Select Subject model isn't installed. Open Craft AI Manager to add it.";

/// Source rows read per strip while downscaling: enough to amortise a compositor call, few enough
/// that a full-resolution canvas is never held in memory at once.
const STRIP_ROWS: usize = 256;

/// The most source pixel data a downscale keeps in flight at once. The strips go to the rayon
/// workers a batch at a time, so the parallelism cannot materialise the whole canvas - which is
/// the whole point of reading it in strips.
const MAX_RESIDENT_BYTES: usize = 32 * 1024 * 1024;

/// Which method a command was asked for (amendment 18).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Classical,
    Ai,
}

/// Read the `method` param. Missing means `classical`, so every caller today is unchanged.
pub(crate) fn method(p: &Value, cmd: &str) -> Result<Method> {
    let bad = |msg: String| EngineError::BadParams { cmd: cmd.to_string(), msg };
    match p.get("method") {
        None | Some(Value::Null) => Ok(Method::Classical),
        Some(Value::String(name)) => match name.as_str() {
            "classical" => Ok(Method::Classical),
            "ai" => Ok(Method::Ai),
            other => Err(bad(format!("`method` must be \"classical\" or \"ai\", not {other:?}"))),
        },
        Some(_) => Err(bad("`method` must be \"classical\" or \"ai\"".to_string())),
    }
}

/// Where the work ran (B08 plus amendment 18): the names Generative Fill returns, plus the new
/// provenance fields. A field the provider did not report is left out, because missing means
/// unknown rather than a problem.
fn provenance(result: &ai::AiResult) -> Value {
    let p = &result.provenance;
    let text = |v: &Option<String>| v.clone().map(Value::String);
    let mut out = serde_json::Map::new();
    for (name, value) in [
        ("model", Some(Value::String(result.model.clone()))),
        ("device", Some(Value::String(result.device.clone()))),
        ("provider", Some(Value::String(result.provider.clone()))),
        ("elapsedMs", Some(Value::from(result.elapsed_ms))),
        ("modelVersion", text(&p.model_version)),
        ("modelSha256", text(&p.model_sha256)),
        ("deviceRequested", text(&p.device_requested)),
        ("deviceName", text(&p.device_name)),
        ("fallbackReason", text(&p.fallback_reason)),
        ("diagnostics", text(&p.diagnostics)),
        ("loadMs", p.load_ms.map(Value::from)),
        ("runMs", p.run_ms.map(Value::from)),
    ] {
        if let Some(value) = value {
            out.insert(name.to_string(), value);
        }
    }
    Value::Object(out)
}

/// The size to send for a document area under `max_pixels`: the largest size that fits the limit at
/// the same aspect ratio, within a pixel. A document bigger than the limit is scaled down, never
/// refused (amendment 18), and one that already fits is sent as it is.
fn sent_size(area: Size, max_pixels: u64) -> Size {
    let (w, h) = (f64::from(area.width), f64::from(area.height));
    let fits = (u64::from(area.width)).checked_mul(u64::from(area.height));
    if w <= 0.0 || h <= 0.0 || fits.is_none_or(|n| n <= max_pixels) {
        return area;
    }
    // The height from the limit and the width from the height, so the pair is within a pixel of the
    // exact ratio; `floor` can leave it a hair over the limit, so step down until it fits.
    let mut sh = ((max_pixels as f64 * h / w).sqrt().floor() as u32).max(1);
    loop {
        let sw = ((w / h * f64::from(sh)).round() as u32).max(1);
        if u64::from(sw).checked_mul(u64::from(sh)).is_none_or(|n| n <= max_pixels) {
            return Size::new(sw, sh);
        }
        if sh <= 1 {
            // The height cannot shrink any further, so the width has to: a document one pixel tall
            // still has to fit under the limit rather than quietly break it.
            let cap = max_pixels.clamp(1, u64::from(u32::MAX)) as u32;
            return Size::new(sw.min(cap).max(1), 1);
        }
        sh -= 1;
    }
}

/// How many strips may be read at once so their source pixels stay inside `MAX_RESIDENT_BYTES`. A
/// strip bigger than the budget is still read, just on its own.
fn strips_in_flight(strip_bytes: usize, n_strips: usize) -> usize {
    (MAX_RESIDENT_BYTES / strip_bytes.max(1)).clamp(1, n_strips.max(1))
}

/// The area read through `sampler` and downscaled by area average to `out` (straight RGBA f32,
/// row-major). Source rows are read in strips, so the full-resolution image is never in memory at
/// once (as `Sampler::rgb_scaled` works), and `cancelled` is asked between strips (`None` once it
/// says so).
fn read_scaled(sampler: &dyn Sampler, area: Rect, out: Size, cancelled: &(dyn Fn() -> bool + Sync)) -> Option<Vec<f32>> {
    let (sw, sh) = (area.width() as usize, area.height() as usize);
    let (ow, oh) = (out.width as usize, out.height as usize);
    if ow == 0 || oh == 0 || sw == 0 || sh == 0 {
        return Some(Vec::new());
    }
    // Which source rows and columns each output pixel averages: these ranges partition the source,
    // so every source pixel is read exactly once.
    let rows = |oy: usize| (oy * sh / oh, (oy + 1) * sh / oh);
    let cols = |ox: usize| (ox * sw / ow, (ox + 1) * sw / ow);
    let rows_per = (STRIP_ROWS / (sh / oh).max(1)).max(1);
    let strips: Vec<usize> = (0..oh.div_ceil(rows_per)).collect();
    // A strip spans at most `ceil(source rows / output rows)` source rows per output row, plus one
    // for the seam, and four bytes per source pixel.
    let strip_bytes = (sh.div_ceil(oh) * rows_per + 1).saturating_mul(sw).saturating_mul(4);
    let batch = strips_in_flight(strip_bytes, strips.len());
    let run = |si: &usize| -> Option<Vec<f32>> {
        if cancelled() {
            return None;
        }
        let oy0 = si * rows_per;
        let oy1 = (oy0 + rows_per).min(oh);
        let (sy0, sy1) = (rows(oy0).0, rows(oy1 - 1).1);
        let strip = sampler.rgba(Rect::new(area.x0, area.y0.saturating_add(sy0 as i32), area.x1, area.y0.saturating_add(sy1 as i32)));
        let mut px = Vec::with_capacity((oy1 - oy0) * ow * 4);
        for oy in oy0..oy1 {
            let (ry0, ry1) = rows(oy);
            let mut acc = vec![0.0f32; ow * 4];
            let mut count = vec![0.0f32; ow];
            for sy in ry0..ry1 {
                let j = sy - sy0;
                let row = strip.get(j * sw..(j + 1) * sw).unwrap_or(&[]);
                for ox in 0..ow {
                    let (cx0, cx1) = cols(ox);
                    let src = row.get(cx0.min(sw)..cx1.min(sw)).unwrap_or(&[]);
                    if src.is_empty() {
                        continue;
                    }
                    let mut sum = [0.0f32; 4];
                    for p in src {
                        for (slot, sample) in sum.iter_mut().zip(p) {
                            *slot += sample;
                        }
                    }
                    for (slot, sample) in acc[ox * 4..ox * 4 + 4].iter_mut().zip(sum) {
                        *slot += sample;
                    }
                    if let Some(n) = count.get_mut(ox) {
                        *n += src.len() as f32;
                    }
                }
            }
            for ox in 0..ow {
                let n = count.get(ox).copied().unwrap_or(0.0).max(1.0);
                for sample in acc[ox * 4..ox * 4 + 4].iter() {
                    px.push((sample / n).clamp(0.0, 1.0));
                }
            }
        }
        Some(px)
    };
    #[cfg(not(target_arch = "wasm32"))]
    let parts: Vec<Option<Vec<f32>>> = {
        use rayon::prelude::*;
        // A batch at a time: the strips inside a batch go out to the workers, and the batch finishes
        // before the next one starts, so at most `batch` strips of source pixels are alive at once.
        let mut parts = Vec::with_capacity(strips.len());
        for chunk in strips.chunks(batch) {
            parts.extend(chunk.par_iter().map(run).collect::<Vec<Option<Vec<f32>>>>());
        }
        parts
    };
    #[cfg(target_arch = "wasm32")]
    let parts: Vec<Option<Vec<f32>>> = strips.iter().map(run).collect();
    let mut scaled = Vec::with_capacity(ow * oh * 4);
    for part in parts {
        scaled.extend(part?);
    }
    Some(scaled)
}

/// Stretch the provider's mask from the size it answered at to the document area, bilinearly
/// (amendment 18). Sample positions are pixel centres; the edge is clamped, never wrapped.
fn upscale_bilinear(mask: &[f32], from: Size, to: Size) -> Vec<f32> {
    let (sw, sh) = (from.width as usize, from.height as usize);
    let (ow, oh) = (to.width as usize, to.height as usize);
    if sw == 0 || sh == 0 || ow == 0 || oh == 0 {
        return Vec::new();
    }
    // Both indices are clamped into the mask, so the last row and column blend with the edge value
    // rather than with zero (which would darken the border of the selection).
    let at = |x: usize, y: usize| mask.get(y.min(sh - 1) * sw + x.min(sw - 1)).copied().unwrap_or(0.0).clamp(0.0, 1.0);
    let mut out = Vec::with_capacity(ow * oh);
    for oy in 0..oh {
        let gy = (oy as f32 + 0.5) * sh as f32 / oh as f32 - 0.5;
        let y0 = gy.max(0.0).floor() as usize;
        let (y0, y1) = (y0.min(sh - 1), y0.min(sh - 1) + 1);
        let fy = (gy - y0 as f32).clamp(0.0, 1.0);
        for ox in 0..ow {
            let gx = (ox as f32 + 0.5) * sw as f32 / ow as f32 - 0.5;
            let x0 = (gx.max(0.0).floor() as usize).min(sw - 1);
            let fx = (gx - x0 as f32).clamp(0.0, 1.0);
            let (top, bottom) = (at(x0, y0) + (at(x0 + 1, y0) - at(x0, y0)) * fx, at(x0, y1) + (at(x0 + 1, y1) - at(x0, y1)) * fx);
            out.push(top + (bottom - top) * fy);
        }
    }
    out
}

/// The coverage over `area` as a `Region`, the shape the classical commands already apply: 8-bit
/// coverage over the bounding box of the pixels it touches. `None` when the mask selects nothing,
/// which the callers read as "no subject found" rather than a reason to clear a selection.
fn mask_region(coverage: &[f32], area: Rect) -> Option<Region> {
    let (w, h) = (area.width() as usize, area.height() as usize);
    if w == 0 || h == 0 {
        return None;
    }
    let bytes: Vec<u8> = coverage.iter().take(w * h).map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8).collect();
    let (mut y0, mut y1, mut x0, mut x1) = (usize::MAX, 0usize, usize::MAX, 0usize);
    for (i, v) in bytes.iter().enumerate() {
        if *v == 0 {
            continue;
        }
        let (y, x) = (i / w, i % w);
        (y0, y1, x0, x1) = (y0.min(y), y1.max(y + 1), x0.min(x), x1.max(x + 1));
    }
    if y0 == usize::MAX {
        return None;
    }
    let width = x1 - x0;
    let mask: Vec<u8> = (y0..y1).flat_map(|y| bytes.get(y * w + x0..y * w + x1).unwrap_or(&[]).to_vec()).collect();
    if mask.len() != width * (y1 - y0) {
        return None;
    }
    let (bx0, by0) = (area.x0.saturating_add(x0 as i32), area.y0.saturating_add(y0 as i32));
    Some(Region { bbox: Rect::new(bx0, by0, bx0.saturating_add(width as i32), by0.saturating_add((y1 - y0) as i32)), mask })
}

/// What one `subject_mask` job produced: the coverage over the whole document area, and where it ran.
struct Masked {
    area: Rect,
    coverage: Vec<f32>,
    ai: Value,
}

/// The shared AI path: ask the manager its limit and the tasks it offers, read the image
/// downscaled, run the job to completion, and stretch the mask back to the document area. `layer`
/// is the layer to sample (`None` = the composite), chosen as the classical commands choose it.
fn mask_job(provider: &Arc<dyn AiProvider>, doc: &Document, layer: Option<LayerId>, sample_all: bool, ctx: &JobCtx) -> Result<Masked> {
    ctx.progress(0.0, "Finding subject");
    // The limit first, as Generative Fill does: for Craft AI Manager this also finds out whether it
    // is running, before any pixels are read. A mask task is never refused for size (amendment 18):
    // the limit says how far down to scale, not whether to stop.
    let max_pixels = match provider.max_pixels(AiTask::SubjectMask) {
        Ok(Some(limit)) if limit > 0 => limit,
        Ok(_) => ai::DEFAULT_MAX_PIXELS,
        Err(error @ (AiError::NoProvider | AiError::TokenMismatch)) => return Err(error.into()),
        Err(_) => ai::DEFAULT_MAX_PIXELS,
    };
    ctx.check()?;
    // The model has to be installed and acknowledged, which is what `/v1/tasks` reports. A manager
    // that cannot answer at all does not offer the task either, so it gets the same sentence.
    let offered = match provider.tasks() {
        Ok(tasks) => tasks.contains(&AiTask::SubjectMask),
        Err(error @ (AiError::NoProvider | AiError::TokenMismatch)) => return Err(error.into()),
        Err(_) => false,
    };
    if !offered {
        return Err(EngineError::Other(NOT_INSTALLED.to_string()));
    }
    let area = doc.bounds();
    if area.is_empty() {
        return Err(EngineError::Other("the document is empty".to_string()));
    }
    let sent = sent_size(area.size(), max_pixels);
    ctx.progress(0.05, "Reading the image");
    let pixels = with_doc_sampler(doc, layer, sample_all, |smp, d| read_scaled(smp, d.bounds(), sent, &|| ctx.cancelled())).ok_or(EngineError::Cancelled)?;
    ctx.check()?;
    let n_sent = usize::try_from(sent.area()).map_err(|_| EngineError::Other("the document is too large for the AI mask task".to_string()))?;
    let req = AiRequest {
        task: AiTask::SubjectMask,
        window: area,
        sent_size: Some(sent),
        pixels,
        format: PixelFormat::RGBA32F,
        // "look everywhere" (amendment 18): the model decides what the subject is.
        coverage: vec![1.0; n_sent],
        prompt: None,
        client_version: format!("photocraft/{}", crate::build_info::VERSION),
    };
    ctx.progress(0.1, "Waiting for Craft AI Manager");
    // `run_to_completion` validates the result and cancels the job when it does not pass, so a mask of
    // the wrong size or with a NaN in it reaches neither the document nor the provider's queue.
    let result = ai::run_to_completion(provider.as_ref(), &req, ctx, POLL_EVERY).map_err(EngineError::from)?;
    let ai = provenance(&result);
    let AiOutput::Mask { coverage, size } = result.output else {
        return Err(EngineError::Other("The AI provider returned an image to a mask request".to_string()));
    };
    ctx.progress(0.9, "Scaling the mask");
    Ok(Masked { area, coverage: upscale_bilinear(&coverage, size, area.size()), ai })
}

/// `select.subject` with `"method": "ai"`: the mask becomes the selection, combined with the current
/// one by `mode`, in one history step through the classical command's own apply code.
pub(crate) fn select_subject(s: &mut Session, p: &Value) -> Result<Value> {
    // The provider comes first, so the message is the same as Generative Fill's with CAIM closed.
    let provider = s.ai_provider().ok_or(AiError::NoProvider)?;
    let sample_all = p.get("sampleAllLayers").and_then(Value::as_bool).unwrap_or(true);
    let m = smartselect_cmds::mode(p, "replace");
    let st = s.active().ok_or(EngineError::NoDocument)?;
    let doc = st.doc.clone();
    // What the classical command samples: the active layer's pixels when sampling one layer, else
    // the composite.
    let target = st.active_layer.filter(|id| !sample_all && doc.layer(*id).and_then(Layer::surface).is_some());
    jobs::run(
        s,
        "Select Subject",
        true,
        move |ctx| mask_job(&provider, &doc, target, sample_all, ctx),
        move |s, masked| {
            let region = mask_region(&masked.coverage, masked.area);
            let mut out = smartselect_cmds::apply(s, "Select Subject", region.as_ref(), m)?;
            if let Some(fields) = out.as_object_mut() {
                fields.insert("ai".to_string(), masked.ai);
            }
            Ok(out)
        },
    )
}

/// `layer.removeBackground` with `"method": "ai"`: the mask becomes the layer's mask in one history
/// step. The pixels never change, and the Background becomes a normal layer first, as the classical
/// command does.
pub(crate) fn remove_background(s: &mut Session, p: &Value) -> Result<Value> {
    let provider = s.ai_provider().ok_or(AiError::NoProvider)?;
    let id = crate::commands::layer_param(s, p)?;
    let doc = &s.active().ok_or(EngineError::NoDocument)?.doc;
    // The same checks as the classical command: an unlocked pixel layer.
    cutout_cmds::check(doc, doc.layer(id).ok_or(EngineError::NoLayer(id))?).map_err(EngineError::Other)?;
    let sample_all = p.get("sampleAllLayers").and_then(Value::as_bool).unwrap_or(false);
    // The model's matte is already soft, so edge refinement is off unless the caller asks for it
    // (amendment 18); when it does, it is the classical command's refinement.
    let refine = p.get("refine").and_then(Value::as_bool).unwrap_or(false);
    jobs::edit_job(
        s,
        "Remove Background",
        move |doc, _, ctx| {
            let masked = mask_job(&provider, doc, Some(id), sample_all, ctx)?;
            let mut region = mask_region(&masked.coverage, masked.area).ok_or_else(|| EngineError::Other("no subject found".to_string()))?;
            if refine {
                ctx.progress(0.95, "Refining edge");
                let refined = with_doc_sampler(doc, Some(id), sample_all, |smp, d| {
                    matting::refine_mask(smp, &matting::region_reader(&region), region.bbox, d.bounds(), &cutout_cmds::REFINE)
                });
                region = refined.unwrap_or(region);
                if ctx.cancelled() {
                    return Err(EngineError::Cancelled);
                }
            }
            crate::extra_cmds::background_to_layer_for_mask(doc, id);
            doc.layer_mut(id).ok_or(EngineError::NoLayer(id))?.mask = Some(LayerMask { surface: matting::region_surface(&region), ..LayerMask::reveal_all() });
            doc.selection = None;
            let b = region.bbox;
            Ok(([b.x0, b.y0, b.width() as i32, b.height() as i32], masked.ai))
        },
        move |(bounds, ai)| json!({ "layer": id.0, "bounds": bounds, "ai": ai }),
    )
}

#[cfg(test)]
#[path = "ai_segment_cmds/tests.rs"]
mod tests;
