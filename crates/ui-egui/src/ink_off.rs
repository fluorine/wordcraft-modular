//! The Draw tab's canvas side in a build without the `ink` extension: no tool is ever active,
//! so every entry point of `ink.rs` does nothing.

use egui::{Key, Painter, Rect, Ui};
use wordcraft_layout::DocLayout;

use crate::WordApp;

/// Never made: no stroke is drawn.
pub enum InkDrag {}

#[derive(Default)]
pub struct InkUi;

pub fn pointer(_: &mut WordApp, _: &Ui, _: &egui::Response, _: &[Rect], _: &DocLayout, _: f32) -> bool {
    false
}

pub fn key(_: &mut WordApp, _: Key) -> bool {
    false
}

pub fn paint(_: &WordApp, _: &Painter, _: &[Rect], _: &DocLayout, _: f32) {}

pub fn cursor(_: &WordApp) -> Option<egui::CursorIcon> {
    None
}

pub fn replay_tick(_: &mut WordApp, _: &egui::Context) {}

pub fn replay_bar(_: &mut WordApp, _: &Ui, _: Rect) {}
