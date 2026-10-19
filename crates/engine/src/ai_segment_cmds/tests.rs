//! The `method: "ai"` paths of Select Subject and Remove Background (SPEC amendment 18): the
//! mask the provider answers with becomes the selection or the layer mask, stretched to the
//! document area, and a run that fails leaves the document exactly as it was.

use std::sync::Arc;

use photocraft_algo::segment::Rng;
use photocraft_algo::selection;
use photocraft_doc::LayerId;
use photocraft_geom::{Rect, Size};
use serde_json::json;

use super::*;
use crate::ai::testing::{FakeProvider, fake_mask};
use crate::jobs::Started;

const RB: &str = "layer.removeBackground";

/// Coverage is stored as bytes in a region, a selection and a layer mask, so a comparison allows
/// one quantisation step.
const TOL: f32 = 1.0 / 255.0;

/// A blank white document.
fn white(w: u32, h: u32) -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": w, "height": h, "depth": 8, "background": "white"})).unwrap();
    s
}

/// Textured disc (centre (100, 80), radius 45) on a noisy background: the fixture the classical
/// Select Subject finds, so both methods can be compared on the same document.
fn disc() -> Session {
    let mut rng = Rng::new(2);
    let nz: Vec<f32> = (0..200 * 160).map(|_| rng.normal()).collect();
    let mut s = white(200, 160);
    s.edit("paint", |doc, _| {
        let bg = doc.layers[0].surface_mut().unwrap();
        let n = bg.channels();
        let mut data = Vec::with_capacity((200 * 160) as usize * n);
        for y in 0..160 {
            for x in 0..200 {
                let (dx, dy) = (x as f32 + 0.5 - 100.0, y as f32 + 0.5 - 80.0);
                let inside = dx * dx + dy * dy <= 45.0 * 45.0;
                let c = if inside { [0.85, 0.2, 0.15] } else { [0.25, 0.45, 0.7] };
                let noise = 0.05 * nz[(y * 200 + x) as usize];
                data.extend_from_slice(&c.map(|v| (v + noise).clamp(0.0, 1.0))[..n.min(3)]);
                if n == 4 {
                    data.push(1.0);
                }
            }
        }
        bg.write_region(Rect::new(0, 0, 200, 160), &data);
        Ok(())
    })
    .unwrap();
    s
}

/// Install `provider` and hand back a handle on it, so a test can read what the provider saw.
fn use_provider(s: &mut Session, provider: FakeProvider) -> Arc<FakeProvider> {
    let provider = Arc::new(provider);
    s.set_ai_provider(Some(provider.clone()));
    provider
}

/// A provider that offers the subject model and answers on the first poll.
fn mask_provider() -> FakeProvider {
    FakeProvider { offer_mask: true, ..Default::default() }
}

/// A provider that answers at the size a 60x40 document is downscaled to (30x20), so the tests
/// cover the downscale and the upscale in one run.
fn small_mask_provider() -> FakeProvider {
    FakeProvider { offer_mask: true, max_pixels: Some(Ok(Some(600))), ..Default::default() }
}

fn active_id(s: &Session) -> LayerId {
    s.active().unwrap().active_layer.unwrap()
}

fn layer(s: &Session, id: LayerId) -> &Layer {
    s.active().unwrap().doc.layer(id).unwrap()
}

/// The selection's coverage over `area`, in row order (zero where there is no selection).
fn coverage(s: &Session, area: Rect) -> Vec<f32> {
    selection::mask_from_surface(s.active().unwrap().doc.selection.as_ref(), area)
}

/// The layer mask's coverage over `area`, in row order (zero where the layer has no mask).
fn mask_coverage(s: &Session, id: LayerId, area: Rect) -> Vec<f32> {
    let l = layer(s, id);
    match l.mask.as_ref() {
        None => vec![0.0; area.width() as usize * area.height() as usize],
        Some(m) => (area.y0..area.y1).flat_map(|y| (area.x0..area.x1).map(move |x| m.value(x, y))).collect(),
    }
}

