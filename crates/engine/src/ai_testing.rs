//! The shared AI test provider: it fills the request with a solid colour and can be told to
//! fail, stall, return bad data or be cancelled. Like Craft AI Manager (SPEC amendment 16) it
//! returns the fill at full strength wherever coverage > 0 (alpha made opaque too) and the input
//! bit for bit elsewhere; blending by coverage is PhotoCraft's job, done once when the layer is
//! composited. `ai::tests` and the commands that call a provider (Generative Fill) both use it.

use std::sync::{Mutex, PoisonError};

use super::*;
use crate::jobs::JobCtx;

#[derive(Default)]
pub(crate) struct FakeState {
    pub(crate) request: Option<AiRequest>,
    pub(crate) polls: usize,
    pub(crate) cancelled: bool,
    /// How many times `cancel` was called.
    pub(crate) cancels: usize,
    /// `observe`'s progress fraction at the start of every poll.
    pub(crate) seen: Vec<f32>,
    /// How many times `max_pixels` was asked.
    pub(crate) limit_asks: usize,
}

#[derive(Default)]
pub(crate) struct FakeProvider {
    pub(crate) state: Mutex<FakeState>,
    pub(crate) polls: usize,
    pub(crate) fail: Option<String>,
    pub(crate) never_finish: bool,
    pub(crate) wrong_size: bool,
    pub(crate) wrong_format: bool,
    pub(crate) wrong_window: bool,
    pub(crate) remote_cancelled: bool,
    pub(crate) cancel_at: Option<(usize, JobCtx)>,
    pub(crate) cancel_fails: bool,
    pub(crate) call_error: Option<&'static str>,
    /// Put a NaN sample in the result: provider pixels are outside our input, so callers check.
    pub(crate) bad_pixel: bool,
    /// The first this many polls fail with a call error (a transient blip).
    pub(crate) poll_errors: usize,
    /// Progress to report on poll 1, 2, …, instead of polls / `polls`.
    pub(crate) progress: Vec<f32>,
    /// Record this job's progress fraction at every poll into `FakeState::seen`.
    pub(crate) observe: Option<JobCtx>,
    /// What `max_pixels` answers: `Ok(None)` (not advertised) unless set.
    pub(crate) max_pixels: Option<Result<Option<u64>, AiError>>,
    /// Offer `subject_mask` in `tasks()` and answer a mask request with `fake_mask`, so the AI
    /// Select Subject and Remove Background tests have a provider that can do the job.
    pub(crate) offer_mask: bool,
}

