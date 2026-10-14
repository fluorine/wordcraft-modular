//! Zotero in a build without the `zotero` extension: there is no connection, and every
//! `ui.zotero.*` command says so.

use serde_json::Value;

use crate::WordApp;

#[derive(Default)]
pub struct ZoteroLink {
    /// Never set: no command is ever in flight.
    pub busy: Option<std::convert::Infallible>,
}

pub fn command(_: &mut WordApp, id: &str, _: &Value, _: Option<&egui::Context>) -> Option<Result<Value, String>> {
    id.strip_prefix("ui.zotero.")?;
    Some(Err(format!("{id}: this build has no `zotero` extension")))
}

pub fn poll(_: &mut WordApp, _: &egui::Context) {}

pub fn show_alert(_: &mut WordApp, _: &egui::Context) {}
