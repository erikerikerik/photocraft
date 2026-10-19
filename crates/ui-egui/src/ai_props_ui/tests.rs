//! The "Made with" block appears only on a layer an AI command made or masked, and says what the
//! record holds. A layer without one draws the same panel as before.

use photocraft_engine::ai_provenance::{self, CMD_REMOVE_BACKGROUND, Record};
use serde_json::json;

use super::*;
use crate::PhotocraftApp;

/// A record for a Remove Background job, with every field the provider can report.
fn record() -> Record {
    Record {
        command: CMD_REMOVE_BACKGROUND.to_string(),
        model: "birefnet-subject".into(),
        model_version: Some("epoch_244".into()),
        model_sha256: Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c".into()),
        device: "CUDA".into(),
        device_name: Some("NVIDIA GeForce RTX 4090".into()),
    }
}

/// A Properties panel for a one-layer document whose layer carries `record`, if it has one.
fn harness(record: Option<&Record>) -> egui_kittest::Harness<'static, PhotocraftApp> {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 200, "height": 120})).unwrap();
    if let Some(record) = record {
        let id = app.session.active().unwrap().active_layer.unwrap();
        let record = record.clone();
        app.session.edit("record", move |doc, _| ai_provenance::write(doc.layer_mut(id).unwrap(), &record)).unwrap();
    }
    egui_kittest::Harness::builder().with_size(vec2(280.0, 900.0)).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            if ui.ctx().cumulative_pass_nr() == 0 {
                PhotocraftApp::setup_context(ui.ctx(), crate::theme::ThemeKind::ProMedium);
                return;
            }
            let state = app.session.active().unwrap();
            let layer = state.doc.layer(state.active_layer.unwrap()).unwrap().clone();
            crate::layer_props_ui::properties(app, ui, &layer);
        },
        app,
    )
}

fn drawn_text(h: &egui_kittest::Harness<'_, PhotocraftApp>) -> Vec<String> {
    fn collect(shape: &egui::Shape, text: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(shape) => text.push(shape.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, text)),
            _ => {}
        }
    }
    let mut text = Vec::new();
    for shape in &h.output().shapes {
        collect(&shape.shape, &mut text);
    }
    text
}

#[test]
fn only_a_layer_with_a_record_gets_the_block() {
    for (record, want) in [(None, false), (Some(record()), true)] {
        let mut h = harness(record.as_ref());
        h.run_steps(3);
        let sections = crate::props_layout::sections_drawn(&h.ctx);
        assert_eq!(sections.iter().any(|(title, _)| title == "Made with"), want, "{sections:?}");
    }
}

#[test]
fn the_block_shows_the_model_version_the_short_sha_and_the_device() {
    let mut h = harness(Some(&record()));
    h.run_steps(3);
    let text = drawn_text(&h);
    let shown = |want: &str| text.iter().any(|text| text == want);
    assert!(shown("birefnet-subject"), "the model id: {text:?}");
    assert!(shown("epoch_244"), "the model version: {text:?}");
    assert!(shown("58f621f00f5d"), "the first 12 characters of the SHA-256: {text:?}");
    assert!(!shown("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c"), "the whole hash is not on the panel");
    assert!(shown("CUDA"), "the device that ran the job: {text:?}");
    for label in ["Made with", "Model", "Version", "SHA-256", "Device"] {
        assert!(shown(label), "the row label {label}: {text:?}");
    }
}

#[test]
fn a_record_with_unknown_fields_shows_only_what_it_has() {
    let partial = Record {
        command: CMD_REMOVE_BACKGROUND.into(),
        model: "lama-fill".into(),
        model_version: None,
        model_sha256: None,
        device: "CPU".into(),
        device_name: None,
    };
    let mut h = harness(Some(&partial));
    h.run_steps(3);
    let text = drawn_text(&h);
    assert!(text.iter().any(|text| text == "lama-fill"));
    assert!(text.iter().any(|text| text == "CPU"));
    assert!(!text.iter().any(|text| text == "SHA-256"), "no row for a value nobody reported: {text:?}");
    assert!(!text.iter().any(|text| text == "Version"), "no row for a value nobody reported: {text:?}");
}

/// U10: the before/after picture of the panel. `cargo test -p photocraft-ui-egui ai_props -- --ignored`
/// writes these; they are for inspection, not compared by any test.
#[test]
#[ignore = "writes synthetic Properties screenshots for a layer with and without an AI record"]
fn write_made_with_snapshots() {
    let _gpu = photocraft_testkit::gpu_lock();
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plan/evidence/ai-provenance");
    std::fs::create_dir_all(&output).unwrap();
    for theme in crate::theme::ThemeKind::ALL {
        for (name, record) in [("no-record", None), ("made-with", Some(record()))] {
            let mut h = harness(record.as_ref());
            PhotocraftApp::setup_context(&h.ctx, theme);
            h.run_steps(3);
            h.render().unwrap().save(output.join(format!("{}-{name}.png", theme.id()))).unwrap();
        }
    }
}
