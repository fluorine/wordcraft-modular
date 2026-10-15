use serde_json::json;
use wordcraft_doc::props::BorderStyle;
use wordcraft_engine::Session;

fn s() -> Session {
    wordcraft_ext_design::install();
    Session::new(wordcraft_doc::Document::new())
}

fn run(s: &mut Session, id: &str, v: serde_json::Value) {
    s.run(id, &v).unwrap_or_else(|e| panic!("{id}: {e}"));
}

#[test]
fn page_border_sides_apply_exactly() {
    let mut s = s();
    run(&mut s, "design.pageBorders", json!({"sides": {"top": {"style": "dashed", "width": 3.0}}, "applyTo": "document"}));
    let pb = s.doc.last_section.page_borders.unwrap();
    assert_eq!(pb.top.map(|t| (t.style, t.width)), Some((BorderStyle::Dashed, 3.0)));
    assert!(pb.left.is_none());
    run(&mut s, "design.pageBorders", json!({"kind": "box", "style": "thick"}));
    assert_eq!(s.doc.last_section.page_borders.unwrap().left.map(|t| t.style), Some(BorderStyle::Thick));
    run(&mut s, "design.pageBorders", json!({"sides": {}}));
    assert!(s.doc.last_section.page_borders.is_none());
}

#[test]
fn page_color_takes_any_custom_hex() {
    let mut s = s();
    run(&mut s, "design.pageColor", json!({"color": "123ABC"}));
    assert_eq!(s.doc.settings.page_color, wordcraft_doc::Rgb::parse("123ABC"));
}
