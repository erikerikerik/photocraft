//! PC-10: the one sentence, and at most one action, for an AI command that needs Craft AI Manager
//! (SPEC amendment 18, FEATURES B04/U11).
//!
//! The shell decides nothing about the manager itself. It asks the native app which state applies -
//! the app owns the filesystem and process seams - and shows what amendment 18 allows. Where there
//! is no seam, the web build, the CLI and a plain test, the engine's own sentence stands and no
//! button appears. Nothing here reads an error message to work out what went wrong: the command's
//! id and parameters say whether the manager is involved, and the app's answer says why.

use photocraft_engine::ai::AiTask;
use serde::{Deserialize, Serialize};

use crate::PhotocraftApp;
use crate::notices::{self, NoticeAction};

/// How far the shell can tell what the local manager is doing, as the native app reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaimState {
    /// Nothing usable is registered: the installed sentence, and no action.
    NotInstalled,
    /// Registered but not answering: the running sentence, plus the launch action.
    NotRunning,
    /// Running, but it does not offer the task the command needs: the engine's own missing-model
    /// sentence, plus the launch action.
    ModelMissing,
    /// Running and offering the task, or answering something other than "not running": the failure
    /// has another cause, so the engine's sentence stands and no launch button appears.
    NoLaunchAction,
}

/// Why starting the manager did not happen, as the native app reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenCaimError {
    /// The registration, or the executable it names, went away between showing the button and
    /// pressing it. The same sentence as never having one, and no further action.
    NotInstalled,
    /// Something is registered and the platform still refused to start it. `reason` is the
    /// platform's own words, shown as a line under the sentence rather than as the sentence.
    Failed(String),
}

/// The task a failed command needed, when that command needs Craft AI Manager at all.
///
/// The set is closed and comes from the command's id and parameters, never from the error text:
/// Generative Fill always needs the manager, and Select Subject or Remove Background only when the
/// caller asked for `"method": "ai"`. Everything else - a classical selection, a bad parameter, a
/// missing layer, a print job - keeps its own message and gets no button.
pub fn needed_task(id: &str, params: &serde_json::Value) -> Option<AiTask> {
    match id {
        "edit.generativeFill" => Some(AiTask::FillMaskedArea),
        "select.subject" | "layer.removeBackground" if params.get("method").and_then(serde_json::Value::as_str) == Some("ai") => Some(AiTask::SubjectMask),
        _ => None,
    }
}

/// Report a failed command. `task` is [`needed_task`] decided when the command ran; `message` is the
/// sentence the engine already produced.
pub fn report_failure(app: &mut PhotocraftApp, task: Option<AiTask>, message: &str) {
    let Some(task) = task else {
        notices::error(app, message.to_string());
        return;
    };
    // Read live, at the moment of the failure: a manager started in the meantime is not reported as
    // missing, and one closed since is not reported as running.
    let state = match app.services.caim_state.as_mut() {
        Some(read) => read(task),
        // No seam: the engine's sentence is the whole story.
        None => {
            notices::error(app, message.to_string());
            return;
        }
    };
    match state {
        CaimState::NotInstalled => {
            app.ui.status = tl!("Craft AI Manager isn't installed.").into();
            app.ui.status_error = true;
            notices::post(app, "Craft AI Manager isn't installed.", Vec::new(), true, None);
        }
        CaimState::NotRunning => {
            app.ui.status = tl!("Craft AI Manager isn't running.").into();
            app.ui.status_error = true;
            notices::post_action(app, "Craft AI Manager isn't running.", Vec::new(), true, NoticeAction::OpenCaim);
        }
        CaimState::ModelMissing => {
            // The feature's own sentence already names the model, so it stays; the button is the
            // part worth adding.
            app.ui.status = message.to_string();
            app.ui.status_error = true;
            notices::post_action(app, message.to_string(), Vec::new(), true, NoticeAction::OpenCaim);
        }
        CaimState::NoLaunchAction => notices::error(app, message.to_string()),
    }
}