/// What the provider answered, recorded by the fake on submit.
fn sent(provider: &Arc<FakeProvider>) -> AiRequest {
    provider.state.lock().unwrap().request.clone().unwrap()
}

/// Two coverages must agree within one byte step.
fn same(got: &[f32], want: &[f32], what: &str) {
    assert_eq!(got.len(), want.len(), "{what}: length");
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() <= TOL, "{what}: pixel {i}: {g} vs {w}");
    }
}

/// The provider's mask stretched to `to` by hand, so the test does not reuse the code it checks:
/// sample at pixel centres, bilinear, with the edge clamped (amendment 18).
fn stretched(mask: &[f32], from: Size, to: Size) -> Vec<f32> {
    let (fw, fh) = (i64::from(from.width), i64::from(from.height));
    let at = |x: i64, y: i64| f64::from(mask[y.min(fh - 1).max(0) as usize * from.width as usize + x.min(fw - 1).max(0) as usize]);
    let row = |y: i64, x: i64, fx: f64| at(x, y) + (at(x + 1, y) - at(x, y)) * fx;
    let mut out = Vec::with_capacity(to.width as usize * to.height as usize);
    for oy in 0..to.height {
        let gy = (f64::from(oy) + 0.5) * f64::from(from.height) / f64::from(to.height) - 0.5;
        let y = (gy.max(0.0).floor() as i64).min(fh - 1);
        let fy = (gy - y as f64).clamp(0.0, 1.0);
        for ox in 0..to.width {
            let gx = (f64::from(ox) + 0.5) * f64::from(from.width) / f64::from(to.width) - 0.5;
            let x = (gx.max(0.0).floor() as i64).min(fw - 1);
            let fx = (gx - x as f64).clamp(0.0, 1.0);
            out.push((row(y, x, fx) + (row(y + 1, x, fx) - row(y, x, fx)) * fy) as f32);
        }
    }
    out
}

#[test]
fn the_default_method_is_the_classical_path_unchanged() {
    let area = Rect::new(0, 0, 200, 160);
    // No provider is installed at all: the classical path still works, with or without the param.
    let mut a = disc();
    let mut b = disc();
    let ra = a.execute("select.subject", json!({})).unwrap();
    let rb = b.execute("select.subject", json!({"method": "classical"})).unwrap();
    assert_eq!(ra, rb, "the same answer with and without the param");
    assert_eq!(ra["changed"], json!(true), "the classical heuristic still finds the disc: {ra}");
    assert_eq!(coverage(&a, area), coverage(&b, area), "the same selection");
    let mut c = disc();
    let mut d = disc();
    let rc = c.execute(RB, json!({})).unwrap();
    let rd = d.execute(RB, json!({"method": null})).unwrap();
    // Each session hands out its own layer ids, so compare the answers with one id swapped in.
    let (id_c, id_d) = (active_id(&c), active_id(&d));
    let (mut rc, rd) = (rc, rd);
    rc["layer"] = rd["layer"].clone();
    assert_eq!(rc, rd, "a null method is the classical one");
    assert_eq!(mask_coverage(&c, id_c, area), mask_coverage(&d, id_d, area), "the same layer mask");
    // The AI path is the only thing that needs a provider.
    assert!(matches!(a.execute("select.subject", json!({"method": "ai"})), Err(EngineError::Other(_))));
}

#[test]
fn an_unknown_method_names_the_two_methods() {
    let mut s = disc();
    for params in [json!({"method": "neural"}), json!({"method": 3}), json!({"method": ["ai"]})] {
        for cmd in ["select.subject", RB] {
            let message = s.execute(cmd, params.clone()).unwrap_err().to_string();
            assert!(message.contains("classical") && message.contains("ai"), "{cmd} {params}: {message}");
        }
    }
    assert!(s.active().unwrap().doc.selection.is_none(), "a rejected param changes nothing");
    assert!(layer(&s, active_id(&s)).mask.is_none(), "a rejected param changes nothing");
}

