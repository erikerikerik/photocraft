use super::*;
use crate::ai::testing::{FakeProvider, request};
use crate::{EngineError, Session, jobs::JobCtx};
use std::sync::Arc;

#[test]
fn ai_submit_poll_result_happy_path() {
    let p = FakeProvider { polls: 2, ..Default::default() };
    assert_eq!(p.tasks().unwrap(), vec![AiTask::FillMaskedArea]);
    let req = request(PixelFormat::RGBA32F);
    let id = p.submit(&req).unwrap();
    assert!(matches!(p.poll(&id).unwrap(), AiJobState::Queued { .. }));
    assert!(matches!(p.poll(&id).unwrap(), AiJobState::Running { .. }));
    assert_eq!(p.poll(&id).unwrap(), AiJobState::Done);
    let r = p.result(&id).unwrap();
    // Amendment 16: the fill at full strength wherever coverage > 0 (the 0.5 pixel too), the input
    // where it is 0, and a covered pixel made opaque. PhotoCraft blends by coverage when it
    // composites the layer, once.
    assert_eq!(r.pixels, vec![0.25, 0.25, 0.25, 0.25, 0.75, 0.75, 0.75, 1.0, 0.75, 0.75, 0.75, 1.0]);
    assert_eq!(req.pixels, vec![0.25; 12]);
    let json = serde_json::to_value(&r).unwrap();
    assert_eq!(json["model"], "solid-fill");
    assert_eq!(json["device"], "CPU");
    assert_eq!(json["provider"], "local fake");
    assert_eq!(json["elapsed_ms"], 12);
    assert_eq!((r.model.as_str(), r.device.as_str(), r.provider.as_str(), r.elapsed_ms), ("solid-fill", "CPU", "local fake", 12));
}

/// BF-02 follow-up (amendment 16): like Craft AI Manager, the fake makes a covered pixel opaque.
/// Alpha has no fill to blend towards, so the fill layer's alpha is the coverage and nothing
/// blends it twice. Formats with no alpha channel are unchanged.
#[test]
fn ai_the_fake_makes_a_covered_pixel_opaque() {
    for format in [PixelFormat::RGBA8, PixelFormat::RGBA32F, PixelFormat::GRAYA8, PixelFormat::CMYKA8, PixelFormat::GRAY8] {
        let p = FakeProvider { polls: 1, ..Default::default() };
        let req = request(format);
        let id = p.submit(&req).unwrap();
        let r = p.result(&id).unwrap();
        let (n, colour) = (format.channels(), format.mode.color_channels());
        for (at, coverage) in [0.0f32, 0.5, 1.0].into_iter().enumerate() {
            let pixel = &r.pixels[at * n..(at + 1) * n];
            let filled = coverage > 0.0;
            let want = if filled { 0.75 } else { 0.25 };
            assert_eq!(&pixel[..colour], vec![want; colour].as_slice(), "{format:?} at coverage {coverage}: the colour channels");
            match pixel.get(colour) {
                Some(alpha) => assert_eq!(*alpha, if filled { 1.0 } else { 0.25 }, "{format:?} at coverage {coverage}: a covered pixel is opaque"),
                None => assert_eq!(n, colour, "{format:?} has no alpha channel to make opaque"),
            }
        }
    }
}

#[test]
fn ai_completion_progress_reaches_one_at_all_depths_and_layouts() {
    for format in [PixelFormat::RGBA8, PixelFormat::RGBA16, PixelFormat::RGBA32F, PixelFormat::GRAY8, PixelFormat::CMYKA8] {
        let p = FakeProvider { polls: 3, ..Default::default() };
        let ctx = JobCtx::new();
        let req = request(format);
        let r = run_to_completion(&p, &req, &ctx, Duration::ZERO).unwrap();
        assert_eq!(ctx.fraction(), 1.0);
        assert_eq!(ctx.message(), "AI job complete");
        assert_eq!(r.format, format);
        assert_eq!(r.window, req.window);
    }
}

#[test]
fn ai_cancel_mid_job_calls_provider_even_when_cancel_fails() {
    for cancel_fails in [false, true] {
        let ctx = JobCtx::new();
        let p = FakeProvider { never_finish: true, polls: 4, cancel_at: Some((2, ctx.clone())), cancel_fails, ..Default::default() };
        assert_eq!(run_to_completion(&p, &request(PixelFormat::RGBA8), &ctx, Duration::ZERO), Err(AiError::Cancelled));
        assert!(p.state.lock().unwrap().cancelled);
        assert!(ctx.fraction() > 0.0 && ctx.fraction() < 1.0);
    }
}

