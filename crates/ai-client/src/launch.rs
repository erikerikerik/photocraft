//! Starting the registered Craft AI Manager, and only when the user asks for it (SPEC amendment
//! 18). There is no shell, no command string, no PATH lookup and no install: the registered
//! executable is started directly with the single literal argument `open`, which is the manager's
//! own "bring the running manager forward, or start one" command (MG-12).

use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::app_file::{Registration, read_registration};

/// Windows: `CREATE_NO_WINDOW`, so a console-subsystem manager does not flash a console window
/// behind PhotoCraft.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Why starting the manager did not happen. Kept typed, so the UI can pick its one sentence rather
/// than reading an error string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchError {
    /// Nothing usable is registered, or the registration or its executable went away between
    /// showing the button and pressing it.
    NotRegistered,
    /// Something is registered, and starting it failed. `reason` is the platform's own words, for
    /// a notice line rather than a sentence the catalogs carry.
    Failed(String),
}

/// The command PhotoCraft would run: the registered executable, one literal argument, and nothing
/// inherited. Split out so a test can read exactly what would be started without starting it.
fn launch_command(registration: &Registration) -> Command {
    let mut command = Command::new(&registration.exe);
    command.arg("open").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// Start the registered manager. `app_file` is `None` for the platform's own registration.
///
/// The registration is read again here rather than trusted from whatever the message was built
/// from: the file, or the executable it names, can have gone in between. `caim open` is safe to
/// press twice - the manager keeps its own launch lock and reports that it is already running - so
/// this never claims a second manager started.
pub fn open_registered(app_file: Option<&Path>) -> Result<(), LaunchError> {
    let registration = match read_registration(app_file) {
        Ok(registration) => registration,
        Err(_) => return Err(LaunchError::NotRegistered),
    };
    match launch_command(&registration).spawn() {
        Ok(child) => {
            // The manager owns its own lifetime, so PhotoCraft must not wait for it - that would
            // stop the frame loop. It does collect the child, so the process cannot linger as a
            // zombie for the rest of this session.
            collect(child);
            Ok(())
        }
        Err(error) => Err(LaunchError::Failed(error.to_string())),
    }
}

/// Wait for a started manager on its own thread, so the one process PhotoCraft spawned is reaped
/// whatever happens to it. Nothing is done with the exit status: the manager's own window, or its
/// absence, is what the user sees.
fn collect(mut child: Child) {
    // Nothing to report if the thread cannot start: the manager still runs, and the
    // unwaited child is collected when this process exits. A thread here is the whole cost.
    let _ = std::thread::Builder::new().name("photocraft-caim-open".into()).spawn(move || {
        let _ = child.wait();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_file::APP_FILE_NAME;
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A fresh directory per test, deleted again at the end. The process id and a counter keep two
    /// tests running at once from colliding on the same name.
    fn temp_dir(who: &str) -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("pc10-launch-{who}-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        dir
    }

    /// Write a registration naming `exe`, and return the app.json path to hand to the code under test.
    fn write_app(dir: &Path, exe: &Path) -> PathBuf {
        let escaped = exe.display().to_string().replace('\\', "\\\\").replace('"', "\\\"");
        let app = dir.join(APP_FILE_NAME);
        std::fs::write(&app, format!("{{\"version\": 1, \"exe\": \"{escaped}\"}}")).expect("test app.json");
        app
    }

    fn exe_name() -> &'static str {
        if cfg!(windows) { "caim.exe" } else { "caim" }
    }

    #[test]
    fn the_command_is_the_registered_executable_with_open() {
        let dir = temp_dir("command");
        let exe = dir.join(exe_name());
        std::fs::write(&exe, "not a real program").expect("test executable");
        let app = write_app(&dir, &exe);
        let registration = read_registration(Some(&app)).expect("registration");
        let command = launch_command(&registration);
        assert_eq!(command.get_program(), exe.as_os_str());
        assert_eq!(command.get_args().collect::<Vec<&OsStr>>(), vec![OsStr::new("open")]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_registered_is_not_started() {
        let dir = temp_dir("nothing");
        let app = dir.join(APP_FILE_NAME);
        assert_eq!(open_registered(Some(&app)), Err(LaunchError::NotRegistered));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_registration_whose_executable_went_away_is_not_started() {
        let dir = temp_dir("gone");
        let app = write_app(&dir, &dir.join(exe_name()));
        assert_eq!(open_registered(Some(&app)), Err(LaunchError::NotRegistered));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn something_registered_that_cannot_be_started_reports_the_platform_words() {
        // Registered, readable, correctly named - but not something the platform will run. The
        // reason is whatever the platform said, so the test only checks that a reason arrived.
        let dir = temp_dir("failed");
        let exe = dir.join(exe_name());
        std::fs::write(&exe, "not a real program").expect("test executable");
        let app = write_app(&dir, &exe);
        match open_registered(Some(&app)) {
            Err(LaunchError::Failed(reason)) => assert!(!reason.is_empty()),
            other => panic!("expected a failed start, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Only Unix can start a script written by a test, so this is the one case where a real start
    /// is checked. On Windows the tests above cover the decision; nothing here pretends otherwise.
    #[cfg(unix)]
    #[test]
    fn a_registered_manager_is_started() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("started");
        let exe = dir.join(exe_name());
        std::fs::write(&exe, "#!/bin/sh\nexit 0\n").expect("test executable");
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o700)).expect("test mode");
        let app = write_app(&dir, &exe);
        open_registered(Some(&app)).expect("the registered manager starts");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
