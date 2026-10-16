//! PC-05: Edit … Generative Fill over MCP, with a stand-in for Craft AI Manager.
//!
//! `photocraft-engine`'s own `FakeProvider` is private to that crate, so this file carries its
//! own: it fills wherever the selection covers, and either finishes after a couple of polls or
//! never finishes at all, which is how the cancel path gets a provider that is still working.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use photocraft_automation::{Backend, Headless, PhotocraftMcp};
use photocraft_engine::ai::{AiError, AiJobId, AiJobState, AiProvider, AiRequest, AiResult, AiTask};
use rmcp::model::{CallToolRequestParams, CallToolResult, ClientConfig};
use rmcp::service::RunningService;
use rmcp::{ClientHandler, RoleClient, ServiceExt};
use serde_json::{Value, json};

#[derive(Clone, Default)]
struct Client;
impl ClientHandler for Client {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::default()
    }
}

/// A stand-in for Craft AI Manager (SPEC amendment 16): the fill comes back at full strength
/// wherever coverage > 0 and the input is returned bit for bit elsewhere.
#[derive(Default)]
struct FakeFill {
    /// Keep every poll `Running`, so a cancel is the only way the job ends.
    never_finish: bool,
    polls: AtomicUsize,
    /// How many times the engine asked the provider to cancel its job.
    cancels: AtomicUsize,
    cancelled: AtomicBool,
    submitted: Mutex<Option<AiRequest>>,
}

impl AiProvider for FakeFill {
    fn tasks(&self) -> Result<Vec<AiTask>, AiError> {
        Ok(vec![AiTask::FillMaskedArea])
    }
    fn submit(&self, req: &AiRequest) -> Result<AiJobId, AiError> {
        req.validate()?;
        *self.submitted.lock().unwrap_or_else(PoisonError::into_inner) = Some(req.clone());
        self.polls.store(0, Ordering::Relaxed);
        self.cancelled.store(false, Ordering::Relaxed);
        Ok(AiJobId::new("fake-fill"))
    }
    fn poll(&self, id: &AiJobId) -> Result<AiJobState, AiError> {
        assert_eq!(id.as_str(), "fake-fill");
        let polls = self.polls.fetch_add(1, Ordering::Relaxed) + 1;
        if self.cancelled.load(Ordering::Relaxed) {
            return Ok(AiJobState::Cancelled);
        }
        if !self.never_finish && polls > 2 {
            return Ok(AiJobState::Done);
        }
        Ok(AiJobState::Running { progress: 0.5, message: "Filling".into() })
    }
    fn result(&self, id: &AiJobId) -> Result<AiResult, AiError> {
        assert_eq!(id.as_str(), "fake-fill");
        let held = self.submitted.lock().unwrap_or_else(PoisonError::into_inner);
        let req = held.as_ref().ok_or_else(|| AiError::Other("No job was submitted".into()))?;
        let mut pixels = req.pixels.clone();
        for (pixel, coverage) in pixels.chunks_exact_mut(req.format.channels()).zip(&req.coverage) {
            if *coverage > 0.0 {
                for sample in pixel.iter_mut().take(req.format.mode.color_channels()) {
                    *sample = 0.75;
                }
                if let Some(alpha) = pixel.get_mut(req.format.mode.color_channels()) {
                    *alpha = 1.0;
                }
            }
        }
        Ok(AiResult {
            window: req.window,
            pixels,
            format: req.format,
            model: "solid-fill".into(),
            device: "CPU".into(),
            provider: "local fake".into(),
            elapsed_ms: 12,
        })
    }
    fn cancel(&self, id: &AiJobId) -> Result<(), AiError> {
        assert_eq!(id.as_str(), "fake-fill");
        self.cancels.fetch_add(1, Ordering::Relaxed);
        self.cancelled.store(true, Ordering::Relaxed);
        Ok(())
    }
}