#[test]
fn ai_already_cancelled_does_not_submit() {
    let p = FakeProvider::default();
    let ctx = JobCtx::new();
    ctx.cancel();
    assert_eq!(run_to_completion(&p, &request(PixelFormat::RGBA8), &ctx, Duration::ZERO), Err(AiError::Cancelled));
    assert!(p.state.lock().unwrap().request.is_none());
    assert!(matches!(EngineError::from(AiError::Cancelled), EngineError::Cancelled));
}

#[test]
fn ai_failure_message_passes_through() {
    let p = FakeProvider { fail: Some("The model could not run on this device".into()), ..Default::default() };
    let e = run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).unwrap_err();
    assert_eq!(e.to_string(), "The model could not run on this device");
    assert_eq!(EngineError::from(e).to_string(), "The model could not run on this device");
}

#[test]
fn ai_provider_cancelled_is_cancelled() {
    let p = FakeProvider { remote_cancelled: true, ..Default::default() };
    assert_eq!(run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO), Err(AiError::Cancelled));
}

#[test]
fn ai_wrong_size_and_format_are_errors() {
    for wrong_size in [true, false] {
        let p = FakeProvider { wrong_size, wrong_format: !wrong_size, ..Default::default() };
        let ctx = JobCtx::new();
        assert!(run_to_completion(&p, &request(PixelFormat::RGBA8), &ctx, Duration::ZERO).is_err());
        assert_ne!(ctx.fraction(), 1.0);
    }
}

#[test]
fn ai_no_provider_message_and_session_round_trip() {
    for mut s in [Session::new(), Session::default()] {
        let e = s.ai_provider().ok_or(AiError::NoProvider).err().unwrap();
        assert_eq!(e.to_string(), "Craft AI Manager isn't running");
        assert_eq!(EngineError::from(e).to_string(), "Craft AI Manager isn't running");
        let p: Arc<dyn AiProvider> = Arc::new(FakeProvider::default());
        s.set_ai_provider(Some(p.clone()));
        assert!(Arc::ptr_eq(&p, &s.ai_provider().unwrap()));
        s.set_ai_provider(None);
        assert!(s.ai_provider().is_none());
    }
}

#[test]
fn ai_wire_task_and_opaque_id_round_trip() {
    assert_eq!(AiTask::FillMaskedArea.id(), "fill_masked_area");
    assert_eq!(serde_json::to_string(&AiTask::FillMaskedArea).unwrap(), "\"fill_masked_area\"");
    let id = AiJobId::new("provider/opaque:42");
    assert_eq!(serde_json::from_str::<AiJobId>(&serde_json::to_string(&id).unwrap()).unwrap(), id);
}

#[test]
fn ai_invalid_requests_fail_before_submit() {
    let base = request(PixelFormat::RGBA8);
    let mut cases = Vec::new();
    let mut r = base.clone();
    r.window = Rect::EMPTY;
    cases.push(r);
    let mut r = base.clone();
    r.window = Rect::new(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    cases.push(r);
    let mut r = base.clone();
    r.pixels.pop();
    cases.push(r);
    let mut r = base.clone();
    r.coverage.pop();
    cases.push(r);
    for coverage in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        let mut r = base.clone();
        r.coverage[0] = coverage;
        cases.push(r);
    }
    for req in cases {
        let p = FakeProvider::default();
        assert!(run_to_completion(&p, &req, &JobCtx::new(), Duration::ZERO).is_err());
        assert!(p.state.lock().unwrap().request.is_none());
    }
}

#[test]
fn ai_wrong_window_is_an_error() {
    let p = FakeProvider { wrong_window: true, ..Default::default() };
    assert!(run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).is_err());
}

#[test]
fn ai_poll_reports_intermediate_progress() {
    let ctx = JobCtx::new();
    let p = FakeProvider { polls: 4, never_finish: true, cancel_at: Some((3, ctx.clone())), ..Default::default() };
    assert_eq!(run_to_completion(&p, &request(PixelFormat::RGBA8), &ctx, Duration::ZERO), Err(AiError::Cancelled));
    assert_eq!(ctx.fraction(), 0.5);
    assert_eq!(ctx.message(), "Filling");
}

#[test]
fn ai_provider_call_errors_pass_through() {
    for call_error in ["submit", "poll", "result"] {
        let p = FakeProvider { call_error: Some(call_error), ..Default::default() };
        let e = run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).unwrap_err();
        assert_eq!(e.to_string(), "The provider call failed");
    }
}

