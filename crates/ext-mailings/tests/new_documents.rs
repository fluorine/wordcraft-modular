use serde_json::json;
use wordcraft_doc::StoryRef;
use wordcraft_engine::Session;

fn s() -> Session {
    wordcraft_ext_mailings::install();
    Session::new(wordcraft_doc::Document::new())
}

fn run(s: &mut Session, id: &str, v: serde_json::Value) -> serde_json::Value {
    s.run(id, &v).unwrap_or_else(|e| panic!("{id}: {e}"))
}

fn text(s: &Session) -> String {
    s.doc.plain_text(StoryRef::Body)
}

/// Envelopes, Labels and Finish & Merge make a new, untitled document, as Word does, and like New
/// its undo history starts afresh. Undo/Redo across the swap put the wrong document under a file
/// (Undo, Save As, Redo, Save wrote the envelope over the saved file), so it isn't offered; the UI
/// asks to save the replaced document first.
#[test]
fn mailings_results_are_new_untitled_documents() {
    let dir = std::env::temp_dir().join(format!("wordcraft-engine-mailings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let original = dir.join("letter.docx");
    let saved_as = dir.join("saved-as.docx");
    let on_file = |p: &std::path::Path| wordcraft_engine::io::open_path(p).unwrap().plain_text(StoryRef::Body);
    for id in ["mailings.envelopes", "mailings.labels", "mailings.finish"] {
        let mut s = s();
        run(&mut s, "mailings.recipients", json!({"csv": "First Name\nAda\nAlan"}));
        run(&mut s, "text.insert", json!({"text": "Dear "}));
        run(&mut s, "mailings.insertField", json!({"field": "First Name"}));
        run(&mut s, "file.save", json!({"path": original.to_string_lossy()}));
        let on_disk = std::fs::read(&original).unwrap();
        run(&mut s, "text.insert", json!({"text": ", unsaved"}));
        let before = text(&s);
        let document = s.document_id();

        run(&mut s, id, json!({}));
        let result = text(&s);
        assert_ne!(result, before, "{id}: the result replaced the document");
        assert_ne!(s.document_id(), document, "{id}: a different document");
        assert_eq!(s.path, None, "{id}: the result is untitled");
        assert_eq!(run(&mut s, "file.save", json!({}))["saved"], false, "{id}: Save asks where (Save As)");
        assert_eq!(std::fs::read(&original).unwrap(), on_disk, "{id}: the original file is untouched");

        // Undo doesn't bring the old document back under the new one's identity; Redo can't
        // put the result back under a file saved in between.
        run(&mut s, "edit.undo", json!({}));
        assert_eq!(text(&s), result, "{id}: history starts afresh, like New");
        run(&mut s, "file.saveAs", json!({"path": saved_as.to_string_lossy()}));
        let written = on_file(&saved_as);
        run(&mut s, "edit.redo", json!({}));
        assert_eq!(text(&s), result, "{id}: nothing to redo");
        run(&mut s, "file.save", json!({}));
        assert_eq!(on_file(&saved_as), written, "{id}: the saved file still holds what was saved");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
