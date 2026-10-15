//! Edit … Generative Fill: fill the selection through the AI provider and put the result on a
//! new layer, as one undo step in a background job. Content-Aware Fill (edit_menu_cmds) reads the
//! window and writes the result; only the fill step here differs, and it is a provider call
//! rather than PatchMatch. The window is the selection's bounds plus padding (SPEC amendment 16,
//! `ai::fill_window`), and the provider returns its fill at full strength: the new layer's alpha
//! is the selection's coverage, which is the one and only blend. No network code lives here.

use std::time::Duration;

use photocraft_color::{ColorMode, SampleType};
use serde_json::{Value, json};

use crate::ai::{self, AiError, AiRequest, AiTask};
use crate::commands::CommandSpec;
use crate::edit_menu_cmds::{self, FillWindow, FilledWindow};
use crate::{EngineError, Result, Session};

const CMD: &str = "edit.generativeFill";
const LABEL: &str = "Generative Fill";

/// How long to wait between provider polls on a job worker.
const POLL_EVERY: Duration = Duration::from_millis(100);

/// Amendment 4: a 32-bit document goes to the model as 16-bit integers, so values outside 0–1
/// (HDR highlights, negative values) are clamped. The result says so.
const NOTE_32_BIT: &str = "32-bit values were clamped to 0–1 for the AI model.";

fn bad(msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: CMD.into(), msg: msg.into() }
}

/// Where a fill ran (FEATURES B08): the model, device and provider the job reported, and how long
/// it took. The engine passes them through as the provider said them.
fn provenance(result: &ai::AiResult) -> Value {
    json!({"model": result.model, "device": result.device, "provider": result.provider, "elapsedMs": result.elapsed_ms})
}

fn run(s: &mut Session, p: &Value) -> Result<Value> {
    // The provider comes first: the menu item stays enabled without one so the user gets the
    // message instead of a greyed-out item.
    let provider = s.ai_provider().ok_or(AiError::NoProvider)?;
    let prompt = match p.get("prompt") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) if text.trim().is_empty() => None,
        Some(Value::String(_)) => Some(()),
        Some(_) => return Err(bad("`prompt` must be a string")),
    };
    if prompt.is_some() {
        // Milestone 1 has one provider task, fill_masked_area, which takes no text.
        return Err(EngineError::Other("Prompts aren't supported yet; leave the prompt empty to remove the selection".into()));
    }
    let id = edit_menu_cmds::pixel_layer(s).map_err(EngineError::Other)?;
    let st = s.active().ok_or(EngineError::NoDocument)?;
    let doc = st.doc.clone();
    if doc.mode != ColorMode::Rgb {
        return Err(EngineError::Other("Generative Fill works on RGB documents for now".into()));
    }
    // Same selection errors as Content-Aware Fill, so the two removal commands explain themselves
    // the same way.
    let sel = doc.selection.clone().ok_or_else(|| bad("no selection"))?;
    let hb = sel.content_bounds().intersect(&doc.bounds());
    if hb.is_empty() {
        return Err(bad("the selection is outside the canvas"));
    }
    // The selection's bounds plus padding (amendment 16), not Content-Aware Fill's sampling window.
    let window = ai::fill_window(hb, doc.bounds());
    let surf = doc.layer(id).and_then(|l| l.surface()).ok_or(EngineError::NoLayer(id))?;
    let fmt = surf.format();
    // Photoshop's Sample All Layers checkbox is off unless the caller asks for it.
    let sample_all = match p.get("sampleAllLayers") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(v)) => *v,
        Some(_) => return Err(bad("`sampleAllLayers` must be true or false")),
    };
    // Refuse a window too large to read before allocating for it, as Content-Aware Fill does.
    crate::fill_cmds::window_size(window)?;
    let note = (fmt.sample == SampleType::F32).then_some(NOTE_32_BIT);
    // A background job when started with Session::start: the provider call runs on a worker
    // against the document snapshot, and cancelling it discards the result (jobs::run never
    // reaches the apply step), so the document stays untouched.
    crate::jobs::run(
        s,
        LABEL,
        true,
        move |ctx| {
            ctx.progress(0.0, LABEL);
            // The provider's limit first (for Craft AI Manager this also finds out whether it is
            // running): nothing is read or encoded for a window it would refuse.
            // Only the two answers that mean "this manager cannot do the job at all" stop the fill:
            // it is not running, or it refused our token. Anything else (a blip, a manager that
            // does not know /v1/tasks) falls back to the default limit and the fill goes ahead; a
            // window that really is too big still fails below, and the submit reports the rest.
            let max_pixels = match provider.max_pixels(AiTask::FillMaskedArea) {
                Ok(Some(limit)) => limit,
                Ok(None) => ai::DEFAULT_MAX_PIXELS,
                Err(error @ (AiError::NoProvider | AiError::TokenMismatch)) => return Err(error.into()),
                Err(_) => ai::DEFAULT_MAX_PIXELS,
            };
            ai::check_window_pixels(window, max_pixels)?;
            ctx.check()?;
            let FillWindow { img, bounds, original, cover, n_hole, .. } = edit_menu_cmds::read_fill_window(&doc, id, &sel, window, sample_all, fmt, ctx)?;
            // The provider needs soft coverage over the whole window, so the feathered edge and
            // the antialiased selection boundary reach it; the apply step blends the same way.
            let coverage = crate::fill_cmds::read_window_mask(&sel, window, &|| ctx.cancelled()).ok_or(EngineError::Cancelled)?;
            let req = AiRequest {
                task: AiTask::FillMaskedArea,
                window,
                pixels: img,
                format: fmt,
                coverage,
                prompt: None,
                client_version: format!("photocraft/{}", crate::build_info::VERSION),
            };
            ctx.progress(0.1, "Waiting for Craft AI Manager");
            let result = ai::run_to_completion(provider.as_ref(), &req, ctx, POLL_EVERY).map_err(EngineError::from)?;
            // Provider pixels are outside our input: one NaN or inf would silently corrupt the
            // layer, so the whole result is rejected instead.
            if result.pixels.iter().any(|v| !v.is_finite()) {
                return Err(EngineError::Other("The AI result contains invalid pixel values; the document is unchanged".into()));
            }
            ctx.progress(1.0, LABEL);
            let ai = provenance(&result);
            Ok((FilledWindow { filled: result.pixels, bounds, original, cover, n_hole }, ai))
        },
        move |s, (done, ai)| {
            let name = s.active().map_or_else(|| LABEL.to_string(), |st| edit_menu_cmds::fill_layer_name(&st.doc, LABEL));
            let mut result = edit_menu_cmds::apply_content_aware_fill(s, LABEL, id, "new", fmt, window, done, &name)?;
            // The provenance travels with the result, so a "Generative Fill" layer can be traced
            // to the model that made it (B08; U04 shows it on the layer).
            if let Some(fields) = result.as_object_mut() {
                fields.insert("ai".to_string(), ai);
                if let Some(note) = note {
                    fields.insert("note".to_string(), Value::String(note.to_string()));
                }
            }
            Ok(result)
        },
    )
}

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: CMD,
        label: "Generative Fill…",
        menu: &["Edit"],
        shortcut: None,
        params: r##"{"prompt":string?,"sampleAllLayers":bool=false} … {layer,filled,window,ai:{model,device,provider,elapsedMs},note?} (fills the selection through the AI provider onto a new "Generative Fill" layer; ai says where the work ran; note says when 32-bit values were clamped)"##,
        enabled: edit_menu_cmds::can_fill,
        run,
        journal: true,
    }]
}

#[cfg(test)]
#[path = "generative_cmds/tests.rs"]
mod tests;
