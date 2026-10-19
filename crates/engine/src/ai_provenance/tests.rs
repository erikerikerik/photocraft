//! The per-layer AI record: it is written and read back the same, it survives the native format
//! byte for byte, and anything that is not one of our records reads as no record at all.

use std::sync::Arc;

use photocraft_doc::{Layer, LayerId};
use serde_json::json;

use super::*;
use crate::Session;

/// A pixel layer to hold a record in.
fn raster(name: &str) -> Layer {
    Layer::raster(name, photocraft_color::PixelFormat::RGBA32F)
}

/// A record with every field filled in.
fn sample() -> Record {
    Record {
        command: CMD_GENERATIVE_FILL.to_string(),
        model: "lama-fill".into(),
        model_version: Some("epoch_244".into()),
        model_sha256: Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c".into()),
        device: "CUDA".into(),
        device_name: Some("NVIDIA GeForce RTX 4090".into()),
    }
}

/// A one-layer document, and its layer.
fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 8, "height": 8, "background": "white"})).unwrap();
    s
}

fn layer_id(s: &Session) -> LayerId {
    s.active().unwrap().active_layer.unwrap()
}

/// Put `record` on the document's layer, as one history step.
fn put(s: &mut Session, record: &Record) {
    let id = layer_id(s);
    let record = record.clone();
    s.edit("record", move |doc, _| write(doc.layer_mut(id).ok_or(crate::EngineError::NoLayer(id))?, &record)).unwrap();
}

#[test]
fn a_record_reads_back_exactly_as_it_was_written() {
    let mut layer = raster("Generative Fill");
    let want = sample();
    write(&mut layer, &want).unwrap();
    assert_eq!(read(&layer).as_ref(), Some(&want));
}

#[test]
fn writing_again_replaces_the_record_rather_than_adding_a_second_one() {
    let mut layer = raster("Generative Fill");
    write(&mut layer, &sample()).unwrap();
    let newer = Record { model: "lama-fill-v2".into(), ..sample() };
    write(&mut layer, &newer).unwrap();
    assert_eq!(layer.psd_blocks.iter().filter(|(key, _)| *key == KEY).count(), 1, "one record per layer");
    assert_eq!(read(&layer).as_ref(), Some(&newer));
}

#[test]
fn a_layer_with_no_record_reads_as_none() {
    let layer = raster("Background");
    assert_eq!(read(&layer), None);
}

#[test]
fn a_block_under_our_key_that_is_not_ours_is_no_record() {
    let mut layer = raster("From a PSD");
    layer.psd_blocks.push((KEY, Arc::new(b"not json at all".to_vec())));
    assert_eq!(read(&layer), None);
}

#[test]
fn a_record_with_fields_this_version_does_not_know_still_loads() {
    let mut s = session();
    let id = layer_id(&s);
    let future = br#"{"command":"edit.generativeFill","model":"lama-fill","device":"CUDA","model_version":"epoch_999","nextTime":{"runsOn":"Neural Engine"}}"#;
    s.edit("future record", move |doc, _| {
        doc.layer_mut(id).ok_or(crate::EngineError::NoLayer(id))?.psd_blocks.push((KEY, Arc::new(future.to_vec())));
        Ok(())
    })
    .unwrap();
    let got = read(s.active().unwrap().doc.layer(id).unwrap()).unwrap();
    assert_eq!(got.model, "lama-fill");
    assert_eq!(got.model_version.as_deref(), Some("epoch_999"));
    assert_eq!(got.model_sha256, None, "a field this version never stored stays unknown");
}

#[test]
fn the_record_survives_the_native_format_byte_for_byte() {
    let mut s = session();
    let id = layer_id(&s);
    put(&mut s, &sample());
    let stored = |l: &Layer| l.psd_blocks.iter().find(|(key, _)| *key == KEY).map(|(_, data)| data.clone()).unwrap();
    let before = s.active().unwrap().doc.clone();
    let bytes = photocraft_format::save_to_bytes(&before, &Default::default()).unwrap();
    let back = photocraft_format::load_from_bytes(&bytes).unwrap();
    let again = back.layer(id).unwrap();
    assert_eq!(read(again), read(before.layer(id).unwrap()));
    assert_eq!(stored(again), stored(before.layer(id).unwrap()), "the same bytes, not a re-encode");
}

