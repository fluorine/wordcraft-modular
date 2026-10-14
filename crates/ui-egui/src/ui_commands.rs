//! The front end's own commands (`ui.*`, handled by `WordApp::ui_command`) and the extension
//! each one belongs to, as the engine's registry records it for engine commands.

use serde_json::{Value, json};

/// A `ui.*` command: id, label, where it lives, owning extension.
pub struct UiCommand {
    pub id: &'static str,
    pub label: &'static str,
    pub location: &'static str,
    pub ext: &'static str,
}

const fn c(id: &'static str, label: &'static str, location: &'static str) -> UiCommand {
    UiCommand { id, label, location, ext: wordcraft_engine::extensions::CORE }
}

const fn owned(id: &'static str, label: &'static str, location: &'static str, ext: &'static str) -> UiCommand {
    UiCommand { id, label, location, ext }
}

/// Every arm of `WordApp::ui_command`.
pub const UI_COMMANDS: &[UiCommand] = &[
    c("ui.tab", "Show Tab", "Ribbon"),
    c("ui.backstage", "File", "File"),
    c("ui.dialog", "Open Dialog", "Dialogs"),
    c("ui.closeDialog", "Close Dialog", "Dialogs"),
    c("ui.changePicture", "Change Picture", "Picture Format › Adjust"),
    c("ui.contextMenu", "Context Menu", "Page"),
    c("ui.collapseRibbon", "Collapse the Ribbon", "Ribbon"),
    c("ui.keytips", "Key Tips", "Ribbon"),
    c("ui.dark", "Dark Mode", "View"),
    c("ui.theme", "Office Theme", "File › Options › General"),
    c("ui.interfaceSize", "Interface Size", "File › Options › General"),
    c("ui.alignmentGuides", "Alignment Guides", "File › Options › Advanced"),
    c("ui.animations", "Animations", "File › Options › Advanced"),
    c("ui.language", "Interface Language", "File › Options › Language"),
    c("ui.saveChanges", "Save Changes", "File"),
    c("ui.openFileDialog", "Open File Dialog", "File"),
    c("ui.fileBrowser", "File Browser", "File"),
    owned("ui.openRecipientList", "Use an Existing List", "Mailings › Select Recipients", "mailings"),
    c("ui.printSettings", "Print Settings", "File › Print"),
    c("ui.print", "Print", "File › Print"),
    c("ui.printSubmit", "Print", "File › Print"),
    c("ui.makeDefaultApp", "Make Default App", "File › Options › General"),
    c("ui.discord", "Discord", "Title Bar"),
    c("ui.caretAnimation", "Caret Animation", "File › Options › Advanced"),
    c("ui.startScreen", "Start Screen", "File › Options › General"),
    owned("ui.linkTextBox", "Create Link", "Shape Format › Text", "shapes"),
];

/// The row for `id`.
pub fn get(id: &str) -> Option<&'static UiCommand> {
    UI_COMMANDS.iter().find(|c| c.id == id)
}

/// `id` is a front-end command whose extension this build leaves out.
pub fn left_out(id: &str) -> bool {
    get(id).is_some_and(|c| !wordcraft_engine::extensions::compiled(c.ext))
}

/// The commands this build has, for command lists.
pub fn describe() -> Vec<Value> {
    UI_COMMANDS
        .iter()
        .filter(|c| wordcraft_engine::extensions::compiled(c.ext))
        .map(|c| json!({"id": c.id, "label": c.label, "location": c.location, "extension": c.ext}))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each `"ui.…" =>` arm of `ui_command` has a row, and each row an arm.
    #[test]
    fn the_table_matches_ui_command() {
        let src = include_str!("lib.rs");
        let body = src.split("fn ui_command(").nth(1).and_then(|b| b.split("\n    }\n").next()).unwrap_or_default();
        let mut arms: Vec<&str> = Vec::new();
        for line in body.lines().map(str::trim_start).filter(|l| l.starts_with("\"ui.")) {
            let head = line.split("=>").next().unwrap_or_default().split(" if ").next().unwrap_or_default();
            arms.extend(head.split('|').map(|a| a.trim().trim_matches('"')).filter(|a| !a.is_empty()));
        }
        arms.sort_unstable();
        arms.dedup();
        let mut rows: Vec<&str> = UI_COMMANDS.iter().map(|c| c.id).collect();
        rows.sort_unstable();
        assert_eq!(arms, rows);
    }

    #[test]
    fn owners_are_known_extensions() {
        for c in UI_COMMANDS {
            assert!(wordcraft_engine::extensions::get(c.ext).is_some(), "{}: {}", c.id, c.ext);
            assert_eq!(left_out(c.id), !wordcraft_engine::extensions::compiled(c.ext));
        }
    }
}
