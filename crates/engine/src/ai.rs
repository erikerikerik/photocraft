//! Backend-neutral AI jobs. Providers run blocking calls on a job worker; no network lives here.
use std::time::Duration;

use photocraft_color::PixelFormat;
use photocraft_geom::{Rect, Size};
use serde::{Deserialize, Serialize};

/// Stable task identifiers shared with providers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiTask {
    #[serde(rename = "fill_masked_area")]
    FillMaskedArea,
    #[serde(rename = "subject_mask")]
    SubjectMask,
}

impl AiTask {
    pub const fn id(self) -> &'static str {
        match self {
            Self::FillMaskedArea => "fill_masked_area",
            Self::SubjectMask => "subject_mask",
        }
    }
}

/// A row-major window in document coordinates. Samples are interleaved, straight-alpha
/// f32 in the document's colour space; PixelFormat retains the original storage depth.
/// Coverage has one finite value in 0..=1 per pixel. No colour conversion is implied.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiRequest {
    pub task: AiTask,
    pub window: Rect,
    /// The size actually sent, when it is smaller than `window` (amendment 18: a mask task
    /// downscales before sending). `None` means the window's own size, which is what a fill sends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_size: Option<Size>,
    pub pixels: Vec<f32>,
    pub format: PixelFormat,
    pub coverage: Vec<f32>,
    pub prompt: Option<String>,
    pub client_version: String,
}

/// Provider-owned identifier: callers must not interpret its contents.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AiJobId(String);

impl AiJobId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Pending states carry progress and a plain status message. Done implies progress 1.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AiJobState {
    Queued { progress: f32, message: String },
    Running { progress: f32, message: String },
    Done,
    Failed(String),
    Cancelled,
}

/// What a provider produced: pixels for the requested window, or a mask at the size it received.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AiOutput {
    Image {
        pixels: Vec<f32>,
        format: PixelFormat,
    },
    /// One finite value in 0..=1 per pixel, row-major, at `size`: the size the provider received,
    /// which for a mask task is the downscaled image, not the document area.
    Mask {
        coverage: Vec<f32>,
        size: Size,
    },
}

/// Where the work ran, as the provider reported it (amendment 18). Every field is optional:
/// missing means unknown, which is never an error and never a reason to refuse a result.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_requested: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_ms: Option<u64>,
}

/// The job's output, plus execution provenance (B08).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiResult {
    pub window: Rect,
    pub output: AiOutput,
    pub model: String,
    pub device: String,
    /// Identifies where the job ran, including local/cloud provider identity.
    pub provider: String,
    pub elapsed_ms: u64,
    /// The extra provenance fields (amendment 18); all empty when the provider says nothing more.
    #[serde(default)]
    pub provenance: AiProvenance,
}

/// What a 401 from the manager means: the token in the endpoint file is not its current one, which
/// is what happens when the manager restarted. The sentence lives here so a caller can act on
/// [`AiError::TokenMismatch`] without copying it.
pub const TOKEN_MISMATCH: &str = "Craft AI Manager refused the request (token mismatch); restart Craft AI Manager.";

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("Craft AI Manager isn't running")]
    NoProvider,
    /// The manager refused the token (401). Its own kind, so a caller can tell "restart the
    /// manager" from a blip it is worth working around.
    #[error("{}", TOKEN_MISMATCH)]
    TokenMismatch,
    #[error("The AI job was cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