#[test]
fn ai_select_subject_stores_the_mask_stretched_to_the_document() {
    let (area, small) = (Rect::new(0, 0, 60, 40), Size::new(30, 20));
    let mut s = white(60, 40);
    let provider = use_provider(&mut s, small_mask_provider());
    let r = s.execute("select.subject", json!({"method": "ai"})).unwrap();
    assert_eq!(r["changed"], json!(true), "{r}");
    assert_eq!(r["bounds"], json!([0, 0, 60, 40]), "the mask touches every edge: {r}");
    let want = stretched(&fake_mask(small), small, Size::new(60, 40));
    same(&coverage(&s, area), &want, "the selection is the provider's mask, upscaled");
    assert!((coverage(&s, area)[20 * 60 + 30] - 1.0).abs() <= TOL, "the middle of the disc is fully selected");
    assert_eq!(coverage(&s, area)[0], 0.0, "the corner is not");
    // The image really was downscaled: 60x40 is 2,400 pixels, and the limit was 600.
    let req = sent(&provider);
    assert_eq!(req.task, AiTask::SubjectMask);
    assert_eq!(req.window, area, "the window is the whole document area");
    assert_eq!(req.sent_image_size(), small, "sent at a third of the width and height");
    assert_eq!(req.format, PixelFormat::RGBA32F);
    assert_eq!(req.pixels.len(), (small.width * small.height) as usize * 4, "one RGBA pixel per sent pixel");
    assert_eq!(req.coverage, vec![1.0; (small.width * small.height) as usize], "look everywhere");
    assert_eq!(req.prompt, None, "a mask task has no prompt");
}

#[test]
fn ai_select_subject_honours_every_mode() {
    let (area, small) = (Rect::new(0, 0, 60, 40), Size::new(30, 20));
    let want = stretched(&fake_mask(small), small, Size::new(60, 40));
    for mode in ["replace", "add", "subtract", "intersect"] {
        let mut s = white(60, 40);
        use_provider(&mut s, small_mask_provider());
        s.execute("select.rect", json!({"x": 0, "y": 0, "width": 20, "height": 20})).unwrap();
        let old = coverage(&s, area);
        let r = s.execute("select.subject", json!({"method": "ai", "mode": mode})).unwrap();
        assert_eq!(r["changed"], json!(true), "{mode}: {r}");
        let got = coverage(&s, area);
        assert_eq!(got.len(), want.len(), "{mode}");
        for (i, (g, (w, o))) in got.iter().zip(want.iter().zip(&old)).enumerate() {
            let expected = match mode {
                "add" => o.max(*w),
                "subtract" => (o - *w).max(0.0),
                "intersect" => o.min(*w),
                _ => *w,
            };
            assert!((g - expected).abs() <= 2.0 * TOL, "{mode}: pixel {i}: {g} vs {expected}");
        }
    }
}

#[test]
fn one_undo_puts_the_old_selection_back() {
    let area = Rect::new(0, 0, 60, 40);
    let mut s = white(60, 40);
    use_provider(&mut s, mask_provider());
    s.execute("select.rect", json!({"x": 4, "y": 4, "width": 10, "height": 10})).unwrap();
    let before = coverage(&s, area);
    let steps = s.active().unwrap().history.past_len();
    s.execute("select.subject", json!({"method": "ai"})).unwrap();
    assert_eq!(s.active().unwrap().history.past_len(), steps + 1, "one history step");
    assert_ne!(coverage(&s, area), before, "the AI mask replaced the rectangle");
    s.execute("edit.undo", json!({})).unwrap();
    same(&coverage(&s, area), &before, "one undo restores the old selection");
}