#[test]
fn ai_cancellation_during_failed_poll_still_calls_cancel() {
    let ctx = JobCtx::new();
    let p = FakeProvider { call_error: Some("poll"), cancel_at: Some((1, ctx.clone())), ..Default::default() };
    assert_eq!(run_to_completion(&p, &request(PixelFormat::RGBA8), &ctx, Duration::ZERO), Err(AiError::Cancelled));
    assert!(p.state.lock().unwrap().cancelled);
}

/// BF-02 P3: a manager busy writing a result can miss a poll; two misses in a row are survived.
#[test]
fn ai_two_poll_errors_in_a_row_are_tolerated() {
    let p = FakeProvider { poll_errors: POLL_ERRORS_TOLERATED, polls: 3, ..Default::default() };
    let r = run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO);
    assert!(r.is_ok(), "{r:?}");
    let s = p.state.lock().unwrap();
    assert_eq!(s.polls, POLL_ERRORS_TOLERATED + 2, "two failed polls, one running, one done");
    assert_eq!(s.cancels, 0, "a job that finished is not cancelled");
}

/// BF-02 P3: the third poll error in a row fails the job, and the provider is told to drop it.
#[test]
fn ai_a_third_poll_error_in_a_row_fails_and_cancels_the_job() {
    let p = FakeProvider { poll_errors: POLL_ERRORS_TOLERATED + 1, polls: 1, ..Default::default() };
    let e = run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).unwrap_err();
    assert_eq!(e.to_string(), "The provider call failed");
    let s = p.state.lock().unwrap();
    assert_eq!(s.polls, POLL_ERRORS_TOLERATED + 1);
    assert_eq!(s.cancels, 1, "the job was cancelled on the provider after the poll errors");
}

/// BF-02 P3: every failure after a successful submit cancels the job on the provider (best effort,
/// even when that cancel fails), so failed jobs do not fill its queue. A failed submit has no job.
#[test]
fn ai_every_error_after_submit_cancels_the_job_on_the_provider() {
    let cases: Vec<(&str, FakeProvider)> = vec![
        ("poll error", FakeProvider { call_error: Some("poll"), ..Default::default() }),
        ("result error", FakeProvider { call_error: Some("result"), ..Default::default() }),
        ("failed job", FakeProvider { fail: Some("The model ran out of memory".into()), ..Default::default() }),
        ("wrong size", FakeProvider { wrong_size: true, ..Default::default() }),
        ("wrong window", FakeProvider { wrong_window: true, ..Default::default() }),
        ("invalid pixels in the result", FakeProvider { bad_pixel: true, ..Default::default() }),
        ("cancel fails too", FakeProvider { call_error: Some("result"), cancel_fails: true, ..Default::default() }),
    ];
    for (what, p) in cases {
        assert!(run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).is_err(), "{what}");
        assert_eq!(p.state.lock().unwrap().cancels, 1, "{what}: the job must be cancelled exactly once");
    }
    let p = FakeProvider { call_error: Some("submit"), ..Default::default() };
    assert!(run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).is_err());
    assert_eq!(p.state.lock().unwrap().cancels, 0, "no job was accepted, so there is nothing to cancel");
}

/// BF-02 P7: the provider's progress fills what is left of the job's bar, from where the caller
/// left it, and never moves backwards even when the provider's own number does.
#[test]
fn ai_provider_progress_continues_from_the_job_s_progress_and_never_goes_back() {
    let ctx = JobCtx::new();
    ctx.progress(0.1, "Waiting");
    let p = FakeProvider { polls: 5, progress: vec![0.0, 0.5, 0.2, 0.8, 0.8], observe: Some(ctx.clone()), ..Default::default() };
    run_to_completion(&p, &request(PixelFormat::RGBA8), &ctx, Duration::ZERO).unwrap();
    let mut seen = p.state.lock().unwrap().seen.clone();
    seen.push(ctx.fraction());
    assert!(seen.windows(2).all(|w| w[1] >= w[0]), "progress went backwards: {seen:?}");
    // Provider 0.5 is half of the remaining 0.9, not a raw 0.5.
    assert!((seen[2] - 0.55).abs() < 1e-6, "{seen:?}");
    assert!((seen[4] - 0.82).abs() < 1e-6, "{seen:?}");
    assert_eq!(ctx.fraction(), 1.0);
}

