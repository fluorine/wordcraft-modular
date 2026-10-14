//! File › Options › Extensions: list the extensions and turn them on or off. Programmatic calls
//! never prompt; a change that would leave another extension without what it needs is refused
//! unless `cascade` is set.

use serde_json::{Value, json};

use crate::extensions::{CORE, EXTENSIONS, Extension, compiled, dependents, enabled_in, get};
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("extensions.list", "Extensions", "File › Options › Extensions", |s, _| Ok(list(s)))
            .params(r#"{} → [{id, name, core, installed, enabled, dependsOn, dependents, formats}]"#)
            .pure(),
        CommandSpec::new("extensions.set", "Turn Extension On or Off", "File › Options › Extensions", set)
            .params(r#"{"id": string, "enabled": bool, "cascade"?: bool (also turn on what it needs, or off what needs it)}"#)
            .pure(),
    ]
}

fn row(s: &Session, e: &Extension) -> Value {
    json!({
        "id": e.id,
        "name": e.name,
        "core": e.id == CORE,
        "installed": compiled(e.id),
        "enabled": enabled_in(&s.prefs.disabled_extensions, e.id),
        "dependsOn": e.depends_on,
        "dependents": dependents(e.id),
        "formats": e.formats,
    })
}

fn list(s: &Session) -> Value {
    Value::Array(EXTENSIONS.iter().map(|e| row(s, e)).collect())
}

/// Every extension `id` needs, directly or through others.
fn needs(id: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    let mut frontier = vec![id];
    while let Some(cur) = frontier.pop() {
        for d in get(cur).map(|e| e.depends_on).unwrap_or(&[]) {
            if !out.contains(d) {
                out.push(d);
                frontier.push(d);
            }
        }
    }
    out
}

fn set(s: &mut Session, v: &Value) -> CmdResult {
    let id = p::req_str(v, "id")?;
    let on = p::bool(v, "enabled").ok_or_else(|| CmdError::Params("`enabled` must be true or false".into()))?;
    let cascade = p::bool(v, "cascade").unwrap_or(false);
    let e = get(id).ok_or_else(|| CmdError::Params(format!("no extension `{id}`")))?;
    if e.id == CORE {
        return if on { Ok(row(s, e)) } else { Err(CmdError::Disabled("the core cannot be turned off".into())) };
    }
    if !compiled(e.id) {
        return Err(CmdError::Disabled(format!("{}: not installed in this build", e.name)));
    }
    let before: Vec<(&str, bool)> = EXTENSIONS.iter().map(|x| (x.id, enabled_in(&s.prefs.disabled_extensions, x.id))).collect();
    let mut disabled = s.prefs.disabled_extensions.clone();
    if on {
        let missing: Vec<&str> = needs(e.id).into_iter().filter(|d| !compiled(d)).collect();
        if !missing.is_empty() {
            return Err(CmdError::Disabled(format!("{} needs {}, which this build doesn't have", e.name, missing.join(", "))));
        }
        let off: Vec<&str> = needs(e.id).into_iter().filter(|d| disabled.iter().any(|x| x == d)).collect();
        if !off.is_empty() && !cascade {
            return Err(CmdError::Disabled(format!("{} needs {}: turn it on too (cascade)", e.name, off.join(", "))));
        }
        disabled.retain(|x| x != e.id && !off.contains(&x.as_str()));
    } else {
        let users: Vec<&str> = dependents(e.id).into_iter().filter(|d| enabled_in(&disabled, d)).collect();
        if !users.is_empty() && !cascade {
            return Err(CmdError::Disabled(format!("{} is needed by {}: turn them off too (cascade)", e.name, users.join(", "))));
        }
        // Each one is recorded, so turning `id` back on does not bring them back too.
        for x in users.into_iter().chain([e.id]) {
            if !disabled.iter().any(|d| d == x) {
                disabled.push(x.to_string());
            }
        }
    }
    disabled.sort();
    s.prefs.disabled_extensions = disabled;
    let changed: Vec<&str> = before.into_iter().filter(|(x, was)| enabled_in(&s.prefs.disabled_extensions, x) != *was).map(|(x, _)| x).collect();
    let mut out = row(s, e);
    if let Some(m) = out.as_object_mut() {
        m.insert("changed".into(), json!(changed));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::Session;

    fn on(s: &Session, id: &str) -> bool {
        crate::extensions::enabled(s, id)
    }

    #[test]
    fn turning_off_refuses_dependents_unless_cascading() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        let e = s.run("extensions.set", &json!({"id": "shapes", "enabled": false})).unwrap_err();
        assert!(e.to_string().contains("charts"), "{e}");
        assert!(on(&s, "shapes"));
        let r = s.run("extensions.set", &json!({"id": "shapes", "enabled": false, "cascade": true})).unwrap();
        assert!(!on(&s, "shapes") && !on(&s, "charts") && !on(&s, "ink"));
        assert!(r["changed"].as_array().unwrap().len() >= 5);
        assert!(on(&s, "comments"), "others stay on");
    }

    #[test]
    fn turning_on_asks_for_what_it_needs() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.run("extensions.set", &json!({"id": "shapes", "enabled": false, "cascade": true})).unwrap();
        assert!(s.run("extensions.set", &json!({"id": "charts", "enabled": true})).is_err());
        s.run("extensions.set", &json!({"id": "charts", "enabled": true, "cascade": true})).unwrap();
        assert!(on(&s, "charts") && on(&s, "shapes"));
        assert!(!on(&s, "ink"), "only what charts needs comes back");
    }

    #[test]
    fn a_command_of_a_turned_off_extension_is_refused_with_its_reason() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.run("extensions.set", &json!({"id": "comments", "enabled": false})).unwrap();
        let e = s.run("review.newComment", &json!({"text": "x"})).unwrap_err();
        assert!(e.to_string().contains("turned off"), "{e}");
        assert!(crate::extensions::hidden(&s, "review.newComment"));
        s.run("extensions.set", &json!({"id": "comments", "enabled": true})).unwrap();
        assert!(!crate::extensions::hidden(&s, "review.newComment"));
    }

    #[test]
    fn core_and_unknown_ids_are_refused() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        assert!(s.run("extensions.set", &json!({"id": "core", "enabled": false})).is_err());
        assert!(s.run("extensions.set", &json!({"id": "nope", "enabled": false})).is_err());
        assert!(s.run("extensions.set", &json!({"id": "comments"})).is_err());
        let list = s.run("extensions.list", &json!({})).unwrap();
        assert!(list.as_array().unwrap().iter().any(|e| e["id"] == "core" && e["enabled"] == true));
    }

    #[test]
    fn hooks_of_a_turned_off_extension_do_not_run() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.run("tools.recordMacro", &json!({"name": "m"})).unwrap();
        s.prefs.disabled_extensions = vec!["macros".into()];
        s.run("text.insert", &json!({"text": "a"})).unwrap();
        s.prefs.disabled_extensions.clear();
        s.run("tools.recordMacro", &json!({})).unwrap();
        let macros = s.run("tools.macros", &json!({})).unwrap();
        assert_eq!(macros["m"].as_array().map(Vec::len), Some(0), "nothing was recorded while macros were off: {macros}");
    }
}
