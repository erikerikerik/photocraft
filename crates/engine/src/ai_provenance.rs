//! Provenance for a layer an AI command made or masked (SPEC amendment 18, B16): the model id,
//! its version, the SHA-256 of the distributed model file and the device that ran the job. The
//! record travels with the layer, so an AI layer always says what made it, and a newer model
//! version never rewrites what an older one left on a different layer.
//!
//! It lives in [`Layer::psd_blocks`] under one four-character key, which is the only free-form
//! per-layer field PhotoCraft has and the only one the native format already saves and loads
//! (`crates/format/src/manifest.rs`, `crates/format/src/convert.rs`), so no document-format change
//! was needed: an older PhotoCraft sees one more unknown tagged block and ignores it, and a PSD
//! keeps it as one. A layer with no record is the normal case, never an error.

use std::sync::Arc;

use photocraft_doc::Layer;
use serde::{Deserialize, Serialize};

use crate::ai::AiResult;
use crate::{EngineError, Result};

/// The key the record is stored under. Four ASCII characters because that is what a tagged block
/// key is; lower case, so it cannot collide with a Photoshop key.
pub const KEY: [u8; 4] = *b"pcai";

/// `edit.generativeFill`, which makes a layer.
pub const CMD_GENERATIVE_FILL: &str = "edit.generativeFill";
/// `layer.removeBackground`, which masks one.
pub const CMD_REMOVE_BACKGROUND: &str = "layer.removeBackground";

/// What made a layer: only what the provider reported. A field it left out stays unknown, which is
/// why everything except the ids every job reports is optional.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// The command that made or masked the layer, by its command id.
    pub command: String,
    /// The model's id, as the manager's manifest names it.
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// SHA-256 of the distributed model file (amendment 18). A compiled or runtime cache is a
    /// disposable implementation artifact and never appears here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_sha256: Option<String>,
    /// The backend that ran the job.
    pub device: String,
    /// The hardware it ran on, when the provider named it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
}

impl Record {
    /// The record for one finished job, as the provider reported it.
    pub fn from_result(command: &str, result: &AiResult) -> Self {
        let p = &result.provenance;
        Record {
            command: command.to_string(),
            model: result.model.clone(),
            model_version: p.model_version.clone(),
            model_sha256: p.model_sha256.clone(),
            device: result.device.clone(),
            device_name: p.device_name.clone(),
        }
    }
}

/// A SHA-256 shortened for a panel: its first 12 characters, or the whole value when it is shorter
/// than that (a provider could report anything).
pub fn short_sha(sha: &str) -> &str {
    sha.get(..12).unwrap_or(sha)
}

/// Store `record` on `layer`, replacing the record the layer already had. Only the layer a job made
/// or masked gets one; nothing here looks at another layer.
///
/// A block under our key that does not read back as one of our records belongs to whatever program
/// wrote it, so it stays byte for byte (AGENTS.md section 8: unknown PSD blocks are kept verbatim)
/// and ours goes in after it.
pub fn write(layer: &mut Layer, record: &Record) -> Result<()> {
    let json = serde_json::to_vec(record).map_err(|e| EngineError::Other(format!("could not store what made the layer: {e}")))?;
    let block = Arc::new(json);
    match layer.psd_blocks.iter_mut().find(|(key, data)| *key == KEY && parse(data).is_some()) {
        Some((_, slot)) => *slot = block,
        None => layer.psd_blocks.push((KEY, block)),
    }
    Ok(())
}

/// The record a layer carries, or `None` when it carries none. A block under our key that is not
/// ours - one another program wrote into a PSD, or a file from a newer PhotoCraft with fields this
/// one does not know - reads as no record rather than as an error.
pub fn read(layer: &Layer) -> Option<Record> {
    layer.psd_blocks.iter().filter(|(key, _)| *key == KEY).find_map(|(_, data)| parse(data))
}

/// One block's bytes as a record, or `None` when they are not one. A PSD keeps a layer block's pad
/// byte inside the length (`crates/io/src/psd_export.rs`), so a record that serialized to an odd
/// number of bytes comes back with a trailing NUL; JSON never ends with one, so the pad is dropped
/// before parsing.
fn parse(data: &[u8]) -> Option<Record> {
    match data.iter().rposition(|&byte| byte != 0) {
        Some(last) => serde_json::from_slice(&data[..=last]).ok(),
        None => None,
    }
}

#[cfg(test)]
#[path = "ai_provenance/tests.rs"]
mod tests;