#[test]
fn ai_remove_background_masks_the_layer_and_never_touches_the_pixels() {
    let area = Rect::new(0, 0, 60, 40);
    let mut s = white(60, 40);
    let id = active_id(&s);
    let pixels = layer(&s, id).surface().unwrap().read_region(area);
    let steps = s.active().unwrap().history.past_len();
    use_provider(&mut s, mask_provider());
    let r = s.execute(RB, json!({"method": "ai"})).unwrap();
    assert_eq!(r["layer"], id.0, "{r}");
    assert_eq!(r["bounds"], json!([0, 0, 60, 40]), "{r}");
    assert_eq!(layer(&s, id).surface().unwrap().read_region(area), pixels, "the layer's pixels are byte-identical");
    let size = Size::new(60, 40);
    same(&mask_coverage(&s, id, area), &stretched(&fake_mask(size), size, size), "the layer mask is the provider's mask");
    assert!(layer(&s, id).mask.as_ref().unwrap().enabled, "the mask reveals the subject");
    assert!(s.active().unwrap().doc.selection.is_none(), "as classical does, it clears the selection");
    assert_ne!(layer(&s, id).name, "Background", "the Background became a normal layer first");
    assert_eq!(s.active().unwrap().history.past_len(), steps + 1, "one history step");
    s.execute("edit.undo", json!({})).unwrap();
    assert!(layer(&s, id).mask.is_none(), "one undo removes the mask");
}

#[test]
fn ai_remove_background_refuses_a_locked_or_non_pixel_layer() {
    let mut s = white(60, 40);
    use_provider(&mut s, mask_provider());
    s.execute("layer.setProps", json!({"locks": {"pixels": true}})).unwrap();
    let message = s.execute(RB, json!({"method": "ai"})).unwrap_err().to_string();
    assert!(message.contains("locked"), "a locked layer: {message}");
    assert!(layer(&s, active_id(&s)).mask.is_none());

    let mut s = white(60, 40);
    use_provider(&mut s, mask_provider());
    s.execute("type.create", json!({"text": "Hi", "size": 24, "x": 10, "y": 20})).unwrap();
    let message = s.execute(RB, json!({"method": "ai"})).unwrap_err().to_string();
    assert!(message.contains("layer"), "a text layer: {message}");
}

#[test]
fn ai_without_a_manager_says_it_is_not_running() {
    for cmd in ["select.subject", RB] {
        let mut s = disc();
        let message = s.execute(cmd, json!({"method": "ai"})).unwrap_err().to_string();
        assert_eq!(message, "Craft AI Manager isn't running", "{cmd}");
        assert!(s.active().unwrap().doc.selection.is_none(), "{cmd}: nothing changed");
        assert!(layer(&s, active_id(&s)).mask.is_none(), "{cmd}: nothing changed");
    }
}

#[test]
fn ai_without_the_subject_model_says_the_exact_sentence() {
    for cmd in ["select.subject", RB] {
        let mut s = white(60, 40);
        use_provider(&mut s, FakeProvider::default());
        let before = s.active().unwrap().revision;
        let message = s.execute(cmd, json!({"method": "ai"})).unwrap_err().to_string();
        assert_eq!(message, NOT_INSTALLED, "{cmd}");
        assert_eq!(s.active().unwrap().revision, before, "{cmd}: nothing changed");
    }
}

#[test]
fn a_cancelled_mask_run_changes_nothing() {
    for cmd in ["select.subject", RB] {
        let mut s = white(60, 40);
        let (before, id) = (s.active().unwrap().revision, active_id(&s));
        use_provider(&mut s, mask_provider());
        let ctx = JobCtx::new();
        ctx.cancel();
        s.set_inline_job_ctx(Some(ctx.clone()));
        assert!(matches!(s.execute(cmd, json!({"method": "ai"})), Err(EngineError::Cancelled)), "{cmd}");
        assert_eq!(ctx.message(), "Finding subject", "{cmd}");
        assert_eq!(s.active().unwrap().revision, before, "{cmd}: the document is unchanged");
        assert!(s.active().unwrap().doc.selection.is_none(), "{cmd}");
        assert!(layer(&s, id).mask.is_none(), "{cmd}");
        s.set_inline_job_ctx(None);
    }
}

