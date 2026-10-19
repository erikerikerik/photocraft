//! Generative Fill: the provider result lands on a new layer, the document is untouched on every
//! failure, and the errors are plain sentences.

use std::sync::Arc;

use photocraft_doc::LayerId;
use photocraft_geom::Rect;
use serde_json::json;

use super::*;
use crate::ai::testing::FakeProvider;
use crate::jobs::Started;

/// A 64×48 document of `mode` at `depth` with a selection around the middle.
fn doc_session(depth: u32, mode: &str) -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 64, "height": 48, "depth": depth, "mode": mode, "background": "white"})).unwrap();
    s.execute("select.rect", json!({"x": 26, "y": 16, "width": 12, "height": 14})).unwrap();
    s
}

/// The Content-Aware Fill fixture: white canvas, vertical black stripes, a red blob to remove.
fn blob_session(depth: u32) -> Session {
    let mut s = doc_session(depth, "rgb");
    s.edit("stripes", |doc, active| {
        let surf = doc.layer_mut(active.unwrap()).unwrap().surface_mut().unwrap();
        for x in (0..64).step_by(8) {
            surf.fill_rect(Rect::new(x, 0, x + 3, 48), &[0.0, 0.0, 0.0, 1.0]);
        }
        surf.fill_rect(Rect::new(28, 18, 36, 28), &[1.0, 0.0, 0.0, 1.0]);
        Ok(())
    })
    .unwrap();
    s
}

/// Install `provider` and hand back a handle on it, so a test can read what the provider saw.
fn use_provider(s: &mut Session, provider: FakeProvider) -> Arc<FakeProvider> {
    let provider = Arc::new(provider);
    s.set_ai_provider(Some(provider.clone()));
    provider
}

fn active_id(s: &Session) -> LayerId {
    s.active().unwrap().active_layer.unwrap()
}

fn name_of(s: &Session, id: LayerId) -> String {
    s.active().unwrap().doc.layer(id).unwrap().name.clone()
}

fn named(s: &Session, name: &str) -> Option<LayerId> {
    s.active().unwrap().doc.walk().into_iter().find(|(_, _, l)| l.name == name).map(|(_, _, l)| l.id)
}

/// Every sample of a layer over the canvas, in row order.
fn snapshot(s: &Session, id: LayerId) -> Vec<f32> {
    let st = s.active().unwrap();
    let doc = &st.doc;
    let surf = doc.layer(id).unwrap().surface().unwrap().clone();
    let b = doc.bounds();
    let mut out = Vec::new();
    for y in b.y0..b.y1 {
        for x in b.x0..b.x1 {
            out.extend(surf.pixel(x, y));
        }
    }
    out
}

/// Alpha of one pixel of a layer (0.0 where it has no pixels).
fn alpha_at(s: &Session, id: LayerId, x: i32, y: i32) -> f32 {
    let st = s.active().unwrap();
    match st.doc.layer(id).unwrap().surface() {
        Some(surf) => surf.pixel(x, y).last().copied().unwrap_or(0.0),
        None => 0.0,
    }
}

#[test]
fn the_result_lands_on_a_new_layer_named_generative_fill_and_the_source_is_untouched() {
    let mut s = blob_session(8);
    let base = active_id(&s);
    let before = snapshot(&s, base);
    let (layers, steps) = (s.active().unwrap().doc.layer_count(), s.active().unwrap().history.past_len());
    use_provider(&mut s, FakeProvider { polls: 1, ..Default::default() });
    let r = s.execute(CMD, json!({})).unwrap();
    assert!(r["filled"].as_u64().unwrap() > 0, "{r}");
    let new = active_id(&s);
    assert_ne!(new, base, "a new layer, not the source");
    assert_eq!(r["layer"].as_u64().unwrap(), new.0, "the result names the new layer");
    assert_eq!(name_of(&s, new), "Generative Fill", "named exactly as D1 decided");
    assert_eq!(named(&s, "Generative Fill"), Some(new));
    // Directly above the layer that was active.
    let order: Vec<LayerId> = s.active().unwrap().doc.walk().into_iter().map(|(_, _, l)| l.id).collect();
    let (at, above) = (order.iter().position(|i| *i == new).unwrap(), order.iter().position(|i| *i == base).unwrap());
    assert_eq!(at, above + 1, "inserted above the active layer");
    // B01: the source pixels are byte-identical, and only one layer and one history step appeared.
    assert_eq!(snapshot(&s, base), before, "the source layer's pixels never change");
    assert_eq!(s.active().unwrap().doc.layer_count(), layers + 1);
    assert_eq!(s.active().unwrap().history.past_len(), steps + 1, "one undo step");
    // The fill really did something inside the selection.
    let filled = s.active().unwrap().doc.layer(new).unwrap().surface().unwrap().pixel(32, 23);
    assert!((filled[0] - 0.75).abs() < 0.02, "the provider's grey landed: {filled:?}");
}