/// The button was pressed: ask the app to start the registered manager. Nothing happens where there
/// is no seam, and nothing is claimed unless the platform actually started a process.
pub fn open_pressed(app: &mut PhotocraftApp) {
    let outcome = match app.services.open_caim.as_mut() {
        Some(open) => open(),
        None => return,
    };
    match outcome {
        Ok(()) => {
            // Starting the manager is not the same as the model being installed, and the notice says
            // so: the AI command still has to be retried once the manager is ready.
            let lines = vec!["Try the AI command again once it is ready. Starting the manager does not install a model.".to_owned()];
            app.ui.status = tl!("Craft AI Manager is starting").into();
            app.ui.status_error = false;
            notices::post(app, "Craft AI Manager is starting", lines, false, None);
        }
        Err(OpenCaimError::NotInstalled) => {
            app.ui.status = tl!("Craft AI Manager isn't installed.").into();
            app.ui.status_error = true;
            notices::post(app, "Craft AI Manager isn't installed.", Vec::new(), true, None);
        }
        Err(OpenCaimError::Failed(reason)) => {
            app.ui.status = tl!("PhotoCraft could not start Craft AI Manager.").into();
            app.ui.status_error = true;
            notices::post(app, "PhotoCraft could not start Craft AI Manager.", vec![reason], true, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use photocraft_engine::Session;

    /// An app with no platform services at all: the shape the web build and a plain test see.
    fn app_fixture() -> PhotocraftApp {
        PhotocraftApp::new(Session::new(), Services::default())
    }

    /// A command needs the manager only for the AI method, decided from the command rather than
    /// from anything the error said.
    #[test]
    fn only_ai_commands_need_the_manager() {
        assert_eq!(needed_task("edit.generativeFill", &serde_json::json!({})), Some(AiTask::FillMaskedArea));
        assert_eq!(needed_task("select.subject", &serde_json::json!({"method": "ai"})), Some(AiTask::SubjectMask));
        assert_eq!(needed_task("layer.removeBackground", &serde_json::json!({"method": "ai"})), Some(AiTask::SubjectMask));
        for (id, params) in [
            ("select.subject", serde_json::json!({})),
            ("select.subject", serde_json::json!({"method": "classical"})),
            ("layer.removeBackground", serde_json::json!({"method": "classical"})),
            ("edit.contentAwareFill", serde_json::json!({})),
            ("file.save", serde_json::json!({})),
        ] {
            assert_eq!(needed_task(id, &params), None, "{id} {params} does not need the manager");
        }
    }

    /// The four states produce exactly the sentences and actions amendment 18 asks for: one
    /// sentence each, at most one action, and no button for a failure that is not the manager's.
    #[test]
    fn each_state_gives_its_own_sentence_and_action() {
        let engine = "The Select Subject model isn't installed. Open Craft AI Manager to add it.";
        let cases = [
            (CaimState::NotInstalled, "Craft AI Manager isn't installed.", false),
            (CaimState::NotRunning, "Craft AI Manager isn't running.", true),
            (CaimState::ModelMissing, engine, true),
            (CaimState::NoLaunchAction, engine, false),
        ];
        for (state, expected, offers) in cases {
            let mut app = app_fixture();
            app.services.caim_state = Some(Box::new(move |_| state));
            report_failure(&mut app, Some(AiTask::SubjectMask), engine);
            let notice = app.ui.notices.last().expect("the failure leaves a notice");
            assert_eq!(notice.title, expected, "{state:?} says its own sentence");
            assert_eq!(notice.action == Some(NoticeAction::OpenCaim), offers, "{state:?} offers the launch button exactly when it should");
            assert_eq!(app.ui.status, expected, "{state:?} says the same thing in the status bar");
            assert!(app.ui.status_error, "{state:?} is an error");
            assert_eq!(app.ui.notices.len(), 1, "{state:?} leaves exactly one notice");
        }
    }

    /// Without the native seam - the web build, the CLI, a test - an AI failure is reported exactly
    /// as it was before PC-10: the engine's sentence, and no button.
    #[test]
    fn no_seam_means_no_button_and_the_engine_sentence() {
        let mut app = app_fixture();
        report_failure(&mut app, Some(AiTask::SubjectMask), "the engine's own sentence");
        let notice = app.ui.notices.last().expect("the failure still leaves a notice");
        assert_eq!(notice.title, "the engine's own sentence");
        assert_eq!(notice.action, None, "nothing can be offered without the native seam");
    }

    /// An unrelated failure never picks up a launch button, whatever the manager's state is.
    #[test]
    fn an_unrelated_failure_keeps_its_own_message() {
        for state in [CaimState::NotRunning, CaimState::NotInstalled] {
            let mut app = app_fixture();
            app.services.caim_state = Some(Box::new(move |_| state));
            report_failure(&mut app, None, "no active layer");
            let notice = app.ui.notices.last().expect("the failure still leaves a notice");
            assert_eq!(notice.title, "no active layer");
            assert_eq!(notice.action, None, "{state:?} is not the reason this command failed");
        }
    }

    /// Pressing the button goes through the seam exactly once, and only the seam's answer decides
    /// what is claimed. A failed start says so; it never pretends the manager is coming.
    #[test]
    fn the_button_calls_the_seam_and_reports_what_happened() {
        let mut app = app_fixture();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let counted = calls.clone();
        app.services.open_caim = Some(Box::new(move || {
            counted.set(counted.get() + 1);
            Ok(())
        }));
        open_pressed(&mut app);
        assert_eq!(calls.get(), 1, "one press, one call");
        let notice = app.ui.notices.last().expect("a start is reported");
        assert_eq!(notice.title, "Craft AI Manager is starting");
        assert_eq!(notice.action, None, "the notice that says a manager is starting offers nothing else");
        assert!(notice.lines.iter().any(|line| line.contains("does not install a model")), "and it says a start is not an install: {notice:?}");
        assert!(!app.ui.status_error, "a start is not an error");

        let mut app = app_fixture();
        app.services.open_caim = Some(Box::new(|| Err(OpenCaimError::Failed("the file could not be run".into()))));
        open_pressed(&mut app);
        let notice = app.ui.notices.last().expect("a failed start is reported too");
        assert_eq!(notice.title, "PhotoCraft could not start Craft AI Manager.");
        assert_eq!(notice.lines, vec!["the file could not be run".to_owned()], "the platform's own words are a line, not the sentence");
        assert_eq!(notice.action, None, "a failed start does not offer the button again");
        assert!(app.ui.status_error, "a failed start is an error");

        // No seam at all: pressing cannot start anything.
        let mut app = app_fixture();
        open_pressed(&mut app);
        assert!(app.ui.notices.is_empty(), "with no seam the press does nothing");
    }

    /// The action survives the control channel's view of notices, so `ui.inspect` shows which
    /// sentence offered which button and automation can press it.
    #[test]
    fn the_action_is_visible_through_the_control_channel() {
        let mut app = app_fixture();
        app.services.caim_state = Some(Box::new(|_| CaimState::NotRunning));
        report_failure(&mut app, Some(AiTask::SubjectMask), "unused");
        let json = serde_json::to_value(&app.ui.notices).expect("notices serialize");
        let action = json[0].get("action").and_then(serde_json::Value::as_str);
        assert_eq!(action, Some("OpenCaim"), "ui.inspect shows the action");
    }
}