#[test]
fn a_mask_of_the_wrong_size_is_rejected_and_the_job_cancelled() {
    let mut s = white(60, 40);
    let (before, id) = (s.active().unwrap().revision, active_id(&s));
    let provider = use_provider(&mut s, FakeProvider { offer_mask: true, wrong_size: true, ..Default::default() });
    let message = s.execute(RB, json!({"method": "ai"})).unwrap_err().to_string();
    assert!(message.contains("does not match"), "{message}");
    assert_eq!(s.active().unwrap().revision, before, "the document is unchanged");
    assert!(layer(&s, id).mask.is_none());
    assert_eq!(provider.state.lock().unwrap().cancels, 1, "the job was cancelled on the provider");
}

#[test]
fn a_mask_with_a_nan_is_rejected() {
    let mut s = white(60, 40);
    let before = s.active().unwrap().revision;
    use_provider(&mut s, FakeProvider { offer_mask: true, bad_pixel: true, ..Default::default() });
    let message = s.execute("select.subject", json!({"method": "ai"})).unwrap_err().to_string();
    assert!(message.contains("invalid pixel values"), "{message}");
    assert_eq!(s.active().unwrap().revision, before, "the document is unchanged");
    assert!(s.active().unwrap().doc.selection.is_none());
}

#[test]
fn a_big_document_is_scaled_down_and_the_selection_still_covers_the_canvas() {
    let mut s = white(6000, 4000);
    let provider = use_provider(&mut s, FakeProvider { offer_mask: true, max_pixels: Some(Ok(Some(1_048_576))), ..Default::default() });
    let r = s.execute("select.subject", json!({"method": "ai"})).unwrap();
    assert_eq!(r["bounds"], json!([0, 0, 6000, 4000]), "the selection covers the whole canvas: {r}");
    let req = sent(&provider);
    let sent = req.sent_image_size();
    let n = u64::from(sent.width) * u64::from(sent.height);
    assert!(n <= 1_048_576, "sent {sent:?} is over the limit");
    assert!((f64::from(sent.width) - 1.5 * f64::from(sent.height)).abs() <= 1.0, "sent {sent:?} is not the same aspect ratio");
    assert!(u64::from(sent.width + 1) * u64::from(sent.height + 1) > 1_048_576, "sent {sent:?} is not the largest size that fits");
    assert_eq!(req.pixels.len(), n as usize * 4, "the downscaled image");
    assert_eq!(req.coverage.len(), n as usize, "coverage at the sent size");
}

#[test]
fn both_results_say_where_the_mask_ran() {
    for cmd in ["select.subject", RB] {
        let mut s = white(60, 40);
        use_provider(&mut s, mask_provider());
        let out = s.execute(cmd, json!({"method": "ai"})).unwrap();
        let ai = out["ai"].as_object().unwrap_or_else(|| panic!("{cmd}: no provenance in {out}"));
        assert_eq!(ai["model"].as_str().unwrap(), "solid-fill", "{cmd}");
        assert_eq!(ai["device"].as_str().unwrap(), "CPU", "{cmd}");
        assert_eq!(ai["provider"].as_str().unwrap(), "local fake", "{cmd}");
        assert_eq!(ai["elapsedMs"].as_u64().unwrap(), 12, "{cmd}");
        assert_eq!(ai["modelVersion"].as_str().unwrap(), "epoch_244", "{cmd}");
        assert_eq!(ai["modelSha256"].as_str().unwrap(), "58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c", "{cmd}");
        assert_eq!(ai["deviceRequested"].as_str().unwrap(), "auto", "{cmd}");
        assert_eq!(ai["deviceName"].as_str().unwrap(), "CPU", "{cmd}");
        assert_eq!(ai["loadMs"].as_u64().unwrap(), 5_124, "{cmd}");
        assert_eq!(ai["runMs"].as_u64().unwrap(), 7_624, "{cmd}");
        assert_eq!(ai.len(), 10, "{cmd}: exactly the provenance the fake reported: {ai:?}");
    }
    // A background job carries the same provenance, so the control channel and MCP see it too.
    let mut s = white(60, 40);
    use_provider(&mut s, mask_provider());
    let id = match s.start("select.subject", json!({"method": "ai"})).unwrap() {
        Started::Job(id) => id,
        Started::Done(v) => panic!("expected a background job: {v}"),
    };
    let job = s.wait_job(id).unwrap();
    assert_eq!(job["ai"]["model"].as_str().unwrap(), "solid-fill", "{job}");
    assert_eq!(job["ai"]["modelVersion"].as_str().unwrap(), "epoch_244", "{job}");
}