/// B08: the result says where the work ran, taken from the provider's `AiResult` provenance.
#[test]
fn the_result_says_which_model_ran_the_fill_and_where() {
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider::default());
    let r = s.execute(CMD, json!({})).unwrap();
    let ai = r["ai"].as_object().unwrap();
    assert_eq!(ai["model"].as_str().unwrap(), "solid-fill", "the model that ran: {r}");
    assert_eq!(ai["device"].as_str().unwrap(), "CPU", "the device it ran on: {r}");
    assert_eq!(ai["provider"].as_str().unwrap(), "local fake", "where it ran: {r}");
    assert_eq!(ai["elapsedMs"].as_u64().unwrap(), 12, "how long it took: {r}");
    assert_eq!(ai.len(), 4, "exactly the four provenance fields: {ai:?}");
    // The background job carries the same provenance, so the control channel and MCP see it too.
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider::default());
    let id = match s.start(CMD, json!({})).unwrap() {
        Started::Job(id) => id,
        Started::Done(v) => panic!("expected a background job: {v}"),
    };
    let job = s.wait_job(id).unwrap();
    assert_eq!(job["ai"]["model"].as_str().unwrap(), "solid-fill", "{job}");
    assert_eq!(job["ai"]["provider"].as_str().unwrap(), "local fake", "{job}");
}

#[test]
fn one_undo_removes_the_generative_layer() {
    let mut s = blob_session(8);
    let base = active_id(&s);
    let (layers, steps) = (s.active().unwrap().doc.layer_count(), s.active().unwrap().history.past_len());
    use_provider(&mut s, FakeProvider::default());
    s.execute(CMD, json!({})).unwrap();
    assert_eq!(s.active().unwrap().doc.layer_count(), layers + 1);
    assert!(s.undo(), "one undo step");
    let st = s.active().unwrap();
    assert_eq!(st.doc.layer_count(), layers);
    assert_eq!(st.active_layer, Some(base), "the source layer is active again");
    assert_eq!(st.history.past_len(), steps);
    assert!(named(&s, "Generative Fill").is_none(), "the new layer is gone");
}

#[test]
fn a_second_run_numbers_the_layer_instead_of_reusing_the_name() {
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider::default());
    s.execute(CMD, json!({})).unwrap();
    s.execute(CMD, json!({})).unwrap();
    assert!(named(&s, "Generative Fill").is_some());
    assert!(named(&s, "Generative Fill 2").is_some(), "no two layers share a name");
}

