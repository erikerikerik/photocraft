//! Where Craft AI Manager registered its own executable (SPEC amendment 18, MG-12).
//!
//! The manager writes `CraftAI/app.json` next to its endpoint file, in the platform's data
//! directory: `{"version": 1, "exe": "<absolute path>"}`. PhotoCraft reads it only to decide
//! whether an AI command can offer to start the manager, and - after the user asks - to start it.
//! It is not the endpoint file: it carries no token, so nothing here is a credential, and nothing
//! here is logged.
//!
//! An unusable registration is an ordinary error, never a panic: a missing, malformed,
//! wrong-version, relative, wrongly-named or since-deleted registration simply offers no launch
//! action, which is what amendment 18 asks for.

use std::io::Read;
use std::path::{Path, PathBuf};

use photocraft_engine::ai::{AiError, AiProvider, AiTask};
use serde::Deserialize;

/// The `version` an app file must declare (the same number Craft AI Manager writes).
pub const APP_VERSION: u64 = 1;
/// The registration file's name, in the manager's own data directory.
pub const APP_FILE_NAME: &str = "app.json";
/// The file is JSON, and the protocol caps JSON at 64 KiB, so a bigger file is never read.
const MAX_FILE_BYTES: u64 = 64 * 1024;

/// The executable a usable registration points at, already checked to exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registration {
    pub exe: PathBuf,
}

/// Why a registration cannot be used to start the manager. Each case is a plain sentence, because
/// the UI shows the one sentence amendment 18 allows and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationError {
    /// No registration file where the manager could have written one.
    Missing,
    /// A file is there but is not something that can be read as a registration.
    Unusable,
    /// Bigger than the protocol's cap, so it was not read at all.
    TooLarge,
    /// Not JSON in the shape the manager writes.
    Malformed,
    /// A `version` this PhotoCraft does not understand.
    WrongVersion,
    /// `exe` is empty or relative: only an absolute path can be started.
    BadPath,
    /// `exe` is not named `caim.exe` (Windows) or `caim` elsewhere.
    WrongName,
    /// Well formed, but the executable it names is gone.
    Absent,
}

impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sentence = match self {
            Self::Missing => "Craft AI Manager has not registered itself with PhotoCraft, so PhotoCraft cannot start it.",
            Self::Unusable => "Craft AI Manager's registration file cannot be read, so PhotoCraft cannot start it.",
            Self::TooLarge => "Craft AI Manager's registration file is larger than 64 KiB, so PhotoCraft will not read it.",
            Self::Malformed => "Craft AI Manager's registration file is not the JSON it should be, so PhotoCraft cannot start it.",
            Self::WrongVersion => "Craft AI Manager's registration file was written by another version, so PhotoCraft cannot start it.",
            Self::BadPath => "Craft AI Manager's registration file does not name an absolute path, so PhotoCraft cannot start it.",
            Self::WrongName => "Craft AI Manager's registration file does not name the manager's executable, so PhotoCraft cannot start it.",
            Self::Absent => "Craft AI Manager's executable is no longer where its registration file says it is.",
        };
        f.write_str(sentence)
    }
}

#[derive(Deserialize)]
struct AppFile {
    version: u64,
    exe: String,
    // Any other field the manager adds later is ignored, as amendment 18 allows.
}

/// The file the manager writes on this platform (amendment 18, MG-12). `None` when the platform has
/// no data directory for it, which the caller treats as "not installed".
pub fn default_app_file() -> Option<PathBuf> {
    crate::endpoint::platform_data_dir().map(|dir| dir.join("CraftAI").join(APP_FILE_NAME))
}

/// Which registration file to read. `CRAFT_AI_ENDPOINT` relocates the manager's files together:
/// when it names an endpoint file, the registration belongs in the same directory. Otherwise the
/// platform default. A directory with an endpoint but no registration is "not installed", which is
/// the honest answer - PhotoCraft never guesses where an executable lives.
pub fn app_file_from_env() -> Option<PathBuf> {
    match std::env::var_os(crate::endpoint::ENDPOINT_ENV) {
        Some(value) if !value.is_empty() => {
            let dir = PathBuf::from(value);
            let dir = dir.parent().map(|dir| dir.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
            Some(dir.join(APP_FILE_NAME))
        }
        _ => default_app_file(),
    }
}

/// Whether a path names the manager's executable on this platform.
fn is_registered_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else { return false };
    #[cfg(windows)]
    {
        name.eq_ignore_ascii_case("caim.exe")
    }
    #[cfg(not(windows))]
    {
        name == "caim"
    }
}