#[test]
fn sent_size_keeps_the_ratio_under_the_limit() {
    assert_eq!(sent_size(Size::new(60, 40), 64_000_000), Size::new(60, 40), "a document that fits is sent as it is");
    assert_eq!(sent_size(Size::new(1000, 1000), 1_000_000), Size::new(1000, 1000), "exactly on the limit stays");
    let big = sent_size(Size::new(6000, 4000), 1_048_576);
    assert_eq!(big, Size::new(1254, 836), "the largest 3:2 size under 1,048,576");
    let tall = sent_size(Size::new(4000, 6000), 1_000_000);
    assert!(u64::from(tall.width) * u64::from(tall.height) <= 1_000_000, "{tall:?}");
    assert!((f64::from(tall.width) * 6000.0 - f64::from(tall.height) * 4000.0).abs() <= 6000.0, "{tall:?} keeps the ratio");
    // A document that can only shrink one way still has to fit: the other side takes the strain.
    for (area, limit) in [(Size::new(1200, 1), 600), (Size::new(1, 1200), 600), (Size::new(4000, 1), 1_000)] {
        let got = sent_size(area, limit);
        assert!(u64::from(got.width) * u64::from(got.height) <= limit, "{area:?} under {limit} sent {got:?}, which is over the limit");
        assert!(got.width >= 1 && got.height >= 1, "{area:?} under {limit} sent {got:?}, which is empty");
        assert!(got.width <= area.width && got.height <= area.height, "{area:?} under {limit} sent {got:?}, which grew");
    }
}

#[test]
fn the_downscale_never_keeps_the_whole_canvas_in_memory() {
    // The 6000x4000 document sent at 1254x836: a strip is 64 output rows, so at most 5 source rows
    // per output row plus the seam, and the sampler answers `[f32; 4]` - sixteen bytes for every
    // source pixel of the 6000-pixel row, which is about 30 MB of source pixels per strip.
    let strip_bytes = (5 * 64 + 1) * 6000 * 16;
    let batch = strips_in_flight(strip_bytes, 14);
    assert!((1..14).contains(&batch), "fourteen strips of 30 MB cannot all be in flight, got {batch}");
    assert!(batch * strip_bytes <= MAX_RESIDENT_BYTES, "{batch} strips of {strip_bytes} bytes stay inside the budget");
    assert_eq!(strips_in_flight(64 * 8 * 16, 3), 3, "small strips are not throttled");
    assert_eq!(strips_in_flight(MAX_RESIDENT_BYTES * 4, 9), 1, "an oversized strip is read on its own");
    assert_eq!(strips_in_flight(0, 5), 5, "an empty strip is not a division by zero");
    assert_eq!(strips_in_flight(strip_bytes, 0), 1, "no strips still asks for at least one");
}

