//! Extension manifests: which extension owns each command.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use serde_json::Value;
use wordcraft_doc::Document;

use crate::{CmdError, CommandSpec, Session};

/// The always-present core.
pub const CORE: &str = "core";

/// An extension's manifest.
#[derive(Clone, Copy, Debug)]
pub struct Extension {
    pub id: &'static str,
    pub name: &'static str,
    /// Extension ids this one needs; [`CORE`] is implied.
    pub depends_on: &'static [&'static str],
    /// File extensions it opens or saves (a format extension may own no command).
    pub formats: &'static [&'static str],
}

const fn ext(id: &'static str, name: &'static str, depends_on: &'static [&'static str]) -> Extension {
    Extension { id, name, depends_on, formats: &[] }
}

const fn format(id: &'static str, name: &'static str, formats: &'static [&'static str]) -> Extension {
    Extension { id, name, depends_on: &[], formats }
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
    format("pdf-export", "PDF export and printing", &["pdf"]),
    format("doc-legacy", "Word 97-2003 documents", &["doc", "dot"]),
];

/// The extension that handles file extension `ext`, if one does (core formats have none).
pub fn for_format(ext: &str) -> Option<&'static Extension> {
    EXTENSIONS.iter().find(|e| e.formats.iter().any(|f| f.eq_ignore_ascii_case(ext)))
}

/// The error for a format this build leaves out.
pub fn missing_format(ext: &str) -> String {
    match for_format(ext) {
        Some(e) => format!("this build has no {} (the `{}` extension)", e.name, e.id),
        None => format!("unsupported format `.{ext}`"),
    }
}

/// The manifest for `id`.
pub fn get(id: &str) -> Option<&'static Extension> {
    EXTENSIONS.iter().find(|e| e.id == id)
}

/// Whether this build contains extension `id` (its cargo feature is on). Core always is.
pub fn compiled(id: &str) -> bool {
    id == CORE || BUILT.iter().any(|(e, on)| *on && *e == id)
}

/// Each extension and whether its cargo feature is on.
const BUILT: &[(&str, bool)] = &[
    ("comments", cfg!(feature = "comments")),
    ("proofing", cfg!(feature = "proofing")),
    ("references", cfg!(feature = "references")),
    ("equation-editor", cfg!(feature = "equation-editor")),
    ("shapes", cfg!(feature = "shapes")),
    ("charts", cfg!(feature = "charts")),
    ("smart-art", cfg!(feature = "smart-art")),
    ("wordart", cfg!(feature = "wordart")),
    ("ink", cfg!(feature = "ink")),
    ("mailings", cfg!(feature = "mailings")),
    ("developer", cfg!(feature = "developer")),
    ("macros", cfg!(feature = "macros")),
    ("compare", cfg!(feature = "compare")),
    ("read-aloud", cfg!(feature = "read-aloud")),
    ("accessibility", cfg!(feature = "accessibility")),
    ("design", cfg!(feature = "design")),
    ("templates", cfg!(feature = "templates")),
    ("restrict-editing", cfg!(feature = "restrict-editing")),
    ("pdf-export", cfg!(feature = "pdf-export")),
    ("doc-legacy", cfg!(feature = "doc-legacy")),
];

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
    ("restrict-editing", &["file.protect"]),
];

fn owner_override(id: &str) -> Option<&'static str> {
    OVERRIDES.iter().find(|(_, ids)| ids.contains(&id)).map(|(ext, _)| *ext)
}

/// Per-session state of the extensions, one value per type. An extension keeps its state
/// here, not in a `Session` field, so a build without it needs no change to `Session`.
#[derive(Default)]
pub struct ExtState(HashMap<TypeId, Box<dyn Any + Send>>);

impl ExtState {
    /// The `T` this session holds, if any.
    pub fn get<T: Any + Send>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>()).and_then(|b| b.downcast_ref())
    }
    /// The `T` this session holds; a default one is added first if there is none.
    pub fn get_mut<T: Any + Send + Default>(&mut self) -> Option<&mut T> {
        self.0.entry(TypeId::of::<T>()).or_insert_with(|| Box::new(T::default())).downcast_mut()
    }
    /// [`Self::get_mut`] as a command error.
    pub fn state<T: Any + Send + Default>(&mut self) -> Result<&mut T, CmdError> {
        self.get_mut().ok_or_else(|| CmdError::Failed("extension state is missing".into()))
    }
}

/// What a hook sees of the running command.
pub struct RunCtx<'a> {
    pub id: &'a str,
    pub params: &'a Value,
    pub mutates: bool,
    /// The document before the command; `Some` only for mutating commands.
    pub before: Option<&'a Document>,
}

/// One extension's calls around `Session::run`. Every hook is optional.
#[derive(Clone, Copy)]
pub struct Hooks {
    pub ext: &'static str,
    /// Before the undo snapshot. `Err` refuses the command; `Ok(true)` asks for `verify`.
    pub before: Option<fn(&mut Session, &RunCtx) -> Result<bool, CmdError>>,
    /// After success, when `before` asked; `Err` rolls the command back.
    pub verify: Option<fn(&Session, &RunCtx) -> Result<(), CmdError>>,
    /// After a mutating command succeeds; `true` = the hook changed the document.
    pub after_edit: Option<fn(&mut Session, &RunCtx) -> bool>,
    /// After `Session::clamp_selection`: keep the extension's own positions valid.
    pub after_clamp: Option<fn(&mut Session)>,
}