/// Read and check the registration. `None` means "use [`app_file_from_env`]".
///
/// The name is stat'd before it is opened, as BF-04 does for the endpoint file: opening a fifo with
/// no writer would block the caller, and the caller is the UI thread.
pub fn read_registration(app_file: Option<&Path>) -> Result<Registration, RegistrationError> {
    let path = match app_file.map(|path| path.to_path_buf()).or_else(app_file_from_env) {
        Some(path) => path,
        None => return Err(RegistrationError::Missing),
    };
    let stat = match std::fs::metadata(&path) {
        Ok(stat) => stat,
        Err(_) => return Err(RegistrationError::Missing),
    };
    if !stat.is_file() {
        return Err(RegistrationError::Unusable);
    }
    if stat.len() > MAX_FILE_BYTES {
        return Err(RegistrationError::TooLarge);
    }
    let mut file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(_) => return Err(RegistrationError::Unusable),
    };
    // Read past the cap, so a file whose metadata lied is still refused before it is parsed.
    let mut bytes = Vec::new();
    if file.by_ref().take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes).is_err() || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(RegistrationError::TooLarge);
    }
    let Ok(text) = String::from_utf8(bytes) else { return Err(RegistrationError::Malformed) };
    let Ok(parsed) = serde_json::from_str::<AppFile>(&text) else { return Err(RegistrationError::Malformed) };
    if parsed.version != APP_VERSION {
        return Err(RegistrationError::WrongVersion);
    }
    let exe = PathBuf::from(parsed.exe.trim());
    if exe.as_os_str().is_empty() || !exe.is_absolute() {
        return Err(RegistrationError::BadPath);
    }
    if !is_registered_name(&exe) {
        return Err(RegistrationError::WrongName);
    }
    if !std::fs::metadata(&exe).is_ok_and(|stat| stat.is_file()) {
        return Err(RegistrationError::Absent);
    }
    Ok(Registration { exe })
}

/// Whether the manager is available for `task`, as far as PhotoCraft can tell without starting
/// anything. The registration comes first: with nothing registered there is nothing to offer, and
/// the manager is only asked whether it is running when a launch could actually help.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagerState {
    /// No usable registration: amendment 18's "isn't installed" sentence, and no launch action.
    NotInstalled,
    /// Registered, but nothing answered on loopback: the launch action belongs here.
    NotRunning,
    /// Running, but it does not offer `task`: the model is missing or unacknowledged.
    ModelMissing,
    /// Running and offering `task`, or answering something other than "not running" (a refused
    /// token, a blip): the failure has another cause, so the engine's own sentence stands alone.
    NoLaunchAction,
}

