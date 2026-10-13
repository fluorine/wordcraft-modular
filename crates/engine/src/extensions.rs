//! Extension manifests: which extension owns each command.

use crate::CommandSpec;

/// The always-present core.
pub const CORE: &str = "core";

/// An extension's manifest.
#[derive(Clone, Copy, Debug)]
pub struct Extension {
    pub id: &'static str,
    pub name: &'static str,
    /// Extension ids this one needs; [`CORE`] is implied.
    pub depends_on: &'static [&'static str],
}

const fn ext(id: &'static str, name: &'static str, depends_on: &'static [&'static str]) -> Extension {
    Extension { id, name, depends_on }
}

/// Every extension, core first.
pub const EXTENSIONS: &[Extension] = &[
    ext(CORE, "Core", &[]),
    ext("comments", "Comments", &[]),
    ext("proofing", "Spelling, grammar and thesaurus", &[]),
    ext("references", "References and citations", &[]),
    ext("equation-editor", "Equation editor", &[]),
    ext("shapes", "Shapes and drawing canvas", &[]),
    ext("charts", "Charts", &["shapes"]),
    ext("smart-art", "SmartArt", &["shapes"]),
    ext("wordart", "WordArt", &["shapes"]),
    ext("ink", "Ink and Draw", &["shapes"]),
    ext("mailings", "Mail merge, envelopes and labels", &["references"]),
    ext("developer", "Content controls", &[]),
    ext("macros", "Macros", &[]),
    ext("compare", "Compare, combine and versions", &[]),
    ext("read-aloud", "Read Aloud", &[]),
    ext("accessibility", "Accessibility checker", &[]),
    ext("design", "Design themes and style sets", &[]),
    ext("templates", "Templates", &[]),
    ext("restrict-editing", "Restrict editing", &[]),
];

/// The manifest for `id`.
pub fn get(id: &str) -> Option<&'static Extension> {
    EXTENSIONS.iter().find(|e| e.id == id)
}

/// Commands that live in a shared module but belong to another extension.
const OVERRIDES: &[(&str, &[&str])] = &[
    (
        "comments",
        &[
            "review.newComment",
            "review.reply",
            "review.editComment",
            "review.deleteComment",
            "review.resolveComment",
            "review.nextComment",
            "review.previousComment",
            "review.comments",
        ],
    ),
    (
        "proofing",
        &[
            "review.spelling",
            "review.issues",
            "review.suggestions",
            "review.addToDictionary",
            "review.ignoreAll",
            "review.applySuggestion",
            "review.proofing",
            "review.thesaurus",
            "review.language",
            "review.editor",
        ],
    ),
    ("macros", &["tools.recordMacro", "tools.macros"]),
    ("compare", &["review.compare", "review.combine", "file.versions"]),
    ("accessibility", &["file.accessibility"]),
    ("templates", &["file.newFromTemplate", "file.saveTemplate"]),
];

fn owner_override(id: &str) -> Option<&'static str> {
    OVERRIDES.iter().find(|(_, ids)| ids.contains(&id)).map(|(ext, _)| *ext)
}

/// Sets the owner of each spec to `ext`, unless an override names another.
pub fn tag(specs: Vec<CommandSpec>, ext: &'static str) -> Vec<CommandSpec> {
    specs.into_iter().map(|s| CommandSpec { ext: owner_override(s.id).unwrap_or(ext), ..s }).collect()
}

/// The extensions that need `id`, directly or through others.
pub fn dependents(id: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let mut frontier = vec![id];
    while let Some(cur) = frontier.pop() {
        for e in EXTENSIONS {
            if e.depends_on.contains(&cur) && !out.contains(&e.id) {
                out.push(e.id);
                frontier.push(e.id);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_command_has_a_known_owner() {
        let reg = crate::cmd::registry();
        for s in reg.all() {
            assert!(get(s.ext).is_some(), "{} names unknown extension {}", s.id, s.ext);
        }
    }

    #[test]
    fn every_extension_owns_a_command() {
        let reg = crate::cmd::registry();
        let owners: HashSet<&str> = reg.all().iter().map(|s| s.ext).collect();
        for e in EXTENSIONS {
            assert!(owners.contains(e.id), "{} owns no command", e.id);
        }
    }

    #[test]
    fn overrides_name_real_commands() {
        let reg = crate::cmd::registry();
        for (ext, ids) in OVERRIDES {
            assert!(get(ext).is_some(), "override names unknown extension {ext}");
            for id in *ids {
                assert_eq!(reg.get(id).map(|s| s.ext), Some(*ext), "{id} is not registered under {ext}");
            }
        }
    }

    #[test]
    fn ids_and_dependencies_are_valid_and_acyclic() {
        let mut seen = HashSet::new();
        for e in EXTENSIONS {
            assert!(seen.insert(e.id), "duplicate extension {}", e.id);
            for d in e.depends_on {
                assert!(get(d).is_some(), "{} depends on unknown {d}", e.id);
                assert_ne!(*d, CORE, "{}: core is implied", e.id);
            }
        }
        for e in EXTENSIONS {
            assert!(!dependents(e.id).contains(&e.id), "dependency cycle through {}", e.id);
        }
    }

    #[test]
    fn dependents_are_transitive() {
        let d = dependents("shapes");
        assert!(d.contains(&"charts") && d.contains(&"ink"));
        assert!(dependents("references").contains(&"mailings"));
        assert!(dependents("charts").is_empty());
    }
}
