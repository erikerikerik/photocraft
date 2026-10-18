//! Where Craft AI Manager is, and the token it asked for (SPEC > Job protocol v1 > Discovery).

use std::fs::File;
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::{Path, PathBuf};

use photocraft_engine::ai::AiError;
use serde::Deserialize;

/// The `version` an endpoint file must declare.
pub const ENDPOINT_VERSION: u64 = 1;
/// Environment variable naming the endpoint file instead of the platform default (amendment 11).
pub const ENDPOINT_ENV: &str = "CRAFT_AI_ENDPOINT";
/// The endpoint file is JSON, and the protocol caps JSON at 64 KiB.
const MAX_FILE_BYTES: u64 = 64 * 1024;
/// A token is a header value: bounded, printable ASCII, and never carrying CR or LF.
const MAX_TOKEN_BYTES: usize = 512;

/// The manager's loopback address plus the token it wrote for this start.
#[derive(Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub addr: SocketAddr,
    pub token: String,
}

impl std::fmt::Debug for Endpoint {
    /// The token is a credential, so it never appears in a log line or a debug print.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Endpoint").field("addr", &self.addr).field("token", &"[redacted]").finish()
    }
}

impl Endpoint {
    /// A loopback endpoint, for tests and for callers that already hold a port and token.
    pub fn loopback(port: u16, token: impl Into<String>) -> Self {
        Self { addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)), token: token.into() }
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// The `Host` header the protocol asks for.
    pub fn host_header(&self) -> String {
        format!("127.0.0.1:{}", self.addr.port())
    }
}

#[derive(Deserialize)]
struct EndpointFile {
    version: u64,
    port: u32,
    token: String,
    /// Optional in v1; the client only ever connects to loopback, so any other value is refused.
    #[serde(default)]
    host: Option<String>,
}

/// Read and check the endpoint file at `path`.
///
/// An unusable file is `AiError::NoProvider`: the answer the user needs is the same one as for a
/// manager that never started, and a half-written file must not look like a working connection.
///
/// What is opened is checked, not just its name (BF-04, mirroring upstream #5459): the file holds
/// the manager's token, so anything other than a regular file the owner alone can read is refused
/// before a byte of it is sent anywhere. The name is stat'd first, so a special file is refused
/// without opening it - opening a fifo would block - and the opened handle is checked again.
pub fn load(path: &Path) -> Result<Endpoint, AiError> {
    let text = read_file(path)?;
    let Ok(parsed) = serde_json::from_str::<EndpointFile>(&text) else {
        return Err(AiError::NoProvider);
    };
    if parsed.version != ENDPOINT_VERSION {
        return Err(AiError::NoProvider);
    }
    if let Some(host) = parsed.host.as_deref().map(str::trim)
        && host != "127.0.0.1"
    {
        return Err(AiError::Other(
            "Craft AI Manager's endpoint file names a host other than 127.0.0.1; PhotoCraft only talks to the manager over loopback.".into(),
        ));
    }
    let Ok(port) = u16::try_from(parsed.port) else {
        return Err(AiError::NoProvider);
    };
    if port == 0 || !is_usable_token(&parsed.token) {
        return Err(AiError::NoProvider);
    }
    Ok(Endpoint::loopback(port, parsed.token))
}

fn read_file(path: &Path) -> Result<String, AiError> {
    // BF-04 (review round 2): judge the name before opening it. Opening a fifo with no writer
    // blocks until a writer appears, so a check that runs after `File::open` can never run at all:
    // the AI job worker would sit in the open. `metadata` is a stat - it opens nothing - so a fifo,
    // a socket, a device or a folder is refused here instead. It follows a symlink to a regular
    // file, which is what the handle check below has always accepted.
    if !std::fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
        return Err(AiError::NoProvider);
    }
    let mut file = File::open(path).map_err(|_| AiError::NoProvider)?;
    let Ok(meta) = file.metadata() else {
        return Err(AiError::NoProvider);
    };
    // BF-04: a folder or a device where the manager's file belongs is not the manager's file.
    // Still checked on the open handle as well, so the size and mode checks below describe the
    // bytes we actually read.
    if !meta.is_file() {
        return Err(AiError::NoProvider);
    }
    // BF-04: Craft AI Manager writes this file 0600, so group or other bits mean something else
    // wrote it or loosened it - and it carries the token that lets a stranger read the user's
    // pixels back. Windows keeps today's behaviour: the file lives in this user's own
    // %LOCALAPPDATA%, where these mode bits say nothing about who can read the file.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o077 != 0 {
            return Err(AiError::Other(format!(
                "Craft AI Manager's endpoint file can be read or changed by other users (mode {:o}), so PhotoCraft won't use it. Restart Craft AI Manager to write a fresh one.",
                meta.mode() & 0o777
            )));
        }
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err(AiError::Other("The Craft AI Manager endpoint file is larger than 64 KiB.".into()));
    }
    let mut bytes = Vec::new();
    // Read past the cap so a file whose metadata lied is still caught before it is parsed.
    if file.by_ref().take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes).is_err() || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(AiError::Other("The Craft AI Manager endpoint file is larger than 64 KiB.".into()));
    }
    String::from_utf8(bytes).map_err(|_| AiError::NoProvider)
}

/// A token that can go into a header: non-empty, bounded, and printable ASCII (no CR, no LF).
fn is_usable_token(token: &str) -> bool {
    !token.is_empty() && token.len() <= MAX_TOKEN_BYTES && token.bytes().all(|b| (0x20..0x7f).contains(&b))
}