/// BF-02 P7: a failed job with no message still gives the user a sentence.
#[test]
fn ai_a_failed_job_without_a_message_gives_a_sentence() {
    for message in ["", "   "] {
        let p = FakeProvider { fail: Some(message.into()), ..Default::default() };
        let e = run_to_completion(&p, &request(PixelFormat::RGBA8), &JobCtx::new(), Duration::ZERO).unwrap_err();
        assert_eq!(e.to_string(), "The AI job failed without saying why.", "{message:?}");
    }
}

/// BF-02 P1: the window is the selection's bounds plus max(32 px, 25% of the longer side),
/// clamped to the canvas.
#[test]
fn ai_fill_window_is_the_selection_bounds_plus_padding() {
    let canvas = Rect::new(0, 0, 4000, 3000);
    // Small box: 32 px of padding.
    assert_eq!(fill_window(Rect::new(100, 200, 140, 220), canvas), Rect::new(68, 168, 172, 252));
    // Big box: 25% of its longer side (400 -> 100), rounded up for odd sizes (401 -> 101).
    assert_eq!(fill_window(Rect::new(1000, 1000, 1400, 1100), canvas), Rect::new(900, 900, 1500, 1200));
    assert_eq!(fill_window(Rect::new(1000, 1000, 1100, 1401), canvas), Rect::new(899, 899, 1201, 1502));
    // Clamped to the canvas at the edges.
    assert_eq!(fill_window(Rect::new(0, 0, 10, 10), canvas), Rect::new(0, 0, 42, 42));
    assert_eq!(fill_window(Rect::new(3990, 2990, 4000, 3000), canvas), Rect::new(3958, 2958, 4000, 3000));
    // Never Content-Aware Fill's square of 4 * sqrt(area): a 2000 x 2000 selection on a 12000 x 8000
    // image is a 3000 x 3000 window (9 MP), not one over 64 MP.
    let big = fill_window(Rect::new(5000, 3000, 7000, 5000), Rect::new(0, 0, 12000, 8000));
    assert_eq!(big, Rect::new(4500, 2500, 7500, 5500));
    assert!(check_window_pixels(big, DEFAULT_MAX_PIXELS).is_ok());
    // Extreme coordinates saturate instead of overflowing.
    let huge = Rect::new(i32::MIN, i32::MIN, i32::MAX, i32::MAX);
    assert_eq!(fill_window(huge, huge), huge);
}

/// BF-02 P2: a window over the limit is refused with a sentence; the default is 64 MP.
#[test]
fn ai_a_window_over_max_pixels_is_refused_with_a_sentence() {
    assert_eq!(DEFAULT_MAX_PIXELS, 64_000_000);
    assert!(check_window_pixels(Rect::new(0, 0, 8000, 8000), DEFAULT_MAX_PIXELS).is_ok(), "exactly 64 MP is allowed");
    let e = check_window_pixels(Rect::new(0, 0, 8001, 8000), DEFAULT_MAX_PIXELS).unwrap_err();
    assert_eq!(e.to_string(), "The area to fill is 64.1 megapixels, over the 64 megapixels Craft AI Manager accepts; select a smaller area.");
    let e = check_window_pixels(Rect::new(0, 0, 12000, 8000), 4_000_000).unwrap_err();
    assert!(e.to_string().starts_with("The area to fill is 96.0 megapixels, over the 4 megapixels"), "{e}");
    // A limit that is not a whole number of megapixels is named as it is: 1.5, not "1".
    let e = check_window_pixels(Rect::new(0, 0, 2000, 1000), 1_500_000).unwrap_err();
    assert_eq!(e.to_string(), "The area to fill is 2.0 megapixels, over the 1.5 megapixels Craft AI Manager accepts; select a smaller area.");
    // Still rounded down, so the sentence never claims a limit the manager does not have.
    let e = check_window_pixels(Rect::new(0, 0, 2000, 1000), 1_599_999).unwrap_err();
    assert!(e.to_string().contains("over the 1.5 megapixels Craft AI Manager accepts"), "{e}");
    // A whole number of megapixels keeps no decimal point.
    let e = check_window_pixels(Rect::new(0, 0, 2000, 1000), 1_000_000).unwrap_err();
    assert!(e.to_string().contains("over the 1 megapixels Craft AI Manager accepts"), "{e}");
    // A provider that advertises nothing: the trait's default.
    assert_eq!(FakeProvider::default().max_pixels(AiTask::FillMaskedArea), Ok(None));
}