/// Ask the installed client which of those states applies. `app_file` is `None` for the platform's
/// own registration; tests pass their own. The provider is asked only when a launch could help, so
/// this never runs for a command that has nothing to do with the manager.
pub fn manager_state(provider: &dyn AiProvider, task: AiTask, app_file: Option<&Path>) -> ManagerState {
    if read_registration(app_file).is_err() {
        return ManagerState::NotInstalled;
    }
    match provider.tasks() {
        Ok(tasks) => {
            if tasks.contains(&task) {
                ManagerState::NoLaunchAction
            } else {
                ManagerState::ModelMissing
            }
        }
        Err(AiError::NoProvider) => ManagerState::NotRunning,
        Err(_) => ManagerState::NoLaunchAction,
    }
}
/// One folder per test, so these run in parallel with each other and with `crate::tests`.
#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_engine::ai::{AiError, AiJobId, AiJobState, AiProvider, AiRequest, AiResult};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(who: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(1);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("photocraft-caim-app-{who}-{}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("a temp folder can be made");
            Self { path }
        }

        fn file(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }

        /// The name the manager's executable has on this platform.
        fn exe_name() -> &'static str {
            if cfg!(windows) { "caim.exe" } else { "caim" }
        }

        /// Write a registration naming `exe` and declaring `version`.
        fn write_app(&self, version: u64, exe: &Path) -> PathBuf {
            let app = self.file(APP_FILE_NAME);
            std::fs::write(&app, format!(r#"{{"version": {version}, "exe": "{}"}}"#, json_escape(&exe.to_string_lossy())))
                .expect("a registration can be written");
            app
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    /// A path as it has to appear inside a JSON string, so a Windows path survives the round trip.
    fn json_escape(path: &str) -> String {
        path.replace('\\', "\\\\").replace('"', "\\\"")
    }

    #[test]
    fn a_valid_registration_is_read() {
        let dir = TempDir::new("valid");
        let exe = dir.file(TempDir::exe_name());
        std::fs::write(&exe, "a stand-in, not the manager").expect("the executable file can be written");
        let app = dir.write_app(APP_VERSION, &exe);
        let registration = read_registration(Some(&app)).expect("a version 1 registration naming an existing caim is usable");
        assert_eq!(registration.exe, exe, "the path is used exactly as it was written");
    }

    #[test]
    fn a_registration_path_may_contain_spaces_and_non_ascii() {
        let dir = TempDir::new("names");
        let nested = dir.path.join("Craft AI Manager ää 图片");
        std::fs::create_dir_all(&nested).expect("a folder with spaces and non-ASCII can be made");
        let exe = nested.join(TempDir::exe_name());
        std::fs::write(&exe, "a stand-in").expect("the executable file can be written");
        let app = dir.write_app(APP_VERSION, &exe);
        assert_eq!(read_registration(Some(&app)).expect("the path is read back as written").exe, exe);
    }

    #[test]
    fn fields_this_photocraft_does_not_know_are_ignored() {
        let dir = TempDir::new("extra");
        let exe = dir.file(TempDir::exe_name());
        std::fs::write(&exe, "a stand-in").expect("the executable file can be written");
        let app = dir.file(APP_FILE_NAME);
        std::fs::write(&app, format!(r#"{{"version": {APP_VERSION}, "exe": "{}", "installedBy": "CAIM 0.9"}}"#, json_escape(&exe.to_string_lossy())))
            .expect("a registration with an extra field can be written");
        assert_eq!(read_registration(Some(&app)).expect("an unknown field is not a reason to refuse a registration").exe, exe);
    }

    #[test]
    fn a_missing_registration_is_not_installed() {
        let dir = TempDir::new("missing");
        assert_eq!(read_registration(Some(&dir.file(APP_FILE_NAME))), Err(RegistrationError::Missing));
    }

    #[test]
    fn a_malformed_registration_is_refused() {
        let dir = TempDir::new("malformed");
        for (body, who) in [
            ("not json at all", "text"),
            ("", "empty"),
            (r#"{"version": 1}"#, "no exe"),
            (r#"{"version": "one", "exe": "/x/caim"}"#, "a version that is not a number"),
        ] {
            let app = dir.file(&format!("app-{who}.json"));
            std::fs::write(&app, body).expect("a file can be written");
            assert_eq!(read_registration(Some(&app)), Err(RegistrationError::Malformed), "{who} is not a registration");
        }
    }

    #[test]
    fn another_version_is_refused() {
        let dir = TempDir::new("version");
        let exe = dir.file(TempDir::exe_name());
        std::fs::write(&exe, "a stand-in").expect("the executable file can be written");
        let app = dir.write_app(APP_VERSION + 1, &exe);
        assert_eq!(read_registration(Some(&app)), Err(RegistrationError::WrongVersion), "a registration from another version is not guessed at");
    }

    #[test]
    fn a_relative_or_empty_path_is_refused() {
        let dir = TempDir::new("relative");
        for exe in ["caim.exe", "", "."] {
            let app = dir.file(&format!("app-{exe}.json"));
            std::fs::write(&app, format!(r#"{{"version": {APP_VERSION}, "exe": "{exe}"}}"#)).expect("a registration can be written");
            assert_eq!(read_registration(Some(&app)), Err(RegistrationError::BadPath), "{exe:?} is not an absolute path, so nothing may be started from it");
        }
    }

    #[test]
    fn an_executable_with_the_wrong_name_is_refused() {
        let dir = TempDir::new("wrong-name");
        for name in ["not-the-manager.exe", "caim.txt", "CAIM"] {
            let exe = dir.file(name);
            std::fs::write(&exe, "a stand-in").expect("a file can be written");
            let app = dir.write_app(APP_VERSION, &exe);
            assert_eq!(read_registration(Some(&app)), Err(RegistrationError::WrongName), "{name} is not the manager's executable on this platform");
        }
    }

    #[test]
    fn a_deleted_executable_is_refused() {
        let dir = TempDir::new("deleted");
        let exe = dir.file(TempDir::exe_name());
        std::fs::write(&exe, "a stand-in").expect("the executable file can be written");
        let app = dir.write_app(APP_VERSION, &exe);
        std::fs::remove_file(&exe).expect("the executable can be removed again");
        assert_eq!(read_registration(Some(&app)), Err(RegistrationError::Absent), "the manager was uninstalled since it registered itself");
    }

    #[test]
    fn a_folder_where_the_registration_belongs_is_unusable() {
        let dir = TempDir::new("folder");
        std::fs::create_dir_all(dir.file(APP_FILE_NAME)).expect("a folder can be made where the file belongs");
        assert_eq!(read_registration(Some(&dir.file(APP_FILE_NAME))), Err(RegistrationError::Unusable));
    }

    #[test]
    fn a_registration_bigger_than_the_cap_is_not_read() {
        let dir = TempDir::new("large");
        let app = dir.file(APP_FILE_NAME);
        let padding = "x".repeat(70 * 1024);
        std::fs::write(&app, format!(r#"{{"version": {APP_VERSION}, "exe": "caim", "note": "{padding}"}}"#)).expect("an oversized file can be written");
        assert_eq!(read_registration(Some(&app)), Err(RegistrationError::TooLarge), "the protocol's cap is the read limit");
    }

    /// A provider that answers what the test says and counts how often it was asked.
    struct FakeProvider {
        answer: Result<Vec<AiTask>, AiError>,
        asked: AtomicU64,
    }

    impl AiProvider for FakeProvider {
        fn tasks(&self) -> Result<Vec<AiTask>, AiError> {
            self.asked.fetch_add(1, Ordering::Relaxed);
            self.answer.clone()
        }
        fn submit(&self, _req: &AiRequest) -> Result<AiJobId, AiError> {
            Err(AiError::Other("PC-10 never gets this far".into()))
        }
        fn poll(&self, _id: &AiJobId) -> Result<AiJobState, AiError> {
            Err(AiError::Other("PC-10 never gets this far".into()))
        }
        fn result(&self, _id: &AiJobId) -> Result<AiResult, AiError> {
            Err(AiError::Other("PC-10 never gets this far".into()))
        }
        fn cancel(&self, _id: &AiJobId) -> Result<(), AiError> {
            Err(AiError::Other("PC-10 never gets this far".into()))
        }
    }

    /// A registration that works, for the state tests, which are about the manager rather than the
    /// file.
    fn registered(dir: &TempDir) -> PathBuf {
        let exe = dir.file(TempDir::exe_name());
        std::fs::write(&exe, "a stand-in").expect("the executable file can be written");
        dir.write_app(APP_VERSION, &exe)
    }

    #[test]
    fn nothing_registered_means_not_installed_and_no_question_to_the_manager() {
        let dir = TempDir::new("state-missing");
        let provider = FakeProvider { answer: Ok(vec![AiTask::SubjectMask]), asked: AtomicU64::new(0) };
        assert_eq!(manager_state(&provider, AiTask::SubjectMask, Some(&dir.file(APP_FILE_NAME))), ManagerState::NotInstalled);
        assert_eq!(provider.asked.load(Ordering::Relaxed), 0, "with nothing registered there is nothing to start, so the manager is not even asked");
    }

    #[test]
    fn a_registered_manager_that_never_answers_is_not_running() {
        let dir = TempDir::new("state-down");
        let app = registered(&dir);
        let provider = FakeProvider { answer: Err(AiError::NoProvider), asked: AtomicU64::new(0) };
        assert_eq!(manager_state(&provider, AiTask::SubjectMask, Some(&app)), ManagerState::NotRunning);
        assert_eq!(provider.asked.load(Ordering::Relaxed), 1, "a launch could help here, so the manager is asked once");
    }

    #[test]
    fn a_manager_without_the_task_is_reported_as_missing_a_model() {
        let dir = TempDir::new("state-model");
        let app = registered(&dir);
        let provider = FakeProvider { answer: Ok(vec![AiTask::FillMaskedArea]), asked: AtomicU64::new(0) };
        assert_eq!(manager_state(&provider, AiTask::SubjectMask, Some(&app)), ManagerState::ModelMissing);
    }

    #[test]
    fn a_manager_that_offers_the_task_gets_no_launch_action() {
        let dir = TempDir::new("state-ready");
        let app = registered(&dir);
        let provider = FakeProvider { answer: Ok(vec![AiTask::SubjectMask]), asked: AtomicU64::new(0) };
        assert_eq!(manager_state(&provider, AiTask::SubjectMask, Some(&app)), ManagerState::NoLaunchAction);
    }

    #[test]
    fn a_manager_that_answered_something_else_keeps_the_engines_own_sentence() {
        let dir = TempDir::new("state-token");
        let app = registered(&dir);
        let provider = FakeProvider { answer: Err(AiError::TokenMismatch), asked: AtomicU64::new(0) };
        assert_eq!(
            manager_state(&provider, AiTask::SubjectMask, Some(&app)),
            ManagerState::NoLaunchAction,
            "a manager that answered is running; starting it again is not the answer"
        );
    }

    /// The sentences are the whole story: none of them repeats a path, which is where a user's
    /// folders live.
    #[test]
    fn the_registration_sentences_name_no_path() {
        for error in [RegistrationError::Missing, RegistrationError::Absent, RegistrationError::WrongName, RegistrationError::BadPath] {
            let sentence = error.to_string();
            assert!(!sentence.contains("photocraft-caim-app"), "{sentence} leaks a temp path");
            assert!(!sentence.is_empty(), "{error:?} says something");
        }
    }
}