#[test]
fn pixels_outside_the_selection_stay_transparent_and_a_feather_blends_by_coverage() {
    // Hard-edged selection: the new layer is transparent outside it, opaque inside.
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider::default());
    s.execute(CMD, json!({})).unwrap();
    let new = active_id(&s);
    for (x, y) in [(2, 2), (25, 23), (40, 23), (32, 8), (32, 35)] {
        assert_eq!(alpha_at(&s, new, x, y), 0.0, "outside the selection at ({x},{y})");
    }
    for (x, y) in [(27, 17), (37, 29), (32, 23)] {
        assert_eq!(alpha_at(&s, new, x, y), 1.0, "inside the selection at ({x},{y})");
    }
    // Feathered selection: the edge is partial coverage, so the fill blends in rather than cutting.
    let mut s = blob_session(8);
    s.execute("select.modify.feather", json!({"radius": 4})).unwrap();
    use_provider(&mut s, FakeProvider::default());
    s.execute(CMD, json!({})).unwrap();
    let new = active_id(&s);
    let edge = alpha_at(&s, new, 25, 23);
    assert!(edge > 0.0 && edge < 1.0, "the feathered edge blends by coverage: {edge}");
    assert_eq!(alpha_at(&s, new, 2, 2), 0.0, "still nothing far outside");
    assert!(alpha_at(&s, new, 32, 23) > 0.7, "still mostly filled in the middle");
    let partial = (16..30).map(|y| alpha_at(&s, new, 25, y)).filter(|a| *a > 0.0 && *a < 1.0).count();
    assert!(partial > 0, "a band of partial coverage, not one row");
}

#[test]
fn a_missing_or_off_canvas_selection_is_refused_in_content_aware_fill_s_words() {
    for rect in [None, Some(json!({"x": 200, "y": 200, "width": 8, "height": 8})), Some(json!({"x": -60, "y": -60, "width": 20, "height": 20}))] {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 64, "height": 48, "background": "white"})).unwrap();
        if let Some(rect) = &rect {
            s.execute("select.rect", rect.clone()).ok();
        }
        use_provider(&mut s, FakeProvider::default());
        let generative = s.execute(CMD, json!({}));
        let content_aware = s.execute("edit.contentAwareFill", json!({}));
        assert!(generative.is_err(), "{rect:?}: {generative:?}");
        assert!(content_aware.is_err(), "{rect:?}: {content_aware:?}");
        // Same reason; the command id in the message is the only difference.
        let reason = |result: &Result<serde_json::Value>| match result {
            Err(EngineError::Disabled(_, why)) => why.clone(),
            other => format!("{other:?}"),
        };
        assert_eq!(reason(&generative), reason(&content_aware), "{rect:?}");
    }
}

#[test]
fn without_a_provider_the_message_names_the_manager() {
    let mut s = blob_session(8);
    assert_eq!(s.execute(CMD, json!({})).unwrap_err().to_string(), "Craft AI Manager isn't running");
    // B04: the menu item stays enabled without a provider, so the user gets the message.
    assert!(s.is_enabled(CMD), "not greyed out");
    s.execute("select.deselect", json!({})).unwrap();
    assert!(!s.is_enabled(CMD), "but greyed out with no selection");
}

#[test]
fn a_prompt_is_refused_until_the_provider_supports_one() {
    for prompt in [json!("a cat"), json!(" "), json!("   \n")] {
        let mut s = blob_session(8);
        use_provider(&mut s, FakeProvider::default());
        let want = if prompt.as_str().unwrap_or_default().trim().is_empty() { Ok(()) } else { Err(()) };
        let got = s.execute(CMD, json!({"prompt": prompt})).map(|_| ());
        match want {
            Ok(()) => assert!(got.is_ok(), "blank prompt {prompt} runs: {got:?}"),
            Err(()) => assert_eq!(got.unwrap_err().to_string(), "Prompts aren't supported yet; leave the prompt empty to remove the selection", "{prompt}"),
        }
    }
    // A prompt of the wrong type names the parameter.
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider::default());
    assert!(s.execute(CMD, json!({"prompt": 7})).unwrap_err().to_string().contains("`prompt`"));
    // Null and absent are the same as no prompt at all.
    for p in [json!({}), json!({"prompt": null}), json!({"prompt": ""})] {
        let mut s = blob_session(8);
        use_provider(&mut s, FakeProvider::default());
        assert!(s.execute(CMD, p.clone()).is_ok(), "{p}");
    }
}