/// The file the manager writes on this platform (amendment 11). `None` when the platform has no
/// place for it, which the caller reports as "no manager".
pub fn default_endpoint_file() -> Option<PathBuf> {
    platform_data_dir().map(|dir| dir.join("CraftAI").join("endpoint.json"))
}

fn platform_data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA").filter(|value| !value.is_empty()).map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").filter(|value| !value.is_empty()).map(|home| PathBuf::from(home).join("Library").join("Application Support"))
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").filter(|value| !value.is_empty()).map(|home| PathBuf::from(home).join(".local").join("share")))
    }
}

/// Which endpoint file to read: `CRAFT_AI_ENDPOINT` when it names one, else the platform default.
pub fn endpoint_file_from_env() -> Option<PathBuf> {
    match std::env::var_os(ENDPOINT_ENV) {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => default_endpoint_file(),
    }
}

/// BF-04: the endpoint-file permissions rule. The module is `#[cfg(test)]` and the tests inside it
/// are `#[cfg(unix)]`, because the mode bits they check are Unix - Linux CI and the FreeBSD run are
/// their evidence. They live here rather than in `crate::tests` so they cannot collide with PC-09's
/// edits to that file. (The two `cfg` layers are not decoration: `clippy.toml`'s
/// `allow-expect-in-tests` only reaches test code, and `#[cfg(all(test, unix))]` is not test code as
/// far as clippy is concerned, so the helper's `expect` calls would fail CI's clippy.)
#[cfg(test)]
mod tests {
    use crate::endpoint::read_file;
    use photocraft_engine::ai::AiError;

    /// BF-04 (review round 2): a fifo with no writer is refused without opening it, because
    /// `File::open` on one blocks until a writer appears. `std` has no mkfifo and the workspace
    /// forbids `unsafe`, so the test asks the platform for the fifo and says so when it has none
    /// (Windows). The read runs on its own thread: if the pre-check ever regresses, the test fails
    /// in five seconds instead of hanging the CI job.
    #[test]
    fn an_endpoint_fifo_with_no_writer_is_refused_without_opening_it() {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("photocraft-caim-perm-fifo-{}-{n}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let made = std::process::Command::new("mkfifo").arg(&path).status();
        match made {
            Ok(status) if status.success() => {}
            unavailable => {
                eprintln!("skipping: this platform has no mkfifo: {unavailable:?}");
                return;
            }
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let fifo = path.clone();
        std::thread::spawn(move || {
            let _ = sender.send(read_file(&fifo));
        });
        let answer = receiver.recv_timeout(std::time::Duration::from_secs(5));
        let _ = std::fs::remove_file(&path);
        match answer {
            Ok(result) => assert_eq!(result, Err(AiError::NoProvider), "a fifo with no writer is 'no manager', and it is refused before anything is opened"),
            Err(timed_out) => panic!("read_file never answered within five seconds ({timed_out:?}): it opened the fifo instead of refusing it"),
        }
    }

    #[cfg(unix)]
    mod unix_perms {
        use crate::endpoint::{ENDPOINT_VERSION, load, read_file};
        use photocraft_engine::ai::AiError;
        use std::os::unix::fs::PermissionsExt;

        /// One endpoint file per test, deleted at the end, so these run in parallel with each other and
        /// with `crate::tests`.
        struct TempEndpoint {
            path: std::path::PathBuf,
        }

        impl TempEndpoint {
            fn new(who: &str) -> Self {
                static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
                let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!("photocraft-caim-perm-{}-{n}.json", std::process::id()));
                std::fs::write(&path, format!(r#"{{"version": {ENDPOINT_VERSION}, "port": 45123, "token": "tok-{who}"}}"#))
                    .expect("an endpoint file can be written");
                Self { path }
            }

            fn set_mode(&self, mode: u32) {
                std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(mode)).expect("the mode of a temp file can be set");
            }
        }

        impl Drop for TempEndpoint {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.path);
            }
        }

        #[test]
        fn an_endpoint_file_other_users_can_read_is_refused() {
            for mode in [0o644u32, 0o660] {
                let endpoint = TempEndpoint::new("readable");
                endpoint.set_mode(mode);
                let error = load(&endpoint.path).expect_err(&format!("a file other users can read (mode {:o}) must not be used", mode));
                let AiError::Other(sentence) = error else {
                    panic!("a file other users can read has to say why it is refused, not just 'no manager': {error}");
                };
                assert!(sentence.contains(&format!("mode {:o}", mode & 0o777)), "the sentence names the mode it found: {sentence}");
                assert!(sentence.contains("other users"), "and says what is wrong with it: {sentence}");
                assert!(sentence.contains("Restart Craft AI Manager"), "and says what to do about it: {sentence}");
            }
        }

        #[test]
        fn an_endpoint_file_only_its_owner_can_read_is_used() {
            let endpoint = TempEndpoint::new("owner");
            endpoint.set_mode(0o600);
            let loaded = load(&endpoint.path).expect("a 0600 endpoint file is what Craft AI Manager writes");
            assert_eq!(loaded.port(), 45123, "the port is read as usual");
            assert_eq!(loaded.token, "tok-owner", "so is the token");
        }

        #[test]
        fn an_endpoint_file_that_is_not_a_regular_file_is_no_manager() {
            let folder = std::env::temp_dir().join(format!("photocraft-caim-perm-dir-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&folder);
            std::fs::create_dir_all(&folder).expect("a temp folder can be made");
            assert_eq!(read_file(&folder), Err(AiError::NoProvider), "a folder where the endpoint file belongs is not a manager");
            let _ = std::fs::remove_dir_all(&folder);
        }
    }
}