#[test]
fn a_huge_document_only_builds_the_mask_it_can_touch() {
    // The largest document `file.new` allows, with a provider mask that selects one corner. The
    // upscale has to build the corner, not a float for every one of ninety billion pixels.
    let (doc, from) = (Size::new(300_000, 300_000), Size::new(100, 100));
    let mut corner = vec![0.0; 100 * 100];
    corner[50 * 100 + 50] = 1.0;
    let bbox = coverage_bbox(&corner, from).unwrap();
    assert_eq!(bbox, Rect::new(50, 50, 51, 51), "the box of the mask the provider answered with");
    let touched = touched_rect(bbox, from, doc);
    let pixels = u64::from(touched.width()) * u64::from(touched.height());
    assert!(pixels > 0, "a mask that selects something has to build something");
    assert!(pixels < 50_000_000, "one mask pixel reaches {pixels} document pixels, not the canvas");
    assert!(pixels <= MAX_MASK_PIXELS, "so the upscale allocates {pixels} floats, not ninety billion");
    // A mask that fills the provider frame on that document really is too big to build: the command
    // says so instead of letting the allocation abort the process.
    let full = vec![1.0; 100 * 100];
    let full_box = touched_rect(coverage_bbox(&full, from).unwrap(), from, doc);
    assert_eq!(full_box, Rect::new(0, 0, 300_000, 300_000), "a full mask touches every document pixel");
    assert!(u64::from(full_box.width()) * u64::from(full_box.height()) > MAX_MASK_PIXELS, "and that is over the cap, so it is refused");
    assert!(coverage_bbox(&vec![0.0; 100], from).is_none(), "a mask that selects nothing has no box");
    assert!(coverage_bbox(&[0.0; 4], Size::new(0, 0)).is_none(), "an empty answer has no box either");
}

#[test]
fn cropping_the_upscale_does_not_change_the_region() {
    // The crop is a saving, never a change: outside the box the mask can reach, the full upscale is
    // zero, so the Region the commands apply is the same either way.
    let (from, doc) = (Size::new(6, 4), Size::new(23, 17));
    let mut mask = vec![0.0; 24];
    for (i, v) in mask.iter_mut().enumerate() {
        *v = if i % 7 == 3 {
            0.75
        } else if i % 5 == 2 {
            0.25
        } else {
            0.0
        };
    }
    let touched = touched_rect(coverage_bbox(&mask, from).unwrap(), from, doc);
    let whole = upscale_bilinear(&mask, from, doc, Rect::new(0, 0, 23, 17));
    let part = upscale_bilinear(&mask, from, doc, touched);
    for y in 0..17usize {
        for x in 0..23usize {
            let inside = x >= touched.x0 as usize && x < touched.x1 as usize && y >= touched.y0 as usize && y < touched.y1 as usize;
            if !inside {
                assert_eq!(whole[y * 23 + x], 0.0, "({x},{y}) is outside the touched box, so the full upscale has to be zero");
            }
        }
    }
    let full = mask_region(&whole, Rect::new(0, 0, 23, 17)).unwrap();
    let cropped = mask_region(&part, touched).unwrap();
    assert_eq!(full.bbox, cropped.bbox, "the same box");
    assert_eq!(full.mask, cropped.mask, "and the same 8-bit coverage in it");
}

#[test]
fn the_upscaled_mask_blends_with_the_edge_not_with_zero() {
    let out = upscale_bilinear(&[1.0; 4], Size::new(2, 2), Size::new(5, 5), Rect::new(0, 0, 5, 5));
    assert!(out.iter().all(|v| (v - 1.0).abs() < 1e-6), "a full mask stays full to the border: {out:?}");
    let flat = upscale_bilinear(&[0.0; 9], Size::new(3, 3), Size::new(3, 3), Rect::new(0, 0, 3, 3));
    assert_eq!(flat, vec![0.0; 9], "the same size is the identity");
    // A sub-rect is the same pixels, only cheaper: the crop the command makes changes nothing.
    let soft = [0.0, 0.4, 1.0, 0.2];
    let whole = upscale_bilinear(&soft, Size::new(2, 2), Size::new(7, 5), Rect::new(0, 0, 7, 5));
    let part = upscale_bilinear(&soft, Size::new(2, 2), Size::new(7, 5), Rect::new(2, 1, 6, 4));
    for (row, slice) in part.chunks(4).enumerate() {
        let y = 1 + row;
        assert_eq!(slice, &whole[y * 7 + 2..y * 7 + 6], "the sub-rect is the slice of the whole");
    }
}

