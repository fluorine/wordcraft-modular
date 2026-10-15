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
    /// Lives in the front end only, so it owns no engine command.
    pub front_end: bool,
}

const fn ext(id: &'static str, name: &'static str, depends_on: &'static [&'static str]) -> Extension {
    Extension { id, name, depends_on, formats: &[], front_end: false }
}

const fn format(id: &'static str, name: &'static str, formats: &'static [&'static str]) -> Extension {
    Extension { id, name, depends_on: &[], formats, front_end: false }
}

const fn front(id: &'static str, name: &'static str, depends_on: &'static [&'static str]) -> Extension {
    Extension { id, name, depends_on, formats: &[], front_end: true }
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
    front("zotero", "Zotero", &["references"]),
];

/// The extension that handles file extension `ext`, if one does (core formats have none).
pub fn for_format(ext: &str) -> Option<&'static Extension> {
    EXTENSIONS.iter().find(|e| e.formats.iter().any(|f| f.eq_ignore_ascii_case(ext)))
}

/// The extension that owns command `id` in the full build, if it is not core. Known even
/// when this build leaves that extension out.
pub fn owner(id: &str) -> Option<&'static str> {
    crate::extension_ids::OWNED.iter().find(|(c, _)| *c == id).map(|(_, e)| *e)
}

/// Command `id` belongs to an extension this build leaves out (so the UI hides it).
pub fn left_out(id: &str) -> bool {
    owner(id).is_some_and(|e| !compiled(e))
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
    ("zotero", cfg!(feature = "zotero")),
];

/// Extension `id` is on, given the extensions turned off: built in, not turned off, and every
/// extension it needs on too. Core always is.
pub fn enabled_in(disabled: &[String], id: &str) -> bool {
    enabled_depth(disabled, id, 0)
}

fn enabled_depth(disabled: &[String], id: &str, depth: usize) -> bool {
    if id == CORE {
        return true;
    }
    // The manifest has no cycles (tested); the bound keeps a broken one from recursing forever.
    depth < EXTENSIONS.len()
        && compiled(id)
        && !disabled.iter().any(|d| d == id)
        && get(id).is_some_and(|e| e.depends_on.iter().all(|d| enabled_depth(disabled, d, depth + 1)))
}

/// Extension `id` is on in this session.
pub fn enabled(s: &Session, id: &str) -> bool {
    enabled_in(&s.prefs.disabled_extensions, id)
}

/// Command `id` belongs to an extension that is off in this session (left out of the build or
/// turned off), so the front end hides it.
pub fn hidden(s: &Session, id: &str) -> bool {
    owner(id).is_some_and(|e| !enabled(s, e))
}

/// The hooks of the extensions that are on in this session, in call order.
pub fn active_hooks(s: &Session, reg: &crate::Registry) -> Vec<Hooks> {
    reg.hooks().iter().filter(|h| enabled(s, h.ext)).copied().collect()
}

/// The extensions whose content `doc` holds. Without them the content still shows and saves;
/// only editing it needs the extension.
pub fn content_needs(doc: &wordcraft_doc::Document) -> Vec<&'static str> {
    use wordcraft_doc::InlineObject as O;
    use wordcraft_doc::graphic::GraphicKind as G;
    fn add(out: &mut Vec<&'static str>, id: &'static str) {
        if !out.contains(&id) {
            out.push(id);
        }
    }
    let mut out = Vec::new();
    if !doc.comments.is_empty() {
        add(&mut out, "comments");
    }
    if !doc.sources.is_empty() {
        add(&mut out, "references");
    }
    doc.each_object(&mut |o| match o {
        O::Equation { .. } => add(&mut out, "equation-editor"),
        O::Graphic { graphic, .. } => add(
            &mut out,
            match graphic.kind {
                G::Chart => "charts",
                G::Diagram => "smart-art",
                G::Ink => "ink",
            },
        ),
        O::ControlStart { .. } => add(&mut out, "developer"),
        _ => {}
    });
    out
}