#[test]
fn non_rgb_documents_are_refused_at_8_and_16_bit() {
    for mode in ["gray", "cmyk", "lab"] {
        for depth in [8, 16] {
            let mut s = doc_session(depth, mode);
            use_provider(&mut s, FakeProvider::default());
            let e = s.execute(CMD, json!({})).unwrap_err();
            assert_eq!(e.to_string(), "Generative Fill works on RGB documents for now", "{mode} at {depth}-bit");
            let st = s.active().unwrap();
            assert_eq!(st.doc.layer_count(), 1, "{mode} at {depth}-bit added a layer");
        }
    }
}

#[test]
fn rgb_works_at_8_16_and_32_bit() {
    for depth in [8, 16, 32] {
        let mut s = blob_session(depth);
        use_provider(&mut s, FakeProvider::default());
        let r = s.execute(CMD, json!({})).unwrap();
        assert!(r["filled"].as_u64().unwrap() > 0, "{depth}-bit: {r}");
        assert_eq!(name_of(&s, active_id(&s)), "Generative Fill", "{depth}-bit");
    }
}

#[test]
fn a_bad_provider_result_is_an_error_and_the_document_is_unchanged() {
    let cases = [
        (FakeProvider { fail: Some("The model ran out of memory".into()), ..Default::default() }, "The model ran out of memory"),
        (FakeProvider { wrong_size: true, ..Default::default() }, "does not match the requested window"),
        (FakeProvider { wrong_format: true, ..Default::default() }, "does not match the requested window"),
        (FakeProvider { bad_pixel: true, ..Default::default() }, "The AI result contains invalid pixel values"),
    ];
    for (provider, want) in cases {
        let mut s = blob_session(8);
        let base = active_id(&s);
        let before = snapshot(&s, base);
        let (layers, steps) = (s.active().unwrap().doc.layer_count(), s.active().unwrap().history.past_len());
        let provider = use_provider(&mut s, provider);
        let e = s.execute(CMD, json!({})).unwrap_err();
        assert!(e.to_string().contains(want), "{want}: {e}");
        let st = s.active().unwrap();
        assert_eq!(st.doc.layer_count(), layers, "{want}: a layer appeared");
        assert_eq!(st.history.past_len(), steps, "{want}: history changed");
        assert_eq!(snapshot(&s, base), before, "{want}: the source layer changed");
        assert!(named(&s, "Generative Fill").is_none(), "{want}: the layer was named anyway");
        // The provider took the job, so it must be told to drop it, exactly once.
        assert_eq!(provider.state.lock().unwrap().cancels, 1, "{want}: the job must be cancelled on the provider exactly once");
    }
}

#[test]
fn cancelling_the_job_leaves_the_document_untouched() {
    let mut s = blob_session(8);
    let base = active_id(&s);
    let before = snapshot(&s, base);
    let (layers, steps) = (s.active().unwrap().doc.layer_count(), s.active().unwrap().history.past_len());
    use_provider(&mut s, FakeProvider { never_finish: true, ..Default::default() });
    let id = match s.start(CMD, json!({})).unwrap() {
        Started::Job(id) => id,
        Started::Done(v) => panic!("expected a background job, ran inline: {v}"),
    };
    assert!(s.cancel_job(id));
    assert!(matches!(s.wait_job(id), Err(EngineError::Cancelled)));
    s.join_cancelled_jobs();
    let st = s.active().unwrap();
    assert_eq!(st.doc.layer_count(), layers, "no layer appeared");
    assert_eq!(st.history.past_len(), steps, "no history step");
    assert_eq!(st.active_layer, Some(base), "the source layer is still active");
    assert_eq!(snapshot(&s, base), before, "the pixels never changed");
    assert!(named(&s, "Generative Fill").is_none());
}

#[test]
fn the_job_is_labelled_generative_fill() {
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider { never_finish: true, ..Default::default() });
    let id = match s.start(CMD, json!({})).unwrap() {
        Started::Job(id) => id,
        Started::Done(v) => panic!("expected a background job: {v}"),
    };
    let info = s.job(id).unwrap();
    assert_eq!(info.label, "Generative Fill");
    assert_eq!(info.command, CMD);
    s.cancel_job(id);
    let _ = s.wait_job(id);
    s.join_cancelled_jobs();
}