/// A headless MCP server whose session has `fill` as its AI provider.
fn mcp_with(fill: Arc<FakeFill>) -> PhotocraftMcp {
    let mut headless = Headless::new();
    headless.session.set_ai_provider(Some(fill));
    PhotocraftMcp::with_backend(Backend::Headless(Arc::new(Mutex::new(headless))))
}

async fn connect(server: PhotocraftMcp) -> RunningService<RoleClient, Client> {
    let (s, c) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        if let Ok(running) = server.serve(s).await {
            let _ = running.waiting().await;
        }
    });
    Client.serve(c).await.expect("client init")
}

async fn call(client: &RunningService<RoleClient, Client>, name: &str, args: Value) -> CallToolResult {
    let mut p = CallToolRequestParams::new(name.to_owned());
    if let Value::Object(m) = args {
        p = p.with_arguments(m);
    }
    client.call_tool(p).await.expect("call_tool transport")
}

fn text(r: &CallToolResult) -> String {
    r.content.iter().filter_map(|c| c.as_text()).map(|t| t.text.clone()).collect::<Vec<_>>().join("\n")
}

fn json_of(r: &CallToolResult) -> Value {
    assert_ne!(r.is_error, Some(true), "tool error: {}", text(r));
    serde_json::from_str(&text(r)).unwrap_or_else(|e| panic!("not JSON ({e}): {}", text(r)))
}

/// A 64x48 RGB document with a selection around the middle, the same shape the engine's own
/// Generative Fill tests use.
async fn doc_with_selection(client: &RunningService<RoleClient, Client>) {
    json_of(&call(client, "doc_new", json!({"width": 64, "height": 48, "mode": "rgb", "depth": 8, "background": "white"})).await);
    json_of(&call(client, "command_run", json!({"id": "select.rect", "params": {"x": 26, "y": 16, "width": 12, "height": 14}})).await);
}

async fn inspect(client: &RunningService<RoleClient, Client>) -> Value {
    json_of(&call(client, "doc_inspect", json!({})).await)
}

/// The layer tree top-to-bottom, without the `selected` flag: which layer is selected is part of
/// the session, not of the document, and Generative Fill moves it to the new layer.
fn layers(doc: &Value) -> Vec<Value> {
    doc["layers"]
        .as_array()
        .expect("a layer tree")
        .iter()
        .map(|l| {
            let mut layer = l.clone();
            layer.as_object_mut().expect("a layer object").remove("selected");
            layer
        })
        .collect()
}

fn layer_names(doc: &Value) -> Vec<String> {
    layers(doc).iter().map(|l| l["name"].as_str().unwrap_or("").to_owned()).collect()
}

/// The rendered document as base64 PNG, so a test can say "not one pixel moved".
async fn preview(client: &RunningService<RoleClient, Client>) -> String {
    let r = call(client, "doc_render_preview", json!({"max_side": 64})).await;
    assert_ne!(r.is_error, Some(true), "{}", text(&r));
    let images: Vec<String> = r.content.iter().filter_map(|c| c.as_image()).map(|i| i.data.clone()).collect();
    assert_eq!(images.len(), 1, "one PNG preview: {images:?}");
    images.into_iter().next().expect("the preview checked above")
}

/// Start the fill without waiting, and hand back the job id.
async fn start_fill(client: &RunningService<RoleClient, Client>) -> u64 {
    let started = json_of(&call(client, "command_run", json!({"id": "edit.generativeFill", "wait": false})).await);
    started["job"].as_u64().unwrap_or_else(|| panic!("a job id, got {started}"))
}

async fn job(client: &RunningService<RoleClient, Client>, id: u64) -> Value {
    let listed = json_of(&call(client, "jobs_list", json!({})).await);
    listed["jobs"].as_array().expect("a job list").iter().find(|j| j["id"] == id).unwrap_or_else(|| panic!("job {id} is listed: {listed}")).clone()
}