impl From<AiError> for crate::EngineError {
    fn from(error: AiError) -> Self {
        match error {
            AiError::Cancelled => Self::Cancelled,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Blocking, object-safe backend contract. Implementations validate their own inputs and
/// return errors for unknown jobs; calls must not panic. Cancellation is best-effort.
pub trait AiProvider: Send + Sync {
    fn tasks(&self) -> Result<Vec<AiTask>, AiError>;
    fn submit(&self, req: &AiRequest) -> Result<AiJobId, AiError>;
    fn poll(&self, id: &AiJobId) -> Result<AiJobState, AiError>;
    fn result(&self, id: &AiJobId) -> Result<AiResult, AiError>;
    fn cancel(&self, id: &AiJobId) -> Result<(), AiError>;
    /// The largest window (in pixels) the provider accepts for `task`, when it says so cheaply.
    /// `Ok(None)` means "not advertised": callers use [`DEFAULT_MAX_PIXELS`]. Called on a job
    /// worker before any pixels are read, so an unreachable provider fails before the slow part.
    fn max_pixels(&self, task: AiTask) -> Result<Option<u64>, AiError> {
        let _ = task;
        Ok(None)
    }
}

/// The window limit when a provider does not advertise one: Craft AI Manager's `max_pixels`.
pub const DEFAULT_MAX_PIXELS: u64 = 64_000_000;

/// How many poll errors in a row a job survives: a manager busy writing a result can miss one
/// read, and that should not throw the job away (the third in a row fails it).
pub const POLL_ERRORS_TOLERATED: usize = 2;

/// Refuse a window over `max_pixels` with a sentence, before anything is read or encoded.
pub fn check_window_pixels(window: Rect, max_pixels: u64) -> Result<(), AiError> {
    let pixels = window.size().area();
    if pixels > max_pixels {
        let (size, limit) = if max_pixels >= 1_000_000 {
            // Round the size up and the limit down, so the sentence never reads "64.0 over 64". A
            // limit that is not a whole number of megapixels keeps its tenth: 1.5, never 1.
            let size = (pixels as f64 / 100_000.0).ceil() / 10.0;
            let limit = (max_pixels as f64 / 100_000.0).floor() / 10.0;
            let limit = if limit.fract() == 0.0 { format!("{limit:.0} megapixels") } else { format!("{limit:.1} megapixels") };
            (format!("{size:.1} megapixels"), limit)
        } else {
            (format!("{pixels} pixels"), format!("{max_pixels} pixels"))
        };
        return Err(AiError::Other(format!("The area to fill is {size}, over the {limit} Craft AI Manager accepts; select a smaller area.")));
    }
    Ok(())
}

/// The window an AI fill sends (SPEC amendment 16): the selection's bounding box `hb` plus
/// padding of max(32 px, 25% of the box's longer side), clamped to the `canvas`. This is not
/// Content-Aware Fill's sampling window, which grows with the square root of the area.
pub fn fill_window(hb: Rect, canvas: Rect) -> Rect {
    let longer = hb.width().max(hb.height());
    let pad = (longer.saturating_add(3) / 4).max(32);
    let pad = i32::try_from(pad).unwrap_or(i32::MAX);
    hb.inflate(pad).intersect(&canvas)
}

fn sample_count(size: Size, format: PixelFormat) -> Result<(usize, usize), AiError> {
    let invalid = || AiError::Other("The AI window is empty or too large".into());
    if size.is_empty() {
        return Err(invalid());
    }
    let pixels = usize::try_from(size.area()).map_err(|_| invalid())?;
    let samples = pixels.checked_mul(format.channels()).filter(|n| *n <= isize::MAX as usize / size_of::<f32>()).ok_or_else(invalid)?;
    Ok((pixels, samples))
}

impl AiRequest {
    /// The size the provider actually receives: `sent_size`, or the window's own size when unset.
    pub fn sent_image_size(&self) -> Size {
        self.sent_size.unwrap_or_else(|| self.window.size())
    }

    /// Check geometry and buffer layout before a provider receives the request. The buffers are
    /// checked against [`AiRequest::sent_image_size`], which is the window's own size for a
    /// fill and the downscaled size a mask task sends (amendment 18).
    pub fn validate(&self) -> Result<(), AiError> {
        let sent = self.sent_image_size();
        let (pixels, samples) = sample_count(sent, self.format)?;
        if self.pixels.len() != samples || self.coverage.len() != pixels {
            return Err(AiError::Other("The AI image or selection size does not match its window".into()));
        }
        if self.coverage.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
            return Err(AiError::Other("The AI selection coverage must be between zero and one".into()));
        }
        Ok(())
    }
}

impl AiResult {
    /// Check a provider's result against the request it answers: the same window, the kind of
    /// output that task asked for, the right number of samples, and only finite values. A mask
    /// comes back at the size the provider received (`sent_image_size`), not the document area.
    pub fn check_against(&self, req: &AiRequest) -> Result<(), AiError> {
        let mismatch = || AiError::Other("The AI result size or format does not match the requested window".into());
        let invalid = || AiError::Other("The AI result contains invalid pixel values".into());
        if self.window != req.window {
            return Err(mismatch());
        }
        match (&self.output, req.task) {
            (AiOutput::Image { pixels, format }, AiTask::FillMaskedArea) => {
                if *format != req.format || pixels.len() != req.pixels.len() {
                    return Err(mismatch());
                }
                if pixels.iter().any(|v| !v.is_finite()) {
                    return Err(invalid());
                }
            }
            (AiOutput::Mask { coverage, size }, AiTask::SubjectMask) => {
                let want = req.sent_image_size();
                let Ok(n) = usize::try_from(want.area()) else {
                    return Err(mismatch());
                };
                if *size != want || coverage.len() != n {
                    return Err(mismatch());
                }
                if coverage.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
                    return Err(invalid());
                }
            }
            // A mask for a fill (or the other way round) is not an answer to this request.
            _ => return Err(mismatch()),
        }
        Ok(())
    }
}

/// Submit and wait on a worker thread. Zero poll interval supports deterministic fast tests.
/// Native waits check cancellation at least every 10 ms. On wasm pending jobs return an
/// unsupported error (and request cancellation), since blocking polling cannot yield to JS.
///
/// Once the provider has the job, every way of failing (poll errors past
/// [`POLL_ERRORS_TOLERATED`], a failed job, a result that fails [`AiResult::check_against`] (its
/// window, the kind of output, its size, or non-finite values)) also cancels it there (best
/// effort), so failed jobs do not pile up in the provider's queue. The provider's progress is mapped onto what is left of the job's progress, so
/// the bar never stalls or jumps back.
pub fn run_to_completion(provider: &dyn AiProvider, req: &AiRequest, ctx: &crate::jobs::JobCtx, poll_every: Duration) -> Result<AiResult, AiError> {
    if ctx.cancelled() {
        return Err(AiError::Cancelled);
    }
    req.validate()?;
    let id = provider.submit(req)?;
    let outcome = wait_for(provider, req, &id, ctx, poll_every);
    // A cancel (ours, or the provider's own) has already been handled; anything else after the
    // submit leaves a job behind unless it is cancelled here.
    if let Err(error) = &outcome
        && *error != AiError::Cancelled
    {
        let _ = provider.cancel(&id);
    }
    outcome
}

fn wait_for(provider: &dyn AiProvider, req: &AiRequest, id: &AiJobId, ctx: &crate::jobs::JobCtx, poll_every: Duration) -> Result<AiResult, AiError> {
    let start = ctx.fraction();
    let mut poll_errors = 0usize;
    loop {
        check_cancel(provider, id, ctx)?;
        let state = provider.poll(id);
        check_cancel(provider, id, ctx)?;
        let state = match state {
            Ok(state) => {
                poll_errors = 0;
                Some(state)
            }
            Err(error) => {
                poll_errors += 1;
                if poll_errors > POLL_ERRORS_TOLERATED {
                    return Err(error);
                }
                None
            }
        };
        match state {
            None => {}
            Some(AiJobState::Queued { progress, message } | AiJobState::Running { progress, message }) => {
                let progress = if progress.is_finite() { progress.clamp(0.0, 1.0) } else { 0.0 };
                ctx.progress(start + (1.0 - start) * progress, &message);
            }
            Some(AiJobState::Done) => {
                let result = provider.result(id);
                check_cancel(provider, id, ctx)?;
                let result = result?;
                // Provider output is outside our input: the wrong kind of result, the wrong size,
                // or one NaN or inf would silently corrupt the document, so the whole result is
                // rejected here, where the caller still cancels the job.
                result.check_against(req)?;
                ctx.progress(1.0, "AI job complete");
                return Ok(result);
            }
            Some(AiJobState::Failed(message)) if message.trim().is_empty() => {
                return Err(AiError::Other("The AI job failed without saying why.".into()));
            }
            Some(AiJobState::Failed(message)) => return Err(AiError::Other(message)),
            Some(AiJobState::Cancelled) => return Err(AiError::Cancelled),
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut remaining = poll_every;
            while !remaining.is_zero() {
                check_cancel(provider, id, ctx)?;
                let step = remaining.min(Duration::from_millis(10));
                std::thread::sleep(step);
                remaining = remaining.saturating_sub(step);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = poll_every;
            return Err(AiError::Other("Waiting for an AI provider is not available in the web build".into()));
        }
    }
}

fn check_cancel(provider: &dyn AiProvider, id: &AiJobId, ctx: &crate::jobs::JobCtx) -> Result<(), AiError> {
    if ctx.cancelled() {
        // Even if the remote cancellation fails, discard the result locally.
        let _ = provider.cancel(id);
        return Err(AiError::Cancelled);
    }
    Ok(())
}

#[cfg(test)]
#[path = "ai_testing.rs"]
pub(crate) mod testing;
#[cfg(test)]
#[path = "ai_tests.rs"]
mod tests;