#[test]
fn the_command_is_registered_once_under_edit() {
    let specs = specs();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].id, "edit.generativeFill");
    assert_eq!(specs[0].menu, &["Edit"]);
    assert_eq!(specs[0].label, "Generative Fill\u{2026}");
    assert!(specs[0].journal);
    let all = crate::command_specs();
    assert_eq!(all.iter().filter(|c| c.id == CMD).count(), 1, "registered exactly once");
}

/// BF-02 X1 / amendment 16: one blend only. A 50%-coverage pixel ends up exactly halfway between
/// the original and the fill in the composited document (before, the provider and the layer alpha
/// both blended, so it got a quarter of the fill).
#[test]
fn a_half_selected_pixel_composites_halfway_between_original_and_fill() {
    for depth in [8, 16, 32] {
        let mut s = doc_session(depth, "rgb");
        // A black canvas, so "halfway to the 0.75 grey" is 0.375 in every channel.
        s.edit("black", |doc, active| {
            let surf = doc.layer_mut(active.unwrap()).unwrap().surface_mut().unwrap();
            surf.fill_rect(Rect::new(0, 0, 64, 48), &[0.0, 0.0, 0.0, 1.0]);
            // A hard 12 x 14 selection with one ring of 50% coverage around a fully selected core.
            let sel = doc.selection.as_mut().unwrap();
            sel.fill_rect(Rect::new(26, 16, 38, 30), &[0.5]);
            sel.fill_rect(Rect::new(27, 17, 37, 29), &[1.0]);
            Ok(())
        })
        .unwrap();
        let half = s.active().unwrap().doc.selection.as_ref().unwrap().pixel(26, 20)[0];
        assert!((half - 0.5).abs() < 1.0 / 255.0, "{depth}-bit: the fixture's edge is 50% selected: {half}");
        use_provider(&mut s, FakeProvider::default());
        s.execute(CMD, json!({})).unwrap();
        let doc = s.active().unwrap().doc.clone();
        let composite = photocraft_compose::render(&doc, Rect::new(0, 0, 64, 48));
        let at = |x: i32, y: i32| composite.px[(y * 64 + x) as usize];
        let (original, fill) = (0.0f32, 0.75f32);
        for c in 0..3 {
            let edge = at(26, 20)[c];
            assert!((edge - (original + fill) / 2.0).abs() <= 1.0 / 255.0, "{depth}-bit: 50% coverage gives {edge}, want halfway (0.375)");
            assert!((at(32, 23)[c] - fill).abs() <= 1.0 / 255.0, "{depth}-bit: full coverage is the fill");
            assert!((at(10, 10)[c] - original).abs() <= 1.0 / 255.0, "{depth}-bit: unselected pixels are the original");
        }
    }
}

/// BF-02 P1: the provider is sent the selection's bounds plus padding, clamped to the canvas, not
/// Content-Aware Fill's sampling window.
#[test]
fn the_window_sent_is_the_selection_bounds_plus_padding() {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 400, "height": 300, "background": "white"})).unwrap();
    s.execute("select.rect", json!({"x": 100, "y": 100, "width": 40, "height": 20})).unwrap();
    let provider = Arc::new(FakeProvider::default());
    s.set_ai_provider(Some(provider.clone()));
    let r = s.execute(CMD, json!({})).unwrap();
    let sent = provider.state.lock().unwrap().request.as_ref().unwrap().window;
    assert_eq!(sent, Rect::new(68, 68, 172, 152), "40 x 20 box plus 32 px on every side");
    assert_eq!(r["window"], json!([68, 68, 104, 84]), "{r}");
    // A box near the corner is clamped to the canvas, and a long box pads by a quarter of its length.
    s.execute("select.rect", json!({"x": 0, "y": 280, "width": 320, "height": 20})).unwrap();
    s.execute(CMD, json!({})).unwrap();
    let sent = provider.state.lock().unwrap().request.as_ref().unwrap().window;
    assert_eq!(sent, Rect::new(0, 200, 400, 300), "320 px long: 80 px of padding, clamped");
}

