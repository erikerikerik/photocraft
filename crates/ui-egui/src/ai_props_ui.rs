//! Properties' read-only "Made with" block for a layer an AI command made or masked (PC-11, U04):
//! the model, its version, the first 12 characters of its SHA-256 and the device that ran the job.
//! The record itself lives on the layer (`photocraft_engine::ai_provenance`); nothing here edits
//! it, and a layer without a record draws nothing at all, so the panel looks exactly as it did.

use egui::{Sense, pos2, vec2};
use photocraft_doc::Layer;

use crate::props_layout::{LABEL_GAP, LABEL_W, section};
use crate::theme::{ROW_GAP, Tokens};

/// The block, when the layer carries a record. Read-only: it never runs a command.
pub fn properties(ui: &mut egui::Ui, layer: &Layer) {
    let Some(record) = photocraft_engine::ai_provenance::read(layer) else { return };
    if !section(ui, "madeWith", tl!("Made with")) {
        return;
    }
    let t = Tokens::get(ui.ctx());
    // The SHA-256 is long and nobody reads it off a panel: show enough to tell two models apart and
    // put the whole value in the tooltip, which is how the panel already carries a long value.
    let sha = record.model_sha256.as_deref();
    let wanted = [
        ("Model", Some(record.model.as_str()), None),
        ("Version", record.model_version.as_deref(), None),
        ("SHA-256", sha.map(photocraft_engine::ai_provenance::short_sha), sha),
        ("Device", Some(record.device.as_str()), record.device_name.as_deref()),
    ];
    // A field the provider never reported gets no row at all, and one whose short form is the whole
    // value needs no tooltip.
    let rows: Vec<(std::sync::Arc<egui::Galley>, &str, Option<&str>)> = wanted
        .iter()
        .filter_map(|(label, value, tip)| {
            let value = value.filter(|value| !value.is_empty())?;
            let g = ui.painter().layout_no_wrap(tl!(label).to_owned(), egui::FontId::proportional(12.0), t.text_dim);
            Some((g, value, tip.filter(|tip| *tip != value)))
        })
        .collect();
    // One label column, as wide as the longest word, so the values line up the way the panel's
    // other label-then-value rows do.
    let label_w = rows.iter().fold(LABEL_W, |w, (g, _, _)| w.max(g.size().x));
    for (g, value, tip) in &rows {
        row(ui, g, value, *tip, label_w);
    }
    ui.add_space(ROW_GAP);
}

/// One read-only row: the dim label the panel's other rows use, then the value as selectable text so
/// it can be copied, with a tooltip when the row shows less than the whole value.
fn row(ui: &mut egui::Ui, label: &std::sync::Arc<egui::Galley>, value: &str, tip: Option<&str>, label_w: f32) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = LABEL_GAP;
        let (lr, _) = ui.allocate_exact_size(vec2(label_w, 18.0), Sense::hover());
        ui.painter().galley(pos2(lr.left(), lr.center().y - label.size().y / 2.0), label.clone(), t.text_dim);
        let resp = ui.add(egui::Label::new(value).selectable(true));
        if let Some(tip) = tip {
            resp.on_hover_text(tip.to_string());
        }
    });
}

#[cfg(test)]
#[path = "ai_props_ui/tests.rs"]
mod tests;