/// Poll until the job is no longer running, and return its `jobs.list` entry.
async fn settled_job(client: &RunningService<RoleClient, Client>, id: u64) -> Value {
    let started = std::time::Instant::now();
    loop {
        let entry = job(client, id).await;
        if entry["state"] != "running" {
            return entry;
        }
        assert!(started.elapsed().as_secs() < 60, "job {id} never settled: {entry}");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

/// Wait for `count` of something the fake does, so a test never races the job worker.
async fn wait_on(counter: &AtomicUsize, count: usize) {
    let started = std::time::Instant::now();
    while counter.load(Ordering::Relaxed) < count {
        assert!(started.elapsed().as_secs() < 30, "the fake provider never reached {count}");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn generative_fill_over_mcp_adds_a_layer_and_one_undo_removes_it() {
    let client = connect(mcp_with(Arc::new(FakeFill::default()))).await;
    doc_with_selection(&client).await;
    let before = inspect(&client).await;
    assert_eq!(layer_names(&before), ["Background"], "the new document has one layer: {before}");
    let pixels_before = preview(&client).await;

    let id = start_fill(&client).await;
    let running = job(&client, id).await;
    assert_eq!(running["state"], "running", "the fill runs as a job: {running}");
    assert_eq!(running["label"], "Generative Fill", "the job is named as D1 decided: {running}");
    assert_eq!(running["command"], "edit.generativeFill", "{running}");

    let done = settled_job(&client, id).await;
    assert_eq!(done["state"], "done", "{done}");
    let result = &done["result"];
    assert!(result["filled"].as_u64().unwrap_or(0) > 0, "the fill covered pixels: {result}");
    assert_eq!(result["ai"]["model"], "solid-fill", "the provenance travels with the job: {result}");
    assert_eq!(result["ai"]["provider"], "local fake", "{result}");

    let after = inspect(&client).await;
    assert_eq!(layer_names(&after), ["Generative Fill", "Background"], "the fill landed on a new layer above the active one: {after}");
    let new = after["layers"][0]["id"].as_u64().expect("the new layer has an id");
    assert_eq!(result["layer"].as_u64(), Some(new), "the result names the new layer: {result}");
    assert_eq!(layers(&after)[1], layers(&before)[0], "the source layer is untouched");
    let pixels_after = preview(&client).await;
    assert_ne!(pixels_after, pixels_before, "the fill is visible in the document");

    // One undo removes the layer and nothing else.
    json_of(&call(&client, "command_run", json!({"id": "edit.undo"})).await);
    let back = inspect(&client).await;
    assert_eq!(layer_names(&back), layer_names(&before), "one undo step took the layer away: {back}");
    assert_eq!(layers(&back), layers(&before), "the document is back where it was: {back}");
    assert_eq!(preview(&client).await, pixels_before, "one undo undid every pixel of the fill");
    client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_a_generative_fill_over_mcp_leaves_the_document_unchanged() {
    let fill = Arc::new(FakeFill { never_finish: true, ..Default::default() });
    let client = connect(mcp_with(fill.clone())).await;
    doc_with_selection(&client).await;
    let before = inspect(&client).await;
    let pixels_before = preview(&client).await;

    let id = start_fill(&client).await;
    // Wait until the provider has the job, so this cancels a fill that is really still running.
    wait_on(&fill.polls, 1).await;
    assert_eq!(job(&client, id).await["state"], "running", "the fake provider is still working: {id}");

    let cancelled = json_of(&call(&client, "jobs_cancel", json!({"job": id})).await);
    assert_eq!(cancelled["cancelled"], 1, "{cancelled}");
    let entry = settled_job(&client, id).await;
    assert_eq!(entry["state"], "cancelled", "the job reports cancelled: {entry}");
    wait_on(&fill.cancels, 1).await;

    let after = inspect(&client).await;
    assert_eq!(layer_names(&after), layer_names(&before), "no Generative Fill layer appeared: {after}");
    assert_eq!(layers(&after), layers(&before), "the document is unchanged: {after}");
    assert_eq!(preview(&client).await, pixels_before, "not one pixel moved");
    client.cancel().await.unwrap();
}