/// BF-02 P2: a window over the provider's `max_pixels` is refused with a sentence before any pixel
/// is read or sent, and the document is unchanged.
#[test]
fn a_window_over_the_provider_limit_is_refused_before_anything_is_sent() {
    let mut s = blob_session(8);
    let layers = s.active().unwrap().doc.layer_count();
    // The window here is the whole 64 x 48 canvas (3072 px).
    let provider = Arc::new(FakeProvider { max_pixels: Some(Ok(Some(3_000))), ..Default::default() });
    s.set_ai_provider(Some(provider.clone()));
    let e = s.execute(CMD, json!({})).unwrap_err();
    assert_eq!(e.to_string(), "The area to fill is 3072 pixels, over the 3000 pixels Craft AI Manager accepts; select a smaller area.");
    let st = provider.state.lock().unwrap();
    assert_eq!(st.limit_asks, 1, "the limit was asked for");
    assert!(st.request.is_none(), "nothing was submitted");
    drop(st);
    assert_eq!(s.active().unwrap().doc.layer_count(), layers);
    // At or under the limit it runs.
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider { max_pixels: Some(Ok(Some(3_072))), ..Default::default() });
    assert!(s.execute(CMD, json!({})).is_ok());
}

/// BF-02 P4: a provider that cannot even say its limit (Craft AI Manager not running) fails the
/// job with the manager's sentence before the window is read or anything is submitted.
#[test]
fn a_provider_that_is_not_running_fails_before_the_window_is_read() {
    let mut s = blob_session(8);
    let provider = Arc::new(FakeProvider { max_pixels: Some(Err(AiError::NoProvider)), ..Default::default() });
    s.set_ai_provider(Some(provider.clone()));
    assert_eq!(s.execute(CMD, json!({})).unwrap_err().to_string(), "Craft AI Manager isn't running");
    assert!(provider.state.lock().unwrap().request.is_none(), "nothing was encoded or submitted");
}

/// BF-02 follow-up: a manager that cannot answer the limit question is not a reason to refuse the
/// fill. Only "isn't running" and a refused token stop early; anything else falls back to the
/// default limit and the fill goes ahead.
#[test]
fn a_failed_limit_check_falls_back_to_the_default_limit() {
    for error in [
        AiError::Other("Craft AI Manager did not answer in time.".into()),
        AiError::Other("Craft AI Manager answered with something that is not its task list.".into()),
        AiError::Other("PhotoCraft lost the connection to Craft AI Manager.".into()),
    ] {
        let mut s = blob_session(8);
        let provider = Arc::new(FakeProvider { max_pixels: Some(Err(error.clone())), ..Default::default() });
        s.set_ai_provider(Some(provider.clone()));
        assert!(s.execute(CMD, json!({})).is_ok(), "{error}: the fill should still run");
        let st = provider.state.lock().unwrap();
        assert_eq!(st.limit_asks, 1, "the limit was asked for once");
        assert!(st.request.is_some(), "{error}: the window was still sent");
    }
}

/// BF-02 follow-up: a refused token is not a blip. It stops the fill before the window is read,
/// with the sentence that tells the user to restart Craft AI Manager.
#[test]
fn a_refused_token_fails_before_the_window_is_read() {
    let mut s = blob_session(8);
    let provider = Arc::new(FakeProvider { max_pixels: Some(Err(AiError::TokenMismatch)), ..Default::default() });
    s.set_ai_provider(Some(provider.clone()));
    assert_eq!(s.execute(CMD, json!({})).unwrap_err().to_string(), "Craft AI Manager refused the request (token mismatch); restart Craft AI Manager.");
    assert!(provider.state.lock().unwrap().request.is_none(), "nothing was encoded or submitted");
}