#[test]
fn a_mask_that_selects_nothing_is_not_a_selection() {
    let area = Rect::new(0, 0, 8, 6);
    assert!(mask_region(&[0.0; 8 * 6], area).is_none(), "an empty mask means no subject found");
    let mut coverage = vec![0.0; 8 * 6];
    let spot = |x: usize, y: usize| y * 8 + x;
    coverage[spot(2, 1)] = 1.0;
    coverage[spot(3, 2)] = 0.5;
    let region = mask_region(&coverage, area).unwrap();
    assert_eq!(region.bbox, Rect::new(2, 1, 4, 3), "the box of the pixels it touches");
    assert_eq!(region.mask, vec![255, 0, 0, 128], "8-bit coverage over that box");
}

/// B16 (amendment 18): the AI mask leaves a record of the model on the layer it masked, in the same
/// history step as the mask; undo takes both, redo brings both back.
#[test]
fn the_ai_mask_records_the_model_and_undo_takes_the_record_with_the_mask() {
    let mut s = white(60, 40);
    let id = active_id(&s);
    use_provider(&mut s, mask_provider());
    s.execute(RB, json!({"method": "ai"})).unwrap();
    let record = ai_provenance::read(layer(&s, id)).unwrap_or_else(|| panic!("the masked layer carries no record"));
    assert_eq!(record.command, ai_provenance::CMD_REMOVE_BACKGROUND, "which command masked the layer");
    assert_eq!(record.model, "solid-fill", "the model that ran");
    assert_eq!(record.model_version.as_deref(), Some("epoch_244"), "the model build that ran");
    assert_eq!(record.model_sha256.as_deref(), Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c"), "the model file it ran");
    assert_eq!(record.device, "CPU", "the backend that ran");
    assert_eq!(record.device_name.as_deref(), Some("CPU"), "the hardware it ran on");
    assert!(layer(&s, id).mask.is_some(), "the mask is there");
    s.execute("edit.undo", json!({})).unwrap();
    assert!(layer(&s, id).mask.is_none(), "undo took the mask");
    assert_eq!(ai_provenance::read(layer(&s, id)), None, "and the record with it");
    s.execute("edit.redo", json!({})).unwrap();
    assert!(layer(&s, id).mask.is_some(), "redo brought the mask back");
    assert_eq!(ai_provenance::read(layer(&s, id)), Some(record), "and the record too");
}

/// The classical path has no model to remember, so it writes nothing (PC-11).
#[test]
fn the_classical_mask_records_nothing() {
    let mut s = disc();
    let id = active_id(&s);
    s.execute(RB, json!({"method": "classical"})).unwrap();
    assert!(layer(&s, id).mask.is_some(), "the classical mask landed");
    assert_eq!(ai_provenance::read(layer(&s, id)), None, "with no record on the layer");
}

/// Select Subject makes a selection, not a layer, so the provenance stays in the command result
/// (amendment 18): nothing is stored on the layer that was sampled.
#[test]
fn select_subject_stores_no_record_on_the_layer() {
    let mut s = white(60, 40);
    let id = active_id(&s);
    use_provider(&mut s, mask_provider());
    let out = s.execute("select.subject", json!({"method": "ai"})).unwrap();
    assert_eq!(out["ai"]["model"].as_str().unwrap(), "solid-fill", "the result still says where it ran: {out}");
    assert_eq!(ai_provenance::read(layer(&s, id)), None, "a selection is not a layer result");
}
