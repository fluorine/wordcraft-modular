//! The bar under the ribbon when the open document has content for an extension that is off:
//! the content shows and saves as it is, and the bar offers to turn the extension on.

use egui::{RichText, Ui};
use serde_json::json;

use crate::WordApp;

/// What the bar knows about the open document (worked out once per document).
#[derive(Default)]
pub struct ExtBar {
    doc: Option<u64>,
    needs: Vec<&'static str>,
    dismissed: bool,
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let id = app.session.document_id();
    if app.ext_bar.doc != Some(id) {
        app.ext_bar = ExtBar { doc: Some(id), needs: wordcraft_engine::extensions::content_needs(&app.session.doc), dismissed: false };
    }
    if app.ext_bar.dismissed || !app.has_document() {
        return;
    }
    let off: Vec<&'static str> = app.ext_bar.needs.iter().copied().filter(|e| !wordcraft_engine::extensions::enabled(&app.session, e)).collect();
    if off.is_empty() {
        return;
    }
    let names: Vec<&str> = off.iter().map(|e| wordcraft_engine::extensions::get(e).map_or(*e, |x| tl!(x.name))).collect();
    let t = crate::theme::Tokens::get(ui.ctx());
    egui::Panel::top("extension_bar").frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(10, 4))).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::i18n::fmt(
                tl!("This document has content for extensions that are off: {names}. It shows and saves as it is."),
                &[("names", &names.join(", "))],
            )));
            let installed: Vec<&str> = off.iter().copied().filter(|e| wordcraft_engine::extensions::compiled(e)).collect();
            if !installed.is_empty() && ui.button(tl!("Turn On")).clicked() {
                for e in installed {
                    if let Err(err) = app.run("extensions.set", json!({"id": e, "enabled": true, "cascade": true})) {
                        app.status(err);
                    }
                }
            }
            if ui.button(tl!("Dismiss")).clicked() {
                app.ext_bar.dismissed = true;
            }
        });
    });
}
