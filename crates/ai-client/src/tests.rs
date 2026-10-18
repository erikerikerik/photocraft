//! PC-03 tests: the client against an in-process fake Craft AI Manager (SPEC amendment 11).
//! Every test uses its own endpoint file, so they run in parallel without touching the environment.

use std::io::{Cursor, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use photocraft_color::{ColorMode, PixelFormat, SampleType};
use photocraft_engine::ai::{AiError, AiJobId, AiJobState, AiOutput, AiProvenance, AiProvider, AiRequest, AiResult, AiTask, TOKEN_MISMATCH};
use photocraft_geom::{Rect, Size};

use crate::endpoint::Endpoint;
use crate::fake_server::{FakeServer, response};
use crate::provider::CaimProvider;
use crate::{http, pixels, protocol};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// A temp endpoint file that deletes itself, so no test leaves state behind.
struct TempEndpoint {
    path: PathBuf,
}

impl TempEndpoint {
    fn new(text: &str) -> Self {
        let endpoint = Self { path: unique_path() };
        endpoint.set(text);
        endpoint
    }

    fn missing() -> Self {
        Self { path: unique_path() }
    }

    fn set(&self, text: &str) {
        // BF-04: the client refuses an endpoint file other users can read, so this writes it the way
        // Craft AI Manager does - owner-only. Windows has no mode bits to get wrong here.
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&self.path)
                .expect("the temp endpoint file should be writable");
            file.write_all(text.as_bytes()).expect("the temp endpoint file should take the text");
        }
        #[cfg(not(unix))]
        std::fs::write(&self.path, text).expect("the temp endpoint file should be writable");
    }

    fn provider(&self) -> CaimProvider {
        CaimProvider::with_endpoint_file(self.path.clone())
    }
}

impl Drop for TempEndpoint {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn unique_path() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("photocraft-caim-test-{}-{n}.json", std::process::id()))
}

fn endpoint_json(port: u16, token: &str) -> String {
    format!(r#"{{"version": 1, "port": {port}, "token": "{token}"}}"#)
}

/// A port nothing is listening on: bind one, then let go of it.
fn closed_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("the test should be able to bind a loopback port");
    listener.local_addr().expect("a bound listener has an address").port()
}

const RGB8: PixelFormat = PixelFormat::new(ColorMode::Rgb, SampleType::U8, false);
const RGB16: PixelFormat = PixelFormat::new(ColorMode::Rgb, SampleType::U16, false);
const RGB32F: PixelFormat = PixelFormat::new(ColorMode::Rgb, SampleType::F32, false);
const RGBA8: PixelFormat = PixelFormat::new(ColorMode::Rgb, SampleType::U8, true);

/// A deterministic window: every sample a different value in 0..=1, so a round trip is visible.
fn request(width: u32, height: u32, format: PixelFormat) -> AiRequest {
    let samples = usize::try_from(width).expect("test width") * usize::try_from(height).expect("test height") * format.channels();
    let pixels: Vec<f32> = (0..samples).map(|index| ((index * 7) % 251) as f32 / 250.0).collect();
    let coverage: Vec<f32> =
        (0..(usize::try_from(width).expect("test width") * usize::try_from(height).expect("test height"))).map(|index| (index % 101) as f32 / 100.0).collect();
    AiRequest {
        task: AiTask::FillMaskedArea,
        window: Rect::new(10, 20, 10 + width as i32, 20 + height as i32),
        sent_size: None,
        pixels,
        format,
        coverage,
        prompt: None,
        client_version: "photocraft/0.0.0-test".into(),
    }
}

const DONE: &str = r#"{"state": "done", "progress": 1.0, "message": ""}"#;
const PROVENANCE: &str = r#"{"model": "lama-onnx", "device": "cpu", "provider": "local", "elapsed_ms": 1234, "image_size": [4, 3]}"#;