impl AiProvider for FakeProvider {
    fn tasks(&self) -> Result<Vec<AiTask>, AiError> {
        let mut tasks = vec![AiTask::FillMaskedArea];
        if self.offer_mask {
            tasks.push(AiTask::SubjectMask);
        }
        Ok(tasks)
    }
    fn submit(&self, req: &AiRequest) -> Result<AiJobId, AiError> {
        if self.call_error == Some("submit") {
            return Err(AiError::Other("The provider call failed".into()));
        }
        req.validate()?;
        let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let limit_asks = s.limit_asks;
        *s = FakeState { request: Some(req.clone()), limit_asks, ..Default::default() };
        Ok(AiJobId::new("fake-job"))
    }
    fn poll(&self, id: &AiJobId) -> Result<AiJobState, AiError> {
        assert_eq!(id.as_str(), "fake-job");
        let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        s.polls += 1;
        if let Some(ctx) = &self.observe {
            s.seen.push(ctx.fraction());
        }
        if let Some((n, ctx)) = &self.cancel_at
            && s.polls == *n
        {
            ctx.cancel();
        }
        if self.call_error == Some("poll") || s.polls <= self.poll_errors {
            return Err(AiError::Other("The provider call failed".into()));
        }
        if let Some(message) = &self.fail {
            return Ok(AiJobState::Failed(message.clone()));
        }
        if s.cancelled || self.remote_cancelled {
            return Ok(AiJobState::Cancelled);
        }
        if !self.never_finish && s.polls > self.polls {
            return Ok(AiJobState::Done);
        }
        let progress = self.progress.get(s.polls - 1).copied().unwrap_or(s.polls as f32 / self.polls.max(1) as f32);
        if s.polls == 1 {
            Ok(AiJobState::Queued { progress, message: "Queued".into() })
        } else {
            Ok(AiJobState::Running { progress, message: "Filling".into() })
        }
    }
    fn result(&self, id: &AiJobId) -> Result<AiResult, AiError> {
        if self.call_error == Some("result") {
            return Err(AiError::Other("The provider call failed".into()));
        }
        assert_eq!(id.as_str(), "fake-job");
        let s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let req = s.request.as_ref().ok_or_else(|| AiError::Other("No job was submitted".into()))?;
        let output = if req.task == AiTask::SubjectMask {
            // A soft-edged disc at the size the request actually sent (amendment 18), so a test can
            // compare the upscaled selection against the same shape.
            let size = req.sent_image_size();
            let mut coverage = fake_mask(size);
            if self.bad_pixel
                && let Some(v) = coverage.first_mut()
            {
                *v = f32::NAN;
            }
            if self.wrong_size {
                coverage.pop();
            }
            AiOutput::Mask { coverage, size }
        } else {
        let mut pixels = req.pixels.clone();
        // Solid 0.75 in every colour channel at full strength wherever the mask is > 0 (amendment
        // 16); the unselected pixels are the input as it came. Alpha has no fill to blend towards,
        // so a covered pixel becomes opaque instead - Craft AI Manager's fake backend does the same.
        for (pixel, coverage) in pixels.chunks_exact_mut(req.format.channels()).zip(&req.coverage) {
            if *coverage > 0.0 {
                for sample in pixel.iter_mut().take(req.format.mode.color_channels()) {
                    *sample = 0.75;
                }
                // The sample after the colour channels, when the format has one.
                if let Some(alpha) = pixel.get_mut(req.format.mode.color_channels()) {
                    *alpha = 1.0;
                }
            }
        }
        if self.bad_pixel
            && let Some(sample) = pixels.first_mut()
        {
            *sample = f32::NAN;
        }
        if self.wrong_size {
            pixels.pop();
        }
        AiOutput::Image { pixels, format: if self.wrong_format { PixelFormat::GRAY8 } else { req.format } }
        }
        ;
        Ok(AiResult {
            window: if self.wrong_window { req.window.translate(1, 0) } else { req.window },
            output,
            model: "solid-fill".into(),
            device: "CPU".into(),
            provider: "local fake".into(),
            elapsed_ms: 12,
            // Amendment 18's extra provenance, so the commands have something to pass through.
            provenance: AiProvenance {
                model_version: Some("epoch_244".into()),
                model_sha256: Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c".into()),
                device_requested: Some("auto".into()),
                device_name: Some("CPU".into()),
                fallback_reason: None,
                diagnostics: None,
                load_ms: Some(5_124),
                run_ms: Some(7_624),
            },
        })
    }
    fn cancel(&self, id: &AiJobId) -> Result<(), AiError> {
        assert_eq!(id.as_str(), "fake-job");
        let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        s.cancelled = true;
        s.cancels += 1;
        drop(s);
        if self.cancel_fails { Err(AiError::Other("Cancellation could not reach the provider".into())) } else { Ok(()) }
    }
    fn max_pixels(&self, task: AiTask) -> Result<Option<u64>, AiError> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).limit_asks += 1;
        self.max_pixels.clone().unwrap_or(Ok(None))
    }
}

pub(crate) fn request(format: PixelFormat) -> AiRequest {
    AiRequest {
        task: AiTask::FillMaskedArea,
        window: Rect::new(-2, 3, 1, 4),
        sent_size: None,
        pixels: vec![0.25; 3 * format.channels()],
        format,
        coverage: vec![0.0, 0.5, 1.0],
        prompt: None,
        client_version: "photocraft/test".into(),
    }
}

/// The fake subject mask: a soft-edged disc centred on `size`, one value in 0..=1 per pixel. It is
/// non-zero at the middle of every edge (so the selection's bounding box is the whole area) and
/// zero in the corners.
pub(crate) fn fake_mask(size: Size) -> Vec<f32> {
    let (w, h) = (size.width.max(1) as f32, size.height.max(1) as f32);
    let mut out = Vec::with_capacity(size.width as usize * size.height as usize);
    for y in 0..size.height {
        for x in 0..size.width {
            let (u, v) = ((x as f32 + 0.5) / w - 0.5, (y as f32 + 0.5) / h - 0.5);
            let r = (u * u + v * v).sqrt();
            out.push(((0.6 - r) / 0.2).clamp(0.0, 1.0));
        }
    }
    out
}