#[test]
fn a_document_with_no_records_saves_and_loads_exactly_as_before() {
    let s = session();
    let before = s.active().unwrap().doc.clone();
    let bytes = photocraft_format::save_to_bytes(&before, &Default::default()).unwrap();
    let back = photocraft_format::load_from_bytes(&bytes).unwrap();
    for (_, _, layer) in back.walk() {
        assert!(layer.psd_blocks.is_empty(), "{} carries a block it never had", layer.name);
        assert_eq!(read(layer), None);
    }
}

#[test]
fn duplicating_a_layer_copies_its_record_without_special_code() {
    let mut layer = raster("Generative Fill");
    write(&mut layer, &sample()).unwrap();
    assert_eq!(read(&layer.duplicate()), read(&layer));
}

#[test]
fn short_sha_keeps_the_first_twelve_characters() {
    assert_eq!(short_sha("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c"), "58f621f00f5d");
    assert_eq!(short_sha("58f621f00f5d"), "58f621f00f5d");
    assert_eq!(short_sha("short"), "short");
}

/// The record a PSD round trip gives back for `record`: export the document, read the file again,
/// and take the record off the layer that came back.
fn psd_round_trip(record: &Record) -> Option<Record> {
    let mut s = session();
    put(&mut s, record);
    let doc = s.active().unwrap().doc.clone();
    let bytes = photocraft_io::export(&doc, "x.psd", &Default::default()).unwrap().bytes;
    let back = photocraft_io::import("x.psd", &bytes).unwrap().document;
    back.walk().into_iter().find_map(|(_, _, layer)| read(layer))
}

#[test]
fn a_record_survives_a_psd_round_trip_whatever_its_length() {
    // A PSD keeps an odd-length block's pad byte inside the length, so a record that serializes to
    // an odd number of bytes comes back with a trailing NUL. Both parities are covered on purpose.
    let mut parities = [false; 2];
    for version in ["epoch_244", "epoch_2444"] {
        let record = Record { model_version: Some(version.into()), ..sample() };
        let len = serde_json::to_vec(&record).unwrap().len();
        parities[len % 2] = true;
        let back = psd_round_trip(&record).unwrap_or_else(|| panic!("a record that serialized to {len} bytes is lost by a PSD round trip"));
        assert_eq!(back, record, "a {len} byte record must survive a PSD round trip");
    }
    assert!(parities[0] && parities[1], "the two records must cover both parities, got {parities:?}");
}

#[test]
fn a_psd_pad_byte_inside_the_block_does_not_hide_the_record() {
    let mut layer = raster("Generative Fill");
    let mut json = serde_json::to_vec(&sample()).unwrap();
    json.push(0);
    write(&mut layer, &sample()).unwrap();
    let slot = layer.psd_blocks.iter_mut().find(|(key, _)| *key == KEY).unwrap();
    slot.1 = Arc::new(json);
    assert_eq!(read(&layer), Some(sample()), "the pad byte a PSD adds inside an odd-length block is not part of the record");
}

#[test]
fn a_block_under_our_key_that_is_not_ours_is_kept_and_our_record_goes_beside_it() {
    let mut layer = raster("Generative Fill");
    let foreign = b"another program's block, not one of our records".to_vec();
    layer.psd_blocks.push((KEY, Arc::new(foreign.clone())));
    let want = sample();
    write(&mut layer, &want).unwrap();
    assert_eq!(read(&layer).as_ref(), Some(&want), "our record is the one that reads back");
    let ours = |l: &Layer| l.psd_blocks.iter().filter(|(key, _)| *key == KEY).filter(|(_, d)| parse(d).is_some()).count();
    assert_eq!(ours(&layer), 1, "one record per layer, beside the foreign block");
    assert!(layer.psd_blocks.iter().any(|(_, d)| d.as_slice() == foreign.as_slice()), "the other program's bytes stay verbatim");
    let newer = Record { model: "lama-fill-v2".into(), ..sample() };
    write(&mut layer, &newer).unwrap();
    assert_eq!(layer.psd_blocks.iter().filter(|(key, _)| *key == KEY).count(), 2, "writing again replaces our record, it does not add a third block");
    assert_eq!(read(&layer).as_ref(), Some(&newer));
    assert!(layer.psd_blocks.iter().any(|(_, d)| d.as_slice() == foreign.as_slice()));
}