/// The other extensions that change with turning `id` on or off in this session: the ones it
/// needs that are off, or the ones that need it and are on.
pub fn affected(s: &Session, id: &str, on: bool) -> Vec<&'static str> {
    if on {
        let mut needs: Vec<&'static str> = Vec::new();
        let mut frontier = vec![id];
        while let Some(cur) = frontier.pop() {
            for d in get(cur).map(|e| e.depends_on).unwrap_or(&[]) {
                if !needs.contains(d) {
                    needs.push(d);
                    frontier.push(d);
                }
            }
        }
        needs.retain(|d| !enabled(s, d));
        needs
    } else {
        dependents(id).into_iter().filter(|d| enabled(s, d)).collect()
    }
}

/// Why a command of extension `ext` cannot run.
pub fn off_reason(ext: &str) -> String {
    let name = get(ext).map_or(ext, |e| e.name);
    if compiled(ext) { format!("the {name} extension is turned off") } else { format!("this build has no {name} extension") }
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
    ("restrict-editing", &["file.protect"]),
    ("shapes", &["insert.shape", "insert.textBox"]),
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
        ("protect", "developer.properties"),
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
        for e in EXTENSIONS.iter().filter(|e| compiled(e.id) && e.formats.is_empty() && !e.front_end) {
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

    /// The table of non-core command owners matches the registry of the full build.
    #[test]
    #[cfg(feature = "full")]
    fn extension_ids_are_current() {
        let mut rows: Vec<(&str, &str)> = crate::cmd::registry().all().iter().filter(|s| s.ext != CORE).map(|s| (s.id, s.ext)).collect();
        rows.sort();
        let mut text = String::from("// @generated by `extension_ids_are_current` (WORDCRAFT_UPDATE_IDS=1); do not edit.\n");
        text.push_str("pub const OWNED: &[(&str, &str)] = &[\n");
        for (id, ext) in &rows {
            text.push_str(&format!("    (\"{id}\", \"{ext}\"),\n"));
        }
        text.push_str("];\n");
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/extension_ids.rs");
        let current = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
        if std::env::var("WORDCRAFT_UPDATE_IDS").is_ok() {
            std::fs::write(&path, &text).unwrap();
        } else {
            assert_eq!(current, text, "src/extension_ids.rs is stale: run with WORDCRAFT_UPDATE_IDS=1");
        }
    }

    #[test]
    fn left_out_follows_the_build() {
        for (id, ext) in crate::extension_ids::OWNED {
            assert_eq!(left_out(id), !compiled(ext), "{id}");
        }
        assert!(!left_out("text.insert"), "core is never left out");
    }

    #[test]
    fn content_needs_names_what_a_document_holds() {
        let mut s = crate::Session::new(wordcraft_doc::Document::new());
        assert!(content_needs(&s.doc).is_empty());
        s.run("text.insert", &serde_json::json!({"text": "Hello"})).unwrap();
        s.run("select.all", &serde_json::json!({})).unwrap();
        s.run("review.newComment", &serde_json::json!({"text": "note"})).unwrap();
        s.run("insert.equation", &serde_json::json!({})).unwrap();
        let needs = content_needs(&s.doc);
        assert!(needs.contains(&"comments") && needs.contains(&"equation-editor"), "{needs:?}");
    }

    #[test]
    fn affected_lists_what_a_switch_changes() {
        let mut s = crate::Session::new(wordcraft_doc::Document::new());
        let off = affected(&s, "shapes", false);
        assert!(off.contains(&"charts") && off.contains(&"ink") && !off.contains(&"comments"), "{off:?}");
        s.prefs.disabled_extensions = vec!["shapes".into(), "charts".into()];
        assert_eq!(affected(&s, "charts", true), vec!["shapes"]);
        assert!(affected(&s, "comments", false).is_empty());
    }

    #[test]
    fn dependents_are_transitive() {
        let d = dependents("shapes");
        assert!(d.contains(&"charts") && d.contains(&"ink"));
        assert!(dependents("references").contains(&"mailings"));
        assert!(dependents("charts").is_empty());
    }
}