impl Hooks {
    pub const fn new(ext: &'static str) -> Self {
        Hooks { ext, before: None, verify: None, after_edit: None, after_clamp: None }
    }
}

/// Sets the owner of each spec to `ext`, unless an override names another, and drops the
/// specs of extensions this build leaves out.
pub fn tag(specs: Vec<CommandSpec>, ext: &'static str) -> Vec<CommandSpec> {
    specs.into_iter().map(|s| CommandSpec { ext: owner_override(s.id).unwrap_or(ext), ..s }).filter(|s| compiled(s.ext)).collect()
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

    /// Known calls by id from a module to an extension it does not depend on.
    const ALLOWED_CALLS: &[(&str, &str)] = &[
        // Comments-only protection lets replies through; a name check, not a call.
        ("protect", "review.reply"),
    ];

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
        for e in EXTENSIONS.iter().filter(|e| compiled(e.id) && e.formats.is_empty()) {
            assert!(owners.contains(e.id), "{} owns no command", e.id);
        }
    }

    #[test]
    fn overrides_name_real_commands() {
        let reg = crate::cmd::registry();
        for (ext, ids) in OVERRIDES {
            assert!(get(ext).is_some(), "override names unknown extension {ext}");
            if !compiled(ext) {
                continue;
            }
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

    /// Each module's default owner, read from the `tag(..)` calls in `cmd/mod.rs`.
    fn module_owners(src: &str) -> Vec<(String, String)> {
        src.lines()
            .filter_map(|l| {
                let rest = l.split("extensions::tag(").nth(1)?;
                let module = rest.split("::specs()").next()?.trim().to_string();
                let ext = rest.split('"').nth(1)?.to_string();
                Some((module, ext))
            })
            .collect()
    }

    /// Command-id literals in a module's non-test code must name a command of the
    /// module's own extension, of core, or of a declared dependency.
    #[test]
    fn id_calls_respect_dependencies() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cmd");
        let mod_src = std::fs::read_to_string(dir.join("mod.rs")).unwrap();
        let reg = crate::cmd::registry();
        let mut bad = Vec::new();
        for (module, ext) in module_owners(&mod_src) {
            let src = std::fs::read_to_string(dir.join(format!("{module}.rs"))).unwrap();
            let code = src.split("#[cfg(test)]").next().unwrap_or("");
            let lits: Vec<&str> = code.split('"').skip(1).step_by(2).collect();
            let defined = |id: &str| code.contains(&format!("CommandSpec::new(\"{id}\""));
            // A module may refer to every extension it hosts commands for.
            let mut hosted: Vec<&str> = vec![ext.as_str()];
            for id in lits.iter().filter(|id| defined(id)) {
                if let Some(s) = reg.get(id) {
                    hosted.push(s.ext);
                    hosted.extend(get(s.ext).map(|e| e.depends_on).unwrap_or(&[]));
                }
            }
            hosted.extend(get(&ext).map(|e| e.depends_on).unwrap_or(&[]));
            for lit in lits {
                let Some(spec) = reg.get(lit) else { continue };
                let callee = spec.ext;
                let own = callee == CORE || hosted.contains(&callee);
                if !own && !ALLOWED_CALLS.contains(&(module.as_str(), lit)) {
                    bad.push(format!("{module} ({ext}) -> {lit} ({callee})"));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "undeclared cross-extension id calls:
{}",
            bad.join(
                "
"
            )
        );
    }

    #[test]
    fn hooks_belong_to_known_extensions() {
        for h in crate::cmd::hooks() {
            assert!(get(h.ext).is_some_and(|e| e.id != CORE), "hook for {}", h.ext);
            assert!(h.before.is_some() || h.verify.is_some() || h.after_edit.is_some() || h.after_clamp.is_some(), "{} has no hook", h.ext);
        }
    }

    #[test]
    fn ext_state_starts_empty_and_keeps_values_by_type() {
        let mut st = ExtState::default();
        assert!(st.get::<u32>().is_none());
        if let Some(n) = st.get_mut::<u32>() {
            *n = 7;
        }
        assert_eq!(st.get::<u32>(), Some(&7));
        assert!(st.get::<String>().is_none());
    }

    #[test]
    fn every_extension_has_a_feature() {
        for e in EXTENSIONS {
            let manifest = include_str!("../Cargo.toml");
            assert!(e.id == CORE || manifest.contains(&format!("\n{} = [", e.id)), "no cargo feature for {}", e.id);
        }
    }

    #[test]
    fn features_enable_their_dependencies() {
        for e in EXTENSIONS.iter().filter(|e| compiled(e.id)) {
            for d in e.depends_on {
                assert!(compiled(d), "{} is built without its dependency {d}", e.id);
            }
        }
    }

    #[test]
    fn formats_have_one_owner_and_a_clear_error() {
        let mut seen = HashSet::new();
        for e in EXTENSIONS {
            for f in e.formats {
                assert!(seen.insert(*f), "two extensions handle .{f}");
            }
        }
        assert_eq!(for_format("PDF").map(|e| e.id), Some("pdf-export"));
        assert!(missing_format("doc").contains("doc-legacy"));
        assert!(for_format("docx").is_none(), "docx is core");
    }

    #[test]
    fn dependents_are_transitive() {
        let d = dependents("shapes");
        assert!(d.contains(&"charts") && d.contains(&"ink"));
        assert!(dependents("references").contains(&"mailings"));
        assert!(dependents("charts").is_empty());
    }
}