/// BF-02 P6: amendment 4's note for 32-bit documents (values are clamped to 0–1 for the model).
#[test]
fn a_32_bit_document_s_result_says_its_values_were_clamped() {
    for depth in [8, 16, 32] {
        let mut s = blob_session(depth);
        use_provider(&mut s, FakeProvider::default());
        let r = s.execute(CMD, json!({})).unwrap();
        if depth == 32 {
            assert_eq!(r["note"], "32-bit values were clamped to 0–1 for the AI model.", "{r}");
        } else {
            assert!(r.get("note").is_none(), "{depth}-bit has nothing to note: {r}");
        }
    }
}

/// BF-02 P7: `sampleAllLayers` must be a boolean; anything else is a bad-params error naming it.
#[test]
fn a_non_boolean_sample_all_layers_is_a_bad_params_error() {
    for value in [json!("yes"), json!(1), json!([true]), json!({})] {
        let mut s = blob_session(8);
        use_provider(&mut s, FakeProvider::default());
        let e = s.execute(CMD, json!({"sampleAllLayers": value})).unwrap_err();
        assert!(matches!(&e, EngineError::BadParams { msg, .. } if msg.contains("`sampleAllLayers`")), "{value}: {e:?}");
    }
    for value in [json!(true), json!(false), json!(null)] {
        let mut s = blob_session(8);
        use_provider(&mut s, FakeProvider::default());
        assert!(s.execute(CMD, json!({"sampleAllLayers": value})).is_ok(), "{value}");
    }
}

/// The record a layer carries, if it has one (PC-11).
fn record_of(s: &Session, id: LayerId) -> Option<ai_provenance::Record> {
    ai_provenance::read(s.active().unwrap().doc.layer(id).unwrap())
}

/// B16 (amendment 18): the layer a fill makes remembers what made it, in the same undo step that
/// creates it. The command result already said so (PC-08); now the layer does too.
#[test]
fn the_new_layer_keeps_a_record_of_the_model_that_made_it() {
    let mut s = blob_session(8);
    let base = active_id(&s);
    use_provider(&mut s, FakeProvider::default());
    let r = s.execute(CMD, json!({})).unwrap();
    let new = active_id(&s);
    let record = record_of(&s, new).unwrap_or_else(|| panic!("the new layer carries no record: {r}"));
    assert_eq!(record.command, ai_provenance::CMD_GENERATIVE_FILL, "which command made the layer");
    assert_eq!(record.model, "solid-fill", "the model that ran");
    assert_eq!(record.model_version.as_deref(), Some("epoch_244"), "the model build that ran");
    assert_eq!(record.model_sha256.as_deref(), Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c"), "the model file it ran");
    assert_eq!(record.device, "CPU", "the backend that ran");
    assert_eq!(record.device_name.as_deref(), Some("CPU"), "the hardware it ran on");
    // Only the layer the job made gets a record: the layer it sampled keeps none.
    assert_eq!(record_of(&s, base), None, "the source layer is untouched");
    // The record is part of the same history step, so one undo removes both.
    s.execute("edit.undo", json!({})).unwrap();
    assert_eq!(named(&s, "Generative Fill"), None, "one undo removes the layer and its record");
}

/// Amendment 18: a later job with a newer model build never rewrites the record an older one left.
#[test]
fn a_newer_model_build_leaves_the_older_layers_record_alone() {
    let mut s = blob_session(8);
    use_provider(&mut s, FakeProvider::default());
    s.execute(CMD, json!({})).unwrap();
    let first = named(&s, "Generative Fill").unwrap();
    let first_record = record_of(&s, first).unwrap();
    use_provider(&mut s, FakeProvider { model_version: Some("epoch_999".into()), ..Default::default() });
    s.execute(CMD, json!({})).unwrap();
    let second = named(&s, "Generative Fill 2").unwrap();
    assert_eq!(record_of(&s, second).unwrap().model_version.as_deref(), Some("epoch_999"), "the new layer records the build that ran");
    assert_eq!(record_of(&s, first), Some(first_record), "the older layer keeps what its own run left there");
}