/// Submit `req` and fetch its result, where the manager answers with `png` as `result.png`.
fn submit_and_fetch(server: &FakeServer, endpoint: &TempEndpoint, req: &AiRequest, png: &[u8], content_type: &str) -> Result<AiResult, AiError> {
    server.queue_json(202, r#"{"id": "job-1"}"#);
    let provider = endpoint.provider();
    let id = provider.submit(req).expect("the job should be accepted");
    server.queue_json(200, PROVENANCE);
    server.queue(response(200, content_type, png));
    provider.result(&id)
}

fn header_value(content_type: Option<&str>) -> Option<String> {
    content_type.and_then(|value| value.split("boundary=").nth(1)).map(str::to_string)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// The named parts of a captured `multipart/form-data` body, in order.
fn multipart_parts(body: &[u8], boundary: &str) -> Vec<(String, Vec<u8>)> {
    let marker = format!("--{boundary}");
    let mut parts = Vec::new();
    let mut at = 0;
    while let Some(offset) = find_bytes(&body[at..], marker.as_bytes()) {
        let start = at + offset + marker.len();
        let end = find_bytes(&body[start..], marker.as_bytes()).unwrap_or(body.len() - start) + start;
        let chunk = &body[start..end];
        if let Some(head_end) = find_bytes(chunk, b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&chunk[..head_end]).to_string();
            let name = head.split("name=\"").nth(1).and_then(|rest| rest.split('"').next()).unwrap_or_default().to_string();
            let mut data = &chunk[head_end + 4..];
            if data.ends_with(b"\r\n") {
                data = &data[..data.len() - 2];
            }
            parts.push((name, data.to_vec()));
        }
        at = end;
    }
    parts
}

fn png_header(png: &[u8]) -> (u32, u32, png::ColorType, png::BitDepth) {
    let mut decoder = png::Decoder::new(Cursor::new(png));
    let info = decoder.read_header_info().expect("the part should be a PNG");
    (info.width, info.height, info.color_type, info.bit_depth)
}

#[test]
fn happy_path_round_trips_8_bit_pixels() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-8"));
    let provider = endpoint.provider();
    let req = request(4, 3, RGB8);

    server.queue_json(202, r#"{"id": "job-8"}"#);
    let id = provider.submit(&req).expect("the job should be accepted");
    assert_eq!(id.as_str(), "job-8");

    server.queue_json(200, r#"{"state": "running", "progress": 0.5, "message": "inpainting"}"#);
    let state = provider.poll(&id).expect("the job should be pollable");
    assert!(matches!(&state, AiJobState::Running { progress, message } if (progress - 0.5).abs() < 1e-6 && message == "inpainting"));

    server.queue_json(200, DONE);
    assert_eq!(provider.poll(&id).expect("the job should report done"), AiJobState::Done);

    let png = pixels::encode_image(&req.pixels, req.format, 4, 3).expect("the client should be able to encode its own window");
    server.queue_json(200, PROVENANCE);
    server.queue(response(200, "image/png", &png));
    let result = provider.result(&id).expect("the result should be readable");

    assert_eq!(result.window, req.window);
    let AiOutput::Image { pixels, format } = &result.output else {
        panic!("a fill answer is an image");
    };
    assert_eq!(*format, req.format);
    assert_eq!(pixels.len(), req.pixels.len());
    for (got, want) in pixels.iter().zip(&req.pixels) {
        assert!((got - want).abs() <= 1.0 / 255.0, "{got} is not within 1/255 of {want}");
    }
    assert_eq!(result.model, "lama-onnx");
    assert_eq!(result.device, "cpu");
    assert_eq!(result.provider, "local");
    assert_eq!(result.elapsed_ms, 1234);
    // Health is not required before a job: the first request is the job itself.
    assert_eq!(
        server.captured().iter().map(|call| (call.method.as_str(), call.path.as_str())).collect::<Vec<_>>(),
        vec![
            ("POST", "/v1/jobs"),
            ("GET", "/v1/jobs/job-8"),
            ("GET", "/v1/jobs/job-8"),
            ("GET", "/v1/jobs/job-8/result"),
            ("GET", "/v1/jobs/job-8/result.png")
        ]
    );
}

#[test]
fn happy_path_round_trips_16_bit_pixels() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-16"));
    let provider = endpoint.provider();
    let req = request(3, 5, RGB16);

    server.queue_json(202, r#"{"id": "job-16"}"#);
    let id = provider.submit(&req).expect("the job should be accepted");
    let png = pixels::encode_image(&req.pixels, req.format, 3, 5).expect("the client should be able to encode its own window");
    server.queue_json(200, r#"{"state": "done"}"#);
    assert_eq!(provider.poll(&id).expect("the job should report done"), AiJobState::Done);
    server.queue_json(200, r#"{"model": "m", "device": "cuda", "provider": "local", "elapsed_ms": 7, "image_size": [3, 5]}"#);
    server.queue(response(200, "image/png", &png));
    let result = provider.result(&id).expect("the result should be readable");
    let AiOutput::Image { pixels, .. } = &result.output else {
        panic!("a fill answer is an image");
    };
    for (got, want) in pixels.iter().zip(&req.pixels) {
        assert!((got - want).abs() <= 1.0 / 65535.0, "{got} is not within 1/65535 of {want}");
    }
}

#[test]
fn a_32_bit_document_sends_16_bit_png() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-32"));
    let req = request(2, 2, RGB32F);
    server.queue_json(202, r#"{"id": "job-32"}"#);
    endpoint.provider().submit(&req).expect("the job should be accepted");
    let captured = server.captured();
    let post = &captured[0];
    let boundary = header_value(post.header("content-type")).expect("the upload should name a boundary");
    let parts = multipart_parts(&post.body, &boundary);
    assert_eq!(parts.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), vec!["job", "image", "mask"]);
    let job: serde_json::Value = serde_json::from_slice(&parts[0].1).expect("the job part should be JSON");
    assert_eq!(job["bit_depth"], 16);
    assert_eq!(png_header(&parts[1].1), (2, 2, png::ColorType::Rgb, png::BitDepth::Sixteen));
}
#[test]
fn the_multipart_upload_has_the_three_parts_in_order() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-parts"));
    let req = request(4, 3, RGBA8);
    server.queue_json(202, r#"{"id": "job-parts"}"#);
    endpoint.provider().submit(&req).expect("the job should be accepted");

    let captured = server.captured();
    let post = &captured[0];
    assert_eq!((post.method.as_str(), post.path.as_str()), ("POST", "/v1/jobs"));
    let content_type = post.header("content-type").expect("the upload should have a content type");
    assert!(content_type.starts_with("multipart/form-data; boundary="), "{content_type}");
    let boundary = header_value(Some(content_type)).expect("the upload should name a boundary");
    let parts = multipart_parts(&post.body, &boundary);
    assert_eq!(parts.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), vec!["job", "image", "mask"]);

    let job: serde_json::Value = serde_json::from_slice(&parts[0].1).expect("the job part should be JSON");
    let mut keys: Vec<&str> = job.as_object().expect("the job part should be an object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["bit_depth", "channels", "client", "image_size", "prompt", "provider", "region", "settings", "task"]);
    // The wire order is the protocol's order, so a manager can read the parts as they arrive.
    let job_text = String::from_utf8_lossy(&parts[0].1);
    let mut seen = 0usize;
    for key in ["task", "image_size", "region", "bit_depth", "channels", "prompt", "settings", "provider", "client"] {
        let found = job_text.find(&format!("\"{key}\":")).expect("the job part should name every protocol key");
        assert!(found > seen, "the job part should name its keys in protocol order: {job_text}");
        seen = found;
    }
    assert_eq!(job["task"], "fill_masked_area");
    assert_eq!(job["image_size"], serde_json::json!([4, 3]));
    assert_eq!(job["region"], serde_json::json!([10, 20, 4, 3]));
    assert_eq!(job["bit_depth"], 8);
    assert_eq!(job["channels"], 4);
    assert!(job["prompt"].is_null());
    assert_eq!(job["settings"], serde_json::json!({}));
    assert_eq!(job["provider"], "auto");
    assert_eq!(job["client"], "photocraft/0.0.0-test");

    assert_eq!(png_header(&parts[1].1), (4, 3, png::ColorType::Rgba, png::BitDepth::Eight));
    assert_eq!(png_header(&parts[2].1), (4, 3, png::ColorType::Grayscale, png::BitDepth::Eight));
    let mut mask = png::Decoder::new(Cursor::new(&parts[2].1)).read_info().expect("the mask should be a PNG");
    let mut mask_pixels = vec![0u8; mask.output_buffer_size().expect("the mask should have a size")];
    mask.next_frame(&mut mask_pixels).expect("the mask should decode");
    for (value, want) in mask_pixels.iter().zip(&req.coverage) {
        assert!((f32::from(*value) / 255.0 - *want).abs() <= 1.0 / 255.0, "{value} is not coverage {want}");
    }
}
#[test]
fn every_request_carries_the_token_from_the_file() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-secret"));
    let provider = endpoint.provider();
    let req = request(2, 2, RGB8);
    server.queue_json(202, r#"{"id": "job-auth"}"#);
    let id = provider.submit(&req).expect("the job should be accepted");
    server.queue_json(200, DONE);
    provider.poll(&id).expect("the job should be pollable");
    server.queue(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".to_vec());
    provider.cancel(&id).expect("the job should be cancellable");

    let captured = server.captured();
    assert_eq!(captured.len(), 3);
    for call in &captured {
        assert_eq!(call.header("authorization"), Some("Bearer tok-secret"), "{} {} lost the token", call.method, call.path);
        assert_eq!(call.header("host"), Some(&format!("127.0.0.1:{}", server.port())[..]));
        assert_eq!(call.header("connection"), Some("close"));
    }
}

#[test]
fn an_unusable_endpoint_file_means_no_manager() {
    let server = FakeServer::start();
    let port = server.port();
    let req = request(2, 2, RGB8);
    let cases: [(&str, TempEndpoint); 8] = [
        ("a missing file", TempEndpoint::missing()),
        ("an empty file", TempEndpoint::new("")),
        ("a file that is not JSON", TempEndpoint::new("{")),
        ("a protocol version PhotoCraft does not know", TempEndpoint::new(r#"{"version": 2, "port": 1, "token": "t"}"#)),
        ("port 0", TempEndpoint::new(&endpoint_json(0, "t"))),
        ("a port that is not a port", TempEndpoint::new(r#"{"version": 1, "port": 70000, "token": "t"}"#)),
        ("an empty token", TempEndpoint::new(&endpoint_json(port, ""))),
        (r#"a token carrying a newline"#, TempEndpoint::new(&format!(r#"{{"version": 1, "port": {port}, "token": "a\nb"}}"#))),
    ];
    for (what, endpoint) in cases {
        let error = endpoint.provider().submit(&req).expect_err(&format!("{what} must not start a job"));
        assert_eq!(error, AiError::NoProvider, "{what}");
    }
    assert_eq!(server.capture_count(), 0, "a bad endpoint file must not get a request");
}

#[test]
fn an_over_large_endpoint_file_is_reported() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&format!(r#"{{"version": 1, "port": {}, "token": "{}"}}"#, server.port(), "t".repeat(70 * 1024)));
    let error = endpoint.provider().submit(&request(2, 2, RGB8)).expect_err("an over-large endpoint file is not a manager");
    assert!(!matches!(error, AiError::NoProvider), "the real reason should be reported: {error}");
    assert_eq!(server.capture_count(), 0);
}

#[test]
fn a_closed_port_is_no_manager() {
    let endpoint = TempEndpoint::new(&endpoint_json(closed_port(), "tok-gone"));
    let error = endpoint.provider().submit(&request(2, 2, RGB8)).expect_err("nothing is listening");
    assert_eq!(error, AiError::NoProvider);
    assert_eq!(error.to_string(), "Craft AI Manager isn't running");
}

#[test]
fn a_wrong_token_says_to_restart_the_manager() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-wrong"));
    server.queue_json(401, r#"{"error": "that token is not mine"}"#);
    let error = endpoint.provider().submit(&request(2, 2, RGB8)).expect_err("401 should fail the job");
    assert_eq!(error.to_string(), "Craft AI Manager refused the request (token mismatch); restart Craft AI Manager.");
}

#[test]
fn other_errors_use_the_manager_s_sentence() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-err"));
    let provider = endpoint.provider();
    server.queue_json(202, r#"{"id": "job-err"}"#);
    let id = provider.submit(&request(2, 2, RGB8)).expect("the job should be accepted");
    for (status, sentence) in [(404u16, "No job by that id."), (409, "The result is not ready yet."), (500, "The model crashed.")] {
        server.queue_json(status, &format!(r#"{{"error": "{sentence}"}}"#));
        assert_eq!(provider.poll(&id).expect_err("the manager said no").to_string(), sentence, "HTTP {status}");
    }
    server.queue(response(400, "text/plain", b"nope"));
    assert_eq!(provider.poll(&id).expect_err("a body that is not JSON still needs a message").to_string(), "Craft AI Manager answered HTTP 400.");
}

#[test]
fn a_malformed_reply_is_an_error_and_never_a_panic() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-bad"));
    let mut non_utf8 = b"HTTP/1.1 200 OK\r\nX-Strange: ".to_vec();
    non_utf8.extend_from_slice(&[0xff, 0xfe, 0x80]);
    non_utf8.extend_from_slice(b"\r\nContent-Length: 2\r\n\r\n{}");
    let cases: [(&str, Vec<u8>); 10] = [
        ("no status line", Vec::new()),
        ("a status line that is not HTTP", b"HTTP/2.0 200 OK\r\nContent-Length: 2\r\n\r\n{}".to_vec()),
        ("a status that is not a number", b"HTTP/1.1 abc OK\r\nContent-Length: 2\r\n\r\n{}".to_vec()),
        ("no Content-Length", b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n".to_vec()),
        ("a chunked body", b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 2\r\n\r\n{}".to_vec()),
        ("two Content-Length headers", b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 3\r\n\r\n{}".to_vec()),
        ("a Content-Length that is not a number", b"HTTP/1.1 200 OK\r\nContent-Length: two\r\n\r\n{}".to_vec()),
        ("a body over the cap", b"HTTP/1.1 200 OK\r\nContent-Length: 99999999999999\r\n\r\n{}".to_vec()),
        ("a body shorter than it promised", b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{}".to_vec()),
        ("a header that is not UTF-8", non_utf8),
    ];
    for (what, reply) in cases {
        server.queue(reply);
        let error = endpoint.provider().submit(&request(2, 2, RGB8)).expect_err(&format!("{what} must not be accepted"));
        assert!(!matches!(error, AiError::NoProvider), "{what} should be reported, not blamed on the manager being away");
    }
}

#[test]
fn the_result_png_must_match_the_window_that_was_sent() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-png"));
    let req = request(4, 3, RGB8);
    let good = pixels::encode_image(&req.pixels, req.format, 4, 3).expect("the client should encode its own window");

    let channels = pixels::channels(req.format).expect("an RGB window has a channel count");
    let wide: Vec<f32> = req.pixels.iter().cycle().take(5 * 3 * channels).copied().collect();
    let wrong_size = pixels::encode_image(&wide, req.format, 5, 3).expect("a 5x3 PNG is a valid PNG");
    let error = submit_and_fetch(&server, &endpoint, &req, &wrong_size, "image/png").expect_err("a result of the wrong size must not be used");
    assert!(error.to_string().contains("size"), "{error}");

    let deep16 = pixels::encode_image(&req.pixels, RGB16, 4, 3).expect("a 16-bit PNG is a valid PNG");
    let error = submit_and_fetch(&server, &endpoint, &req, &deep16, "image/png").expect_err("a result of the wrong depth must not be used");
    assert!(error.to_string().contains("bit depth"), "{error}");

    let gray = pixels::encode_mask(&req.coverage, 4, 3).expect("a grayscale PNG is a valid PNG");
    submit_and_fetch(&server, &endpoint, &req, &gray, "image/png").expect_err("a grayscale result must not be used");

    submit_and_fetch(&server, &endpoint, &req, &good, "application/octet-stream").expect_err("the result has to be labelled a PNG");
    submit_and_fetch(&server, &endpoint, &req, b"not a png at all", "image/png").expect_err("a body that is not a PNG must not be used");

    let huge = vec![7u8; 2 * 1024 * 1024];
    let error = submit_and_fetch(&server, &endpoint, &req, &huge, "image/png").expect_err("a 2 MiB result for a 12-pixel window is refused");
    assert!(error.to_string().contains("limit"), "{error}");

    // And the good one still works, so the checks above are not just failing everything.
    let result = submit_and_fetch(&server, &endpoint, &req, &good, "image/png").expect("the result should be readable");
    let AiOutput::Image { pixels, .. } = &result.output else {
        panic!("a fill answer is an image");
    };
    assert_eq!(pixels.len(), req.pixels.len());
}

#[test]
fn cancel_goes_to_the_manager_that_took_the_job() {
    let first = FakeServer::start();
    let second = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(first.port(), "tok-first"));
    let provider = endpoint.provider();
    first.queue_json(202, r#"{"id": "job-stay"}"#);
    let id = provider.submit(&request(2, 2, RGB8)).expect("the job should be accepted");
    // The manager restarts: a new port, a new token, a new file.
    endpoint.set(&endpoint_json(second.port(), "tok-second"));
    first.queue(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".to_vec());
    provider.cancel(&id).expect("the job should be cancelled where it was started");
    let captured = first.captured();
    let paths: Vec<(&str, &str)> = captured.iter().map(|call| (call.method.as_str(), call.path.as_str())).collect();
    assert!(paths.contains(&("DELETE", "/v1/jobs/job-stay")), "{paths:?}");
    assert_eq!(second.capture_count(), 0, "the new manager must not be told about the old job");
}

#[test]
fn the_endpoint_file_is_read_again_on_every_submit() {
    let first = FakeServer::start();
    let second = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(first.port(), "tok-one"));
    let provider = endpoint.provider();
    let req = request(2, 2, RGB8);
    first.queue_json(202, r#"{"id": "job-one"}"#);
    provider.submit(&req).expect("the first job should be accepted");
    endpoint.set(&endpoint_json(second.port(), "tok-two"));
    second.queue_json(202, r#"{"id": "job-two"}"#);
    provider.submit(&req).expect("the second job should reach the new manager");
    assert_eq!(first.captured()[0].header("authorization"), Some("Bearer tok-one"));
    assert_eq!(second.captured()[0].header("authorization"), Some("Bearer tok-two"));
    assert_eq!(second.capture_count(), 1);
}
#[test]
fn an_endpoint_file_naming_another_host_is_refused() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&format!(r#"{{"version": 1, "host": "127.0.0.2", "port": {}, "token": "t"}}"#, server.port()));
    let error = endpoint.provider().submit(&request(2, 2, RGB8)).expect_err("PhotoCraft must not leave loopback");
    assert!(error.to_string().contains("127.0.0.1"), "{error}");
    assert_eq!(server.capture_count(), 0, "a non-loopback host must not get a request");
    let ok = TempEndpoint::new(&format!(r#"{{"version": 1, "host": "127.0.0.1", "port": {}, "token": "t"}}"#, server.port()));
    server.queue_json(202, r#"{"id": "job-loop"}"#);
    ok.provider().submit(&request(2, 2, RGB8)).expect("loopback is the address the protocol allows");
}

#[test]
fn non_rgb_documents_are_refused_before_any_request() {
    let server = FakeServer::start();
    let provider = TempEndpoint::new(&endpoint_json(server.port(), "tok-mode")).provider();
    for format in [
        PixelFormat::GRAY8,
        PixelFormat::GRAYA8,
        PixelFormat::CMYKA8,
        PixelFormat::new(ColorMode::Lab, SampleType::F32, true),
        PixelFormat::new(ColorMode::Indexed, SampleType::U8, false),
    ] {
        let error = provider.submit(&request(2, 2, format)).expect_err("only RGB is in scope for M1");
        assert!(error.to_string().contains("RGB"), "{format:?}: {error}");
    }
    assert_eq!(server.capture_count(), 0, "an unsupported document should not reach the manager");
}

#[test]
fn a_request_that_does_not_match_its_window_is_refused() {
    let server = FakeServer::start();
    // Kept alive: the provider reads the file before it encodes (BF-02 P4).
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-shape"));
    let provider = endpoint.provider();
    let mut short = request(4, 3, RGB8);
    short.pixels.pop();
    assert!(provider.submit(&short).is_err(), "too few samples for the window");
    let mut wide = request(4, 3, RGB8);
    wide.coverage.pop();
    assert!(provider.submit(&wide).is_err(), "too few coverage values for the window");
    let empty = AiRequest {
        task: AiTask::FillMaskedArea,
        window: Rect::EMPTY,
        sent_size: None,
        pixels: Vec::new(),
        format: RGB8,
        coverage: Vec::new(),
        prompt: None,
        client_version: "photocraft/test".into(),
    };
    assert!(provider.submit(&empty).is_err(), "an empty window cannot be filled");
    let mut poisoned = request(4, 3, RGB8);
    poisoned.pixels[0] = f32::NAN;
    let error = provider.submit(&poisoned).expect_err("a NaN cannot be sent as a pixel");
    assert!(error.to_string().contains("not a number"), "{error}");
    let mut outside = request(4, 3, RGB8);
    outside.coverage[0] = 1.4;
    assert!(provider.submit(&outside).is_err(), "coverage outside 0..=1 is not a selection");
    assert_eq!(server.capture_count(), 0, "a bad request should not reach the manager");
}

#[test]
fn a_prompt_is_sent_as_text_or_refused() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-prompt"));
    let mut req = request(2, 2, RGB8);
    req.prompt = Some("remove the cat\nplease".into());
    server.queue_json(202, r#"{"id": "job-prompt"}"#);
    endpoint.provider().submit(&req).expect("a plain-text prompt should be sent");
    let captured = server.captured();
    let boundary = header_value(captured[0].header("content-type")).expect("the upload should name a boundary");
    let job: serde_json::Value = serde_json::from_slice(&multipart_parts(&captured[0].body, &boundary)[0].1).expect("the job part should be JSON");
    assert_eq!(job["prompt"], "remove the cat\nplease");

    let mut control = request(2, 2, RGB8);
    control.prompt = Some("remove\0the cat".into());
    assert!(endpoint.provider().submit(&control).is_err(), "a prompt with a NUL is not plain text");
    let mut long = request(2, 2, RGB8);
    long.prompt = Some("q".repeat(5 * 1024));
    assert!(endpoint.provider().submit(&long).is_err(), "a prompt over 4 KiB is refused");
}

#[test]
fn a_job_id_that_cannot_go_in_a_url_is_refused() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-id"));
    for body in [r#"{"id": ""}"#, r#"{"id": "a b"}"#, r#"{"id": "a/b"}"#, r#"{"id": ".."}"#, r#"{"id": "a?b"}"#, r#"{"notAnId": 1}"#, "not json"] {
        server.queue_json(202, body);
        assert!(endpoint.provider().submit(&request(2, 2, RGB8)).is_err(), "{body} is not a usable job id");
    }
    server.queue_json(202, &format!(r#"{{"id": "{}"}}"#, "j".repeat(600)));
    assert!(endpoint.provider().submit(&request(2, 2, RGB8)).is_err(), "an absurdly long job id is refused");
    let calls = server.captured();
    assert_eq!(calls.iter().filter(|call| call.path != "/v1/jobs").count(), 0, "a refused job id must not be sent back to the manager");
}

#[test]
fn tasks_lists_what_the_manager_offers() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-tasks"));
    let provider = endpoint.provider();
    server.queue_json(200, r#"{"tasks": [{"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 400000000}, {"id": "select_subject"}]}"#);
    assert_eq!(provider.tasks().expect("the task list should parse"), vec![AiTask::FillMaskedArea]);
    server.queue_json(200, r#"{"tasks": []}"#);
    assert!(provider.tasks().expect("an empty task list is a valid answer").is_empty());
    server.queue_json(200, r#"{"nonsense": true}"#);
    assert!(provider.tasks().is_err(), "an answer that is not a task list is an error");
    assert_eq!(TempEndpoint::missing().provider().tasks().expect_err("no manager, no tasks"), AiError::NoProvider);
}

#[test]
fn job_states_map_to_the_engine_states() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-state"));
    let provider = endpoint.provider();
    server.queue_json(202, r#"{"id": "job-state"}"#);
    let id = provider.submit(&request(2, 2, RGB8)).expect("the job should be accepted");
    let cases: [(&str, AiJobState); 5] = [
        (
            r#"{"state": "queued", "progress": 0.0, "message": "waiting for the model"}"#,
            AiJobState::Queued { progress: 0.0, message: "waiting for the model".into() },
        ),
        (r#"{"state": "running", "progress": 1.5, "message": "more than whole"}"#, AiJobState::Running { progress: 1.0, message: "more than whole".into() }),
        (r#"{"state": "done", "progress": 1.0, "message": ""}"#, AiJobState::Done),
        (r#"{"state": "failed", "message": "the model ran out of memory"}"#, AiJobState::Failed("the model ran out of memory".into())),
        (r#"{"state": "cancelled"}"#, AiJobState::Cancelled),
    ];
    for (body, want) in cases {
        server.queue_json(200, body);
        assert_eq!(provider.poll(&id).expect("the state should map"), want, "{body}");
    }
    server.queue_json(200, r#"{"state": "levitating"}"#);
    assert!(provider.poll(&id).is_err(), "a state PhotoCraft does not know is an error");
    server.queue_json(200, r#"{"progress": 0.5}"#);
    assert!(provider.poll(&id).is_err(), "an answer with no state is an error");
}

#[test]
fn a_job_this_session_never_started_is_an_error() {
    let server = FakeServer::start();
    let provider = TempEndpoint::new(&endpoint_json(server.port(), "tok-unknown")).provider();
    for id in ["never-submitted", "a b", ""] {
        let error = provider.poll(&AiJobId::new(id)).expect_err("PhotoCraft must not guess job ids");
        assert!(matches!(error, AiError::Other(_)), "{id:?}: {error}");
    }
    assert!(provider.cancel(&AiJobId::new("never-submitted")).is_err());
    assert!(provider.result(&AiJobId::new("never-submitted")).is_err());
    assert_eq!(server.capture_count(), 0, "an id this session never got must not be sent to the manager");
}

#[test]
fn a_json_answer_over_the_cap_is_refused() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-cap"));
    let provider = endpoint.provider();
    server.queue_json(202, r#"{"id": "job-cap"}"#);
    let id = provider.submit(&request(2, 2, RGB8)).expect("the job should be accepted");
    server.queue(response(200, "application/json", format!(r#"{{"state": "running", "message": "{}"}}"#, "m".repeat(100 * 1024)).as_bytes()));
    let error = provider.poll(&id).expect_err("a 100 KiB status answer is over the protocol cap");
    assert!(error.to_string().contains("limit"), "{error}");
}

#[test]
fn provenance_that_disagrees_with_the_window_is_refused() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-size"));
    let req = request(4, 3, RGB8);
    let png = pixels::encode_image(&req.pixels, req.format, 4, 3).expect("the client should encode its own window");
    server.queue_json(202, r#"{"id": "job-size"}"#);
    let provider = endpoint.provider();
    let id = provider.submit(&req).expect("the job should be accepted");
    server.queue_json(200, r#"{"model": "m", "device": "cpu", "provider": "local", "elapsed_ms": 1, "image_size": [99, 99]}"#);
    let error = provider.result(&id).expect_err("a result that claims another size is not this window");
    assert!(error.to_string().contains("size"), "{error}");
    // Missing provenance says "unknown" rather than failing a finished job.
    server.queue_json(200, r#"{}"#);
    server.queue(response(200, "image/png", &png));
    let result = provider.result(&id).expect("a result with no provenance is still a result");
    assert_eq!((result.model.as_str(), result.device.as_str(), result.provider.as_str(), result.elapsed_ms), ("unknown", "unknown", "unknown", 0));
}

// ------------------------------------------------------------------------------------- BF-02

/// A window whose PNG does not compress: an LCG, so the upload really is as big as the window.
fn noisy_request(width: u32, height: u32) -> AiRequest {
    let mut req = request(width, height, RGB8);
    let mut state = 0x2545_f491_u32;
    for sample in &mut req.pixels {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *sample = f32::from((state >> 24) as u8) / 255.0;
    }
    req
}

/// X2: a manager that answers 401 as soon as the head arrives, then ignores the rest of the upload.
/// The answer is still read and used, so the user is told to restart the manager (the common case:
/// it restarted with a new token), not that the connection was lost.
#[test]
fn an_early_401_during_a_large_upload_is_reported_as_the_token_message() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-stale"));
    // About 25 MB of PNG: far more than the loopback socket buffers hold, so the manager has
    // answered long before the upload ends.
    let req = noisy_request(2900, 2900);
    server.queue_early(response(401, "application/json", br#"{"error": "That token is not this manager's."}"#));
    let error = endpoint.provider().submit(&req).expect_err("a 401 must fail the job");
    assert_eq!(error.to_string(), TOKEN_MISMATCH);
    assert!(matches!(error, AiError::TokenMismatch), "the engine stops a fill on this kind and only this kind: {error}");
    let captured = server.captured();
    assert_eq!(captured.len(), 1);
    assert_eq!((captured[0].method.as_str(), captured[0].path.as_str()), ("POST", "/v1/jobs"));
    assert!(captured[0].body.len() < 64 * 1024, "the fake manager answered from the head, it did not keep the body");
}

/// X2: the same for any status the manager sends early (413, 503): its own sentence, not "lost".
/// The write-failure half of that path - nothing written again before the reply is read - is
/// proven by `a_failed_upload_write_is_not_tried_again_before_the_reply_is_read`.
#[test]
fn an_early_error_reply_uses_the_manager_s_sentence() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-busy"));
    server.queue_early(response(503, "application/json", br#"{"error": "Craft AI Manager is busy; try again in a moment."}"#));
    let error = endpoint.provider().submit(&noisy_request(2900, 2900)).expect_err("a 503 must fail the job");
    assert_eq!(error.to_string(), "Craft AI Manager is busy; try again in a moment.");
}

/// P4: the endpoint file is read before the image is encoded. The window holds a NaN, which the
/// encoder refuses ("not a number"); with no endpoint the answer is "isn't running", so the encoder
/// was never reached. With an endpoint, the same request does reach the encoder.
#[test]
fn with_no_endpoint_the_image_is_never_encoded() {
    let mut req = request(64, 64, RGB8);
    req.pixels[0] = f32::NAN;
    let error = TempEndpoint::missing().provider().submit(&req).expect_err("no manager");
    assert_eq!(error, AiError::NoProvider);
    assert_eq!(error.to_string(), "Craft AI Manager isn't running");
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-order"));
    let error = endpoint.provider().submit(&req).expect_err("a NaN cannot be encoded");
    assert!(error.to_string().contains("not a number"), "{error}");
    assert_eq!(server.capture_count(), 0);
}

/// P5: the multipart body is written piece by piece. The pieces are byte for byte the body PC-03
/// built in one buffer, and the image and mask are borrowed, not copied.
#[test]
fn the_multipart_pieces_equal_the_old_body_and_borrow_the_parts() {
    let job = br#"{"task": "fill_masked_area"}"#.to_vec();
    let image = vec![0x89u8; 70_000];
    let mask = vec![7u8; 333];
    let parts = [
        protocol::Part { name: "job", filename: Some("job.json"), content_type: "application/json", data: &job },
        protocol::Part { name: "image", filename: Some("image.png"), content_type: "image/png", data: &image },
        protocol::Part { name: "mask", filename: None, content_type: "image/png", data: &mask },
    ];
    let boundary = protocol::boundary(&parts).expect("a boundary");
    let pieces = protocol::multipart_pieces(&boundary, &parts);
    assert_eq!(pieces.concat(), protocol::multipart(&boundary, &parts), "the bytes on the wire are unchanged");
    for data in [&job[..], &image[..], &mask[..]] {
        let borrowed = pieces.iter().any(|piece| matches!(piece, std::borrow::Cow::Borrowed(b) if b.as_ptr() == data.as_ptr() && b.len() == data.len()));
        assert!(borrowed, "a part of {} bytes was copied instead of borrowed", data.len());
    }
}

/// P5: what a real submit puts on the wire is exactly the reference multipart body of its parts.
#[test]
fn the_uploaded_body_is_the_reference_multipart_body() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-wire"));
    let req = request(5, 4, RGBA8);
    server.queue_json(202, r#"{"id": "job-wire"}"#);
    endpoint.provider().submit(&req).expect("the job should be accepted");
    let post = &server.captured()[0];
    let boundary = header_value(post.header("content-type")).expect("a boundary");
    let parts = multipart_parts(&post.body, &boundary);
    let rebuilt = protocol::multipart(
        &boundary,
        &[
            protocol::Part { name: "job", filename: Some("job.json"), content_type: "application/json", data: &parts[0].1 },
            protocol::Part { name: "image", filename: Some("image.png"), content_type: "image/png", data: &parts[1].1 },
            protocol::Part { name: "mask", filename: Some("mask.png"), content_type: "image/png", data: &parts[2].1 },
        ],
    );
    assert_eq!(post.body, rebuilt);
    assert_eq!(post.header("content-length"), Some(&rebuilt.len().to_string()[..]));
}

/// P5: a body over 1 GiB is refused before connecting (the pieces here share one 64 MiB buffer).
#[test]
fn a_body_over_one_gib_is_refused_before_anything_is_sent() {
    let server = FakeServer::start();
    let chunk = vec![0u8; 64 * 1024 * 1024];
    let body: Vec<&[u8]> = std::iter::repeat_n(&chunk[..], 17).collect();
    let endpoint = crate::Endpoint::loopback(server.port(), "tok-big");
    let error = http::request(&endpoint, "POST", "/v1/jobs", Some("multipart/form-data; boundary=x"), &body, 1024).err().expect("over 1 GiB");
    assert!(error.to_string().contains("1 GiB"), "{error}");
    assert_eq!(server.capture_count(), 0, "nothing was sent");
    assert_eq!(http::body_length(&body[..16]).expect("exactly 1 GiB is allowed"), http::MAX_REQUEST_BODY_BYTES);
}

/// P2: the manager's `max_pixels` is read from `/v1/tasks`.
#[test]
fn max_pixels_comes_from_the_task_list() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-limit"));
    let provider = endpoint.provider();
    server.queue_json(
        200,
        r#"{"tasks": [{"id": "select_subject", "max_pixels": 5}, {"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 64000000}]}"#,
    );
    assert_eq!(provider.max_pixels(AiTask::FillMaskedArea), Ok(Some(64_000_000)));
    server.queue_json(200, r#"{"tasks": [{"id": "fill_masked_area"}]}"#);
    assert_eq!(provider.max_pixels(AiTask::FillMaskedArea), Ok(None), "no limit advertised");
    server.queue_json(401, r#"{"error": "no"}"#);
    let stale = provider.max_pixels(AiTask::FillMaskedArea).expect_err("stale token");
    assert_eq!(stale.to_string(), TOKEN_MISMATCH);
    assert!(matches!(stale, AiError::TokenMismatch), "the engine falls back to the default limit for every other error: {stale}");
    assert_eq!(TempEndpoint::missing().provider().max_pixels(AiTask::FillMaskedArea), Err(AiError::NoProvider));
    let calls = server.captured();
    assert!(calls.iter().all(|call| (call.method.as_str(), call.path.as_str()) == ("GET", "/v1/tasks")), "{calls:?}");
}

/// P2 end to end: Generative Fill asks the manager's limit first and refuses an over-limit window
/// with a sentence before any pixel is uploaded.
#[test]
fn generative_fill_refuses_a_window_over_the_manager_s_limit_before_uploading() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-e2e"));
    let mut s = photocraft_engine::Session::new();
    s.execute("file.new", serde_json::json!({"width": 64, "height": 48, "background": "white"})).expect("a document");
    s.execute("select.rect", serde_json::json!({"x": 26, "y": 16, "width": 12, "height": 14})).expect("a selection");
    s.set_ai_provider(Some(std::sync::Arc::new(endpoint.provider())));
    server.queue_json(200, r#"{"tasks": [{"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 1000}]}"#);
    let error = s.execute("edit.generativeFill", serde_json::json!({})).expect_err("over the manager's limit");
    assert!(error.to_string().contains("over the 1000 pixels Craft AI Manager accepts"), "{error}");
    let calls: Vec<(String, String)> = server.captured().into_iter().map(|call| (call.method, call.path)).collect();
    assert_eq!(calls, vec![("GET".to_string(), "/v1/tasks".to_string())], "nothing was uploaded");
}

/// P3 end to end: after the manager took a job, three poll errors in a row fail it and PhotoCraft
/// tells the manager to drop the job (DELETE), so failed jobs do not fill its queue.
#[test]
fn a_job_that_fails_after_submit_is_deleted_on_the_manager() {
    use photocraft_engine::ai::run_to_completion;
    use photocraft_engine::jobs::JobCtx;
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-cleanup"));
    let provider = endpoint.provider();
    server.queue_json(202, r#"{"id": "job-gone"}"#);
    for _ in 0..3 {
        server.queue_json(500, r#"{"error": "The manager tripped."}"#);
    }
    server.queue(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".to_vec());
    let error = run_to_completion(&provider, &request(4, 3, RGB8), &JobCtx::new(), std::time::Duration::ZERO).expect_err("three poll errors in a row");
    assert_eq!(error.to_string(), "The manager tripped.");
    let calls: Vec<(String, String)> = server.captured().into_iter().map(|call| (call.method, call.path)).collect();
    assert_eq!(calls.len(), 5, "{calls:?}");
    assert_eq!(calls[4], ("DELETE".to_string(), "/v1/jobs/job-gone".to_string()));

    // A failed result is cleaned up the same way, and two poll errors in a row are survived.
    server.queue_json(202, r#"{"id": "job-flaky"}"#);
    server.queue_json(500, r#"{"error": "blip"}"#);
    server.queue_json(500, r#"{"error": "blip"}"#);
    server.queue_json(200, DONE);
    server.queue_json(404, r#"{"error": "The result has expired."}"#);
    server.queue(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n".to_vec());
    let error = run_to_completion(&provider, &request(4, 3, RGB8), &JobCtx::new(), std::time::Duration::ZERO).expect_err("the result failed");
    assert_eq!(error.to_string(), "The result has expired.");
    let calls: Vec<(String, String)> = server.captured().into_iter().skip(5).map(|call| (call.method, call.path)).collect();
    assert_eq!(calls.len(), 6, "{calls:?}");
    assert_eq!(calls[4], ("GET".to_string(), "/v1/jobs/job-flaky/result".to_string()));
    assert_eq!(calls[5], ("DELETE".to_string(), "/v1/jobs/job-flaky".to_string()));
}

/// P7: a failed job without a message still gives the user a sentence.
#[test]
fn a_failed_job_without_a_message_gives_a_sentence() {
    for body in [r#"{"state": "failed"}"#, r#"{"state": "failed", "message": ""}"#, r#"{"state": "failed", "message": "  \u0007 "}"#] {
        let state = protocol::parse_state(body.as_bytes()).expect("a failed job is a state");
        assert_eq!(state, AiJobState::Failed("Craft AI Manager could not finish the job and did not say why.".into()), "{body}");
    }
}

/// P7: the image and mask PNGs use fast compression (loopback: speed over size).
#[test]
fn pngs_are_encoded_with_fast_compression() {
    let req = noisy_request(97, 61);
    let png = pixels::encode_image(&req.pixels, RGB8, 97, 61).expect("the window encodes");
    let raw: Vec<u8> = req.pixels.iter().map(|value| (value * 255.0 + 0.5) as u8).collect();
    let reference = |compression: png::Compression| {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, 97, 61);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(compression);
        let mut writer = encoder.write_header().expect("header");
        writer.write_image_data(&raw).expect("data");
        writer.finish().expect("finish");
        out
    };
    assert_ne!(reference(png::Compression::Fast), reference(png::Compression::Balanced), "the check below can tell them apart");
    assert_eq!(png, reference(png::Compression::Fast));
    assert!(matches!(pixels::PNG_COMPRESSION, png::Compression::Fast));
}

/// A writer that takes `quota` bytes and then fails, counting the attempts so a write after the
/// failure is visible.
#[derive(Debug)]
struct StallingWriter {
    stall: Mutex<Stall>,
    quota: usize,
}

#[derive(Debug, Default)]
struct Stall {
    taken: usize,
    failures: usize,
}

impl StallingWriter {
    fn new(quota: usize) -> Self {
        Self { stall: Mutex::new(Stall::default()), quota }
    }
}

impl Write for StallingWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let mut stall = self.stall.lock().unwrap_or_else(PoisonError::into_inner);
        if stall.taken >= self.quota {
            stall.failures += 1;
            return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "the manager stopped reading"));
        }
        let take = (self.quota - stall.taken).min(data.len());
        stall.taken += take;
        Ok(take)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// BF-02 follow-up (http.rs): after an upload write fails, nothing goes back out before the reply
/// is read. Dropping the BufWriter flushes what it still holds, and that second write resets the
/// connection on Windows, which throws away the answer the manager already sent.
#[test]
fn a_failed_upload_write_is_not_tried_again_before_the_reply_is_read() {
    // Many small pieces, so the 64 KiB BufWriter still holds bytes when the write fails; one large
    // piece would go straight to the writer and leave the buffer empty.
    let piece = vec![7u8; 1024];
    let body: Vec<&[u8]> = (0..400).map(|_| piece.as_slice()).collect();
    let (writer, error) =
        http::write_request(StallingWriter::new(40 * 1024), &Endpoint::loopback(1, "tok"), "POST", "/v1/jobs", None, &body, body.len() * 1024)
            .expect_err("the writer stops after 40 KiB");
    assert_eq!(error.to_string(), "PhotoCraft lost the connection to Craft AI Manager.");
    let stall = writer.stall.lock().unwrap_or_else(PoisonError::into_inner);
    assert_eq!(stall.failures, 1, "the failed write is not attempted a second time");
    assert_eq!(stall.taken, 40 * 1024, "the upload stopped part-way, with bytes left in the buffer");
}

/// BF-02 follow-up (amendment 16): the manager answers at full strength, opaque wherever the mask
/// is above zero, and PhotoCraft blends once - when the "Generative Fill" layer composites. So what
/// is shown is the fill weighted by the selection's coverage, not a blend done twice.
#[test]
fn a_full_strength_result_from_the_manager_composites_by_coverage() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-blend"));
    let mut s = photocraft_engine::Session::new();
    s.execute("file.new", serde_json::json!({"width": 64, "height": 48, "background": "black"})).expect("a document");
    s.execute("select.rect", serde_json::json!({"x": 26, "y": 16, "width": 12, "height": 14})).expect("a selection");
    // A half-covered ring around a fully covered middle: the coverage the manager is sent, and the
    // weight the composite is blended by.
    s.edit("half selected", |doc, _active| {
        let sel = doc.selection.as_mut().expect("the selection");
        sel.fill_rect(Rect::new(26, 16, 38, 30), &[0.5]);
        sel.fill_rect(Rect::new(27, 17, 37, 29), &[1.0]);
        Ok(())
    })
    .expect("a half-selected ring");
    // The padding around a 14-pixel selection is wider than the document, so the window is the
    // whole canvas and the manager's answer lines up with it pixel for pixel.
    let mut answer: Vec<f32> = Vec::with_capacity(64 * 48 * 4);
    for y in 0..48 {
        for x in 0..64 {
            let px = if (26..38).contains(&x) && (16..30).contains(&y) { [0.75, 0.75, 0.75, 1.0] } else { [0.0, 0.0, 0.0, 1.0] };
            answer.extend_from_slice(&px);
        }
    }
    let png = pixels::encode_image(&answer, RGBA8, 64, 48).expect("the manager's result encodes");
    s.set_ai_provider(Some(std::sync::Arc::new(endpoint.provider())));
    server.queue_json(200, r#"{"tasks": [{"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 64000000}]}"#);
    server.queue_json(202, r#"{"id": "job-blend"}"#);
    server.queue_json(200, DONE);
    server.queue_json(200, r#"{"model": "lama-onnx", "device": "cpu", "provider": "local", "elapsed_ms": 1234, "image_size": [64, 48]}"#);
    server.queue(response(200, "image/png", &png));
    let r = s.execute("edit.generativeFill", serde_json::json!({})).expect("the fill runs");
    assert_eq!(r["ai"]["model"].as_str(), Some("lama-onnx"), "{r}");
    // The layer holds the manager's answer at full strength; its alpha is the coverage, which is how
    // the blend is carried without blending yet.
    let state = s.active().expect("a document");
    let fill_layer = state.active_layer.expect("the fill layer is active");
    let doc = state.doc.clone();
    let surf = doc.layer(fill_layer).expect("the fill layer").surface().expect("a raster layer");
    let ring = surf.pixel(26, 20);
    assert!((ring[0] - 0.75).abs() <= 1.0 / 255.0, "the client hands back full strength, not a pre-blend: {ring:?}");
    assert!((ring[3] - 0.5).abs() <= 1.0 / 255.0, "the layer's alpha carries the coverage: {ring:?}");
    // What the canvas shows: one blend, weighted by coverage, over the black canvas.
    let composite = photocraft_compose::render(&doc, Rect::new(0, 0, 64, 48));
    let shown = |x: i32, y: i32| composite.px[(y * 64 + x) as usize];
    for channel in 0..3 {
        assert!((shown(26, 20)[channel] - 0.375).abs() <= 1.0 / 255.0, "half coverage composites halfway: {}", shown(26, 20)[channel]);
        assert!((shown(32, 23)[channel] - 0.75).abs() <= 1.0 / 255.0, "full coverage is the fill: {}", shown(32, 23)[channel]);
        assert!(shown(10, 10)[channel].abs() <= 1.0 / 255.0, "outside the selection is the canvas: {}", shown(10, 10)[channel]);
    }
    let post = server.captured().into_iter().find(|call| call.method == "POST").expect("the upload");
    let boundary = header_value(post.header("content-type")).expect("the upload should name a boundary");
    let parts = multipart_parts(&post.body, &boundary);
    assert_eq!(png_header(&parts[1].1), (64, 48, png::ColorType::Rgba, png::BitDepth::Eight), "the window the manager was sent");
}

/// BF-02 follow-up: a failed /v1/tasks is not a reason to refuse the fill. A manager that answers
/// with something that is not its task list still gets the window, under the default limit.
#[test]
fn a_manager_that_cannot_state_its_limit_still_gets_the_fill() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-nolimit"));
    let mut s = photocraft_engine::Session::new();
    s.execute("file.new", serde_json::json!({"width": 64, "height": 48, "background": "white"})).expect("a document");
    s.execute("select.rect", serde_json::json!({"x": 26, "y": 16, "width": 12, "height": 14})).expect("a selection");
    s.set_ai_provider(Some(std::sync::Arc::new(endpoint.provider())));
    server.queue_json(200, r#"{"nonsense": true}"#);
    server.queue_json(202, r#"{"id": "job-nolimit"}"#);
    server.queue_json(200, DONE);
    server.queue_json(200, r#"{"model": "lama-onnx", "device": "cpu", "provider": "local", "elapsed_ms": 1234, "image_size": [64, 48]}"#);
    let grey = vec![0.75f32; 64 * 48 * 4];
    server.queue(response(200, "image/png", &pixels::encode_image(&grey, RGBA8, 64, 48).expect("the result encodes")));
    let r = s.execute("edit.generativeFill", serde_json::json!({})).expect("the fill runs anyway");
    assert!(r["filled"].as_u64().unwrap_or_default() > 0, "the window was filled: {r}");
    let calls: Vec<(String, String)> = server.captured().into_iter().map(|call| (call.method, call.path)).collect();
    assert!(calls.contains(&("POST".to_string(), "/v1/jobs".to_string())), "the fill was still sent: {calls:?}");
}

/// BF-02 follow-up: a refused token is different - the manager is running but will not answer, so
/// the fill stops with the restart sentence before any pixel is encoded.
#[test]
fn a_refused_token_stops_generative_fill_before_the_upload() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-stale-e2e"));
    let mut s = photocraft_engine::Session::new();
    s.execute("file.new", serde_json::json!({"width": 64, "height": 48, "background": "white"})).expect("a document");
    s.execute("select.rect", serde_json::json!({"x": 26, "y": 16, "width": 12, "height": 14})).expect("a selection");
    s.set_ai_provider(Some(std::sync::Arc::new(endpoint.provider())));
    server.queue_json(401, r#"{"error": "That token is not this manager's."}"#);
    let error = s.execute("edit.generativeFill", serde_json::json!({})).expect_err("a refused token");
    assert_eq!(error.to_string(), TOKEN_MISMATCH, "{error}");
    let calls: Vec<(String, String)> = server.captured().into_iter().map(|call| (call.method, call.path)).collect();
    assert_eq!(calls, vec![("GET".to_string(), "/v1/tasks".to_string())], "nothing was uploaded");
}
// ---------------------------------------------------------------------------- PC-09: mask results and M2 fields

/// A mask request (amendment 18): the window is the whole document area, the image goes out
/// downscaled, and the mask says "look everywhere".
fn mask_request(width: u32, height: u32, sent: (u32, u32)) -> AiRequest {
    let mut req = request(width, height, RGBA8);
    req.task = AiTask::SubjectMask;
    req.sent_size = Some(Size::new(sent.0, sent.1));
    let n = usize::try_from(sent.0).expect("test width") * usize::try_from(sent.1).expect("test height");
    req.pixels = (0..n * req.format.channels()).map(|index| ((index * 7) % 251) as f32 / 250.0).collect();
    req.coverage = vec![1.0; n];
    req
}

/// A PNG in a shape the client did not ask for, for the mask answers that must be refused.
fn raw_png(width: u32, height: u32, color: png::ColorType, depth: png::BitDepth, raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(Cursor::new(&mut out), width, height);
    encoder.set_color(color);
    encoder.set_depth(depth);
    let mut writer = encoder.write_header().expect("the test PNG should have a header");
    writer.write_image_data(raw).expect("the test PNG should take its pixels");
    writer.finish().expect("the test PNG should finish");
    out
}

/// The samples of a PNG part, whatever it holds.
fn png_samples(png: &[u8]) -> Vec<u8> {
    let decoder = png::Decoder::new(Cursor::new(png));
    let mut reader = decoder.read_info().expect("the part should be a PNG");
    let mut raw = vec![0u8; reader.output_buffer_size().expect("a PNG has an output buffer size")];
    reader.next_frame(&mut raw).expect("the part should decode");
    raw
}

/// B20: an M1-shaped manager - no amendment 18 field anywhere - answers a Fill exactly as today.
#[test]
fn an_m1_shaped_manager_answers_a_fill_exactly_as_today() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-m1-fill"));
    let provider = endpoint.provider();
    server.queue_json(200, r#"{"tasks": [{"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 4000000}]}"#);
    server.queue_json(200, r#"{"tasks": [{"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 4000000}]}"#);
    assert_eq!(provider.tasks().expect("the M1 task list is readable"), vec![AiTask::FillMaskedArea]);
    assert_eq!(provider.max_pixels(AiTask::FillMaskedArea), Ok(Some(4_000_000)));
    let req = request(4, 3, RGB8);
    let png = pixels::encode_image(&req.pixels, RGB8, 4, 3).expect("the window encodes");
    let result = submit_and_fetch(&server, &endpoint, &req, &png, "image/png").expect("the M1 answer is a fill");
    let AiOutput::Image { pixels: out, format } = result.output else {
        panic!("a fill answer is an image: {:?}", result.output);
    };
    assert_eq!(format, RGB8);
    assert_eq!(out.len(), req.pixels.len(), "the same window comes back");
    for (got, want) in out.iter().zip(&req.pixels) {
        assert!((got - want).abs() <= 1.0 / 255.0, "{got} is not within 1/255 of {want}");
    }
    assert_eq!(result.provenance, AiProvenance::default(), "an M1 manager says nothing more, and that is not an error");
}

/// B20: an M2-shaped manager, carrying the new fields and fields this build has never heard of,
/// works for both task kinds.
#[test]
fn an_m2_shaped_manager_with_unknown_fields_answers_both_tasks() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-m2"));
    let provider = endpoint.provider();
    server.queue_json(
        200,
        r#"{"tasks": [
            {"id": "fill_masked_area", "needs_prompt": false, "max_pixels": 4000000, "output": "image", "ready": true, "model": "lama-onnx", "queue_ms": 5, "extra": {"a": 1}},
            {"id": "subject_mask", "needs_prompt": false, "max_pixels": 1000000, "output": "mask", "ready": true, "model": "birefnet-lite", "queue_ms": 5, "extra": [1, 2, 3]}
        ]}"#,
    );
    assert_eq!(provider.tasks().expect("both tasks are on offer"), vec![AiTask::FillMaskedArea, AiTask::SubjectMask]);
    let fill = request(4, 3, RGB8);
    let image = pixels::encode_image(&fill.pixels, RGB8, 4, 3).expect("the window encodes");
    server.queue_json(202, r#"{"id": "job-m2-fill"}"#);
    let id = provider.submit(&fill).expect("the fill is accepted");
    server.queue_json(
        200,
        r#"{"model": "lama-onnx", "device": "cpu", "provider": "local", "elapsed_ms": 1234, "image_size": [4, 3], "kind": "image", "model_version": "1.4", "model_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef", "device_requested": "auto", "device_name": "CPU", "fallback_reason": null, "diagnostics": "onnxruntime: cpu ep", "load_ms": 400, "run_ms": 800, "queue_ms": 5}"#,
    );
    server.queue(response(200, "image/png", &image));
    let result = provider.result(&id).expect("the M2 fill answer is a fill");
    assert!(matches!(result.output, AiOutput::Image { .. }), "{:?}", result.output);
    assert_eq!(result.provenance.model_version.as_deref(), Some("1.4"));

    let mask = mask_request(8, 6, (4, 3));
    server.queue_json(202, r#"{"id": "job-m2-mask"}"#);
    let id = provider.submit(&mask).expect("the mask job is accepted");
    server.queue_json(
        200,
        r#"{"model": "birefnet-lite", "device": "cuda", "provider": "local", "elapsed_ms": 90, "image_size": [4, 3], "kind": "mask", "model_version": "2025.09", "model_sha256": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "device_requested": "auto", "device_name": "NVIDIA GeForce RTX 4090", "fallback_reason": null, "load_ms": 2100, "run_ms": 60, "queue_ms": 5}"#,
    );
    let want: Vec<f32> = (0..12).map(|index| (index % 4) as f32 / 3.0).collect();
    server.queue(response(200, "image/png", &pixels::encode_mask(&want, 4, 3).expect("the mask encodes")));
    let result = provider.result(&id).expect("the M2 mask answer is a mask");
    let AiOutput::Mask { coverage, size } = result.output else {
        panic!("a mask answer is a mask: {:?}", result.output);
    };
    assert_eq!((size.width, size.height), (4, 3), "the mask comes back at the size that was sent");
    assert_eq!(coverage.len(), 12);
    assert_eq!(result.device, "cuda", "`device` still names the backend that ran");
    assert_eq!(result.provenance.device_name.as_deref(), Some("NVIDIA GeForce RTX 4090"));
}

/// Amendment 18: `ready` says the model is installed and acknowledged, and a task this build
/// cannot speak is skipped rather than an error.
#[test]
fn a_task_that_is_not_ready_is_not_offered() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-ready"));
    let provider = endpoint.provider();
    server.queue_json(
        200,
        r#"{"tasks": [{"id": "fill_masked_area", "max_pixels": 4000000, "output": "image", "ready": true, "model": "lama-onnx"}, {"id": "subject_mask", "max_pixels": 1000000, "output": "mask", "ready": false, "model": null}]}"#,
    );
    assert_eq!(provider.tasks().expect("the fill is on offer"), vec![AiTask::FillMaskedArea], "`ready: false` hides the mask task");
    // An unknown id, an answer that contradicts the id, and a repeat are all skipped.
    server.queue_json(
        200,
        r#"{"tasks": [{"id": "sam2_object_select", "output": "mask"}, {"id": "subject_mask", "output": "image"}, {"id": "fill_masked_area", "output": "mask"}, {"id": "fill_masked_area"}, {"id": "fill_masked_area", "output": "image"}]}"#,
    );
    assert_eq!(provider.tasks().expect("one task survives"), vec![AiTask::FillMaskedArea]);
    // A field of the wrong JSON type says nothing, so the M1 answer stands.
    server.queue_json(200, r#"{"tasks": [{"id": "subject_mask", "ready": "yes", "output": 3}]}"#);
    assert_eq!(provider.tasks().expect("a wrong-typed field is ignored"), vec![AiTask::SubjectMask]);
    // `max_pixels` still comes from the same list, ready or not.
    server.queue_json(200, r#"{"tasks": [{"id": "subject_mask", "max_pixels": 1234, "ready": false}]}"#);
    assert_eq!(provider.max_pixels(AiTask::SubjectMask), Ok(Some(1_234)));
}

/// A mask result round trips: 8-bit grayscale at the sent size becomes coverage in 0..=1, within
/// one 8-bit step of what the manager sent.
#[test]
fn a_mask_result_round_trips_at_the_sent_size() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-mask-round"));
    let provider = endpoint.provider();
    let req = mask_request(20, 10, (5, 4));
    let want: Vec<f32> = (0..20).map(|index| (index % 5) as f32 / 4.0).collect();
    let png = pixels::encode_mask(&want, 5, 4).expect("the mask encodes");
    server.queue_json(202, r#"{"id": "job-mask"}"#);
    let id = provider.submit(&req).expect("the mask job is accepted");
    server.queue_json(200, r#"{"model": "m", "device": "cpu", "provider": "local", "elapsed_ms": 5, "image_size": [5, 4], "kind": "mask"}"#);
    server.queue(response(200, "image/png", &png));
    let result = provider.result(&id).expect("the mask is a result");
    let AiOutput::Mask { coverage, size } = &result.output else {
        panic!("a mask answer is a mask: {:?}", result.output);
    };
    assert_eq!((size.width, size.height), (5, 4), "the mask is at the size that was sent, not the window");
    assert_eq!(coverage.len(), 20);
    for (got, want) in coverage.iter().zip(&want) {
        assert!((got - want).abs() <= 1.0 / 255.0, "{got} is not within one 8-bit step of {want}");
        assert!((0.0..=1.0).contains(got), "coverage stays in 0..=1");
    }
    result.check_against(&req).expect("the engine accepts this mask for the request it answers");
}

/// A mask answer in any other shape is an error, never a panic: 16-bit, RGB, the wrong size, no
/// `kind` on a mask task, an image for a mask task, or a mask for a fill.
#[test]
fn a_mask_answer_of_the_wrong_shape_is_refused() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-mask-bad"));
    let provider = endpoint.provider();
    let grey = |w: u32, h: u32| {
        raw_png(w, h, png::ColorType::Grayscale, png::BitDepth::Eight, &vec![128u8; usize::try_from(w).unwrap() * usize::try_from(h).unwrap()])
    };
    let cases: Vec<(&str, &str, Vec<u8>, AiRequest, bool)> = vec![
        (
            "a 16-bit mask",
            r#"{"kind": "mask", "image_size": [4, 3]}"#,
            raw_png(4, 3, png::ColorType::Grayscale, png::BitDepth::Sixteen, &[0u8; 24]),
            mask_request(8, 6, (4, 3)),
            true,
        ),
        (
            "an RGB mask",
            r#"{"kind": "mask", "image_size": [4, 3]}"#,
            raw_png(4, 3, png::ColorType::Rgb, png::BitDepth::Eight, &[0u8; 36]),
            mask_request(8, 6, (4, 3)),
            true,
        ),
        ("a mask at the window's size", r#"{"kind": "mask"}"#, grey(8, 6), mask_request(8, 6, (4, 3)), true),
        ("a mask answer with no kind", r#"{"model": "m"}"#, grey(4, 3), mask_request(8, 6, (4, 3)), false),
        ("an image for a mask task", r#"{"kind": "image"}"#, grey(4, 3), mask_request(8, 6, (4, 3)), false),
        ("a mask for a fill", r#"{"kind": "mask", "image_size": [4, 3]}"#, grey(4, 3), request(4, 3, RGB8), false),
    ];
    for (what, json, png, req, reads_png) in cases {
        server.queue_json(202, r#"{"id": "job-bad"}"#);
        let id = provider.submit(&req).unwrap_or_else(|error| panic!("the job is accepted for {what}: {error}"));
        server.queue_json(200, json);
        if reads_png {
            // The client asks for the image only once the answer could still be the right shape.
            server.queue(response(200, "image/png", &png));
        }
        let before = server.capture_count();
        match provider.result(&id) {
            Ok(result) => panic!("a {what} must be refused, got {:?}", result.output),
            Err(error) => assert!(matches!(error, AiError::Other(_)), "a {what} is an error with words: {error}"),
        }
        let calls = server.captured();
        let paths: Vec<&str> = calls.iter().skip(before).map(|call| call.path.as_str()).collect();
        assert_eq!(paths.contains(&"/v1/jobs/job-bad/result.png"), reads_png, "a {what} decides whether the image is asked for: {paths:?}");
    }
    // A fill whose answer says `"kind": "image"` is still amendment 11's answer.
    let req = request(4, 3, RGB8);
    let png = pixels::encode_image(&req.pixels, RGB8, 4, 3).expect("the window encodes");
    let result = submit_and_fetch(&server, &endpoint, &req, &png, "image/png").expect("an image kind is a fill");
    assert!(matches!(result.output, AiOutput::Image { .. }), "{:?}", result.output);
}

/// Amendment 18: the new result fields land in the engine's provenance, and a value of the wrong
/// JSON type is ignored rather than fatal.
#[test]
fn the_new_provenance_fields_are_parsed_or_ignored() {
    let full = protocol::parse_provenance(
        br#"{"model": "birefnet-lite", "device": "cuda", "provider": "local", "elapsed_ms": 90, "image_size": [4, 3], "kind": "mask", "model_version": "2025.09", "model_sha256": "ab12", "device_requested": "auto", "device_name": "NVIDIA GeForce RTX 4090", "fallback_reason": "insufficient_memory", "diagnostics": "onnxruntime: CUDA ep would not start", "load_ms": 2100, "run_ms": 60}"#,
    )
    .expect("the M2 result parses");
    assert_eq!(full.kind.as_deref(), Some("mask"));
    assert_eq!(full.extra.model_version.as_deref(), Some("2025.09"));
    assert_eq!(full.extra.model_sha256.as_deref(), Some("ab12"));
    assert_eq!(full.extra.device_requested.as_deref(), Some("auto"));
    assert_eq!(full.extra.device_name.as_deref(), Some("NVIDIA GeForce RTX 4090"));
    assert_eq!(full.extra.fallback_reason.as_deref(), Some("insufficient_memory"), "the reason is kept as text");
    assert_eq!(full.extra.diagnostics.as_deref(), Some("onnxruntime: CUDA ep would not start"));
    assert_eq!((full.extra.load_ms, full.extra.run_ms), (Some(2100), Some(60)));
    assert_eq!(full.elapsed_ms, 90, "elapsed_ms stays the total");
    // Every field empty, null or the wrong type: the result still stands, and each field is unknown.
    let junk = protocol::parse_provenance(
        br#"{"model": 7, "device": [], "kind": true, "model_version": 1, "model_sha256": {}, "device_requested": false, "device_name": "", "fallback_reason": 3, "diagnostics": null, "load_ms": "2100", "run_ms": -5, "image_size": "not a pair", "elapsed_ms": 1e9}"#,
    )
    .expect("a result full of junk still parses");
    assert_eq!(junk.model, "unknown");
    assert_eq!(junk.kind, None, "a kind of the wrong type is not a kind");
    assert_eq!(junk.extra, AiProvenance::default(), "nothing was said in a way this build can read");
    assert_eq!(junk.image_size, None);
    assert_eq!(junk.elapsed_ms, 0);
}

/// A mask submit sends the same three parts in the same order, with `image_size` at the size
/// actually sent, `region` as the whole window in document coordinates, and an all-255 mask.
#[test]
fn a_mask_submit_sends_the_sent_size_and_an_all_255_mask() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-mask-submit"));
    server.queue_json(202, r#"{"id": "job-mask-submit"}"#);
    endpoint.provider().submit(&mask_request(200, 100, (40, 20))).expect("the mask job is accepted");
    let post = server.captured().pop().expect("the upload was captured");
    assert_eq!((post.method.as_str(), post.path.as_str()), ("POST", "/v1/jobs"));
    let boundary = header_value(post.header("content-type")).expect("the upload should name a boundary");
    let parts = multipart_parts(&post.body, &boundary);
    let names: Vec<&str> = parts.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["job", "image", "mask"], "the same three parts in the same order");
    let job: serde_json::Value = serde_json::from_slice(&parts[0].1).expect("the job part is JSON");
    assert_eq!(job["task"], serde_json::json!("subject_mask"));
    assert_eq!(job["image_size"], serde_json::json!([40, 20]), "image_size is the size actually sent");
    assert_eq!(job["region"], serde_json::json!([10, 20, 200, 100]), "region stays the whole window in document coordinates");
    assert_eq!(job["bit_depth"], serde_json::json!(8));
    assert_eq!(job["channels"], serde_json::json!(4));
    assert_eq!(job["provider"], serde_json::json!("auto"), "the client never chooses a backend");
    assert_eq!(png_header(&parts[1].1), (40, 20, png::ColorType::Rgba, png::BitDepth::Eight), "the image is at the sent size");
    assert_eq!(png_header(&parts[2].1), (40, 20, png::ColorType::Grayscale, png::BitDepth::Eight), "the mask is 8-bit grayscale at the sent size");
    let mask = png_samples(&parts[2].1);
    assert_eq!(mask.len(), 40 * 20, "one byte a pixel");
    assert!(mask.iter().all(|byte| *byte == 255), "the mask says look everywhere");
}

/// Amendment 18: a 400 whose sentence names a missing model is passed through unchanged, so the
/// user can be told which model to add.
#[test]
fn a_missing_model_answers_with_its_own_sentence() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-not-installed"));
    let provider = endpoint.provider();
    server.queue_json(200, r#"{"tasks": [{"id": "subject_mask", "max_pixels": 1000000, "output": "mask", "ready": true}]}"#);
    assert!(provider.tasks().expect("the manager offers the task").contains(&AiTask::SubjectMask));
    server.queue_json(400, r#"{"error": "The Select Subject model isn't installed."}"#);
    let error = provider.submit(&mask_request(8, 6, (8, 6))).expect_err("the manager refuses the job");
    assert_eq!(error.to_string(), "The Select Subject model isn't installed.");
    assert!(matches!(error, AiError::Other(_)), "this is not a token problem: {error}");
}

/// PC-09 end to end: `select.subject` with `"method": "ai"` through this client and the fake
/// manager - the mask answer, and the provenance the command returns.
#[test]
fn select_subject_ai_gets_a_mask_through_the_real_client() {
    let server = FakeServer::start();
    let endpoint = TempEndpoint::new(&endpoint_json(server.port(), "tok-subject-e2e"));
    let mut s = photocraft_engine::Session::new();
    s.execute("file.new", serde_json::json!({"width": 8, "height": 6, "background": "white"})).expect("a document");
    s.set_ai_provider(Some(std::sync::Arc::new(endpoint.provider())));
    let tasks =
        r#"{"tasks": [{"id": "subject_mask", "needs_prompt": false, "max_pixels": 1000000, "output": "mask", "ready": true, "model": "birefnet-lite"}]}"#;
    server.queue_json(200, tasks);
    server.queue_json(200, tasks);
    server.queue_json(202, r#"{"id": "job-subject"}"#);
    server.queue_json(200, DONE);
    server.queue_json(
        200,
        r#"{"model": "birefnet-lite", "device": "cuda", "provider": "local", "elapsed_ms": 90, "image_size": [8, 6], "kind": "mask", "model_version": "2025.09", "model_sha256": "ab12", "device_requested": "auto", "device_name": "NVIDIA GeForce RTX 4090", "fallback_reason": null, "load_ms": 2100, "run_ms": 60}"#,
    );
    // The left half of the document is the subject.
    let coverage: Vec<f32> = (0..48).map(|index| if index % 8 < 4 { 1.0 } else { 0.0 }).collect();
    server.queue(response(200, "image/png", &pixels::encode_mask(&coverage, 8, 6).expect("the mask encodes")));
    let r = s.execute("select.subject", serde_json::json!({"method": "ai"})).expect("the AI selection runs");
    let ai = &r["ai"];
    assert_eq!(ai["model"].as_str(), Some("birefnet-lite"), "{r}");
    assert_eq!(ai["device"].as_str(), Some("cuda"), "the backend that ran: {r}");
    assert_eq!(ai["deviceName"].as_str(), Some("NVIDIA GeForce RTX 4090"), "{r}");
    assert_eq!(ai["modelVersion"].as_str(), Some("2025.09"), "{r}");
    assert_eq!(ai["modelSha256"].as_str(), Some("ab12"), "{r}");
    assert_eq!(ai["deviceRequested"].as_str(), Some("auto"), "{r}");
    assert_eq!(ai["loadMs"].as_u64(), Some(2100), "{r}");
    assert_eq!(ai["runMs"].as_u64(), Some(60), "{r}");
    assert!(!ai.as_object().expect("the provenance is an object").contains_key("fallbackReason"), "a null reason is not reported: {r}");
    let calls: Vec<(String, String)> = server.captured().into_iter().map(|call| (call.method, call.path)).collect();
    assert!(calls.contains(&("POST".to_string(), "/v1/jobs".to_string())), "the mask job was sent: {calls:?}");
    assert!(calls.contains(&("GET".to_string(), "/v1/jobs/job-subject/result.png".to_string())), "the mask was fetched: {calls:?}");
}
