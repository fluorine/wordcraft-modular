//! What the core front end asks of the engine's extensions, answered as "absent" when a build
//! leaves the extension out.

use wordcraft_engine::{MathEdit, Session};

/// The equation being edited.
pub fn equation_editing(s: &Session) -> Option<&MathEdit> {
    #[cfg(feature = "equation-editor")]
    return wordcraft_engine::cmd::equation::editing(s);
    #[cfg(not(feature = "equation-editor"))]
    {
        let _ = s;
        None
    }
}

/// Ink Replay is showing.
pub fn ink_replay(s: &Session) -> bool {
    #[cfg(feature = "ink")]
    return wordcraft_engine::cmd::draw::draw_state(&s.ext).replay;
    #[cfg(not(feature = "ink"))]
    {
        let _ = s;
        false
    }
}

/// A Draw tab tool (pen, eraser, lasso) is picked instead of Select.
pub fn ink_tool_active(s: &Session) -> bool {
    #[cfg(feature = "ink")]
    return wordcraft_engine::cmd::draw::draw_state(&s.ext).mode != wordcraft_engine::cmd::draw::DrawMode::Select;
    #[cfg(not(feature = "ink"))]
    {
        let _ = s;
        false
    }
}

/// The mail-merge list's field names and records.
pub fn merge_list(s: &Session) -> (Vec<String>, Vec<Vec<String>>) {
    #[cfg(feature = "mailings")]
    {
        let m = wordcraft_engine::cmd::mailings::merge_state(s);
        (m.headers.clone(), m.rows.clone())
    }
    #[cfg(not(feature = "mailings"))]
    {
        let _ = s;
        (Vec::new(), Vec::new())
    }
}

/// The whole recipient list, unfiltered, with its filter and sort.
#[derive(Default)]
pub struct MergeSource {
    pub headers: Vec<String>,
    pub source: Vec<Vec<String>>,
    pub filter: Vec<wordcraft_engine::cmd::recipients::FilterRule>,
    pub sort: Vec<wordcraft_engine::cmd::recipients::SortKey>,
}

pub fn merge_source(s: &Session) -> MergeSource {
    #[cfg(feature = "mailings")]
    {
        let m = wordcraft_engine::cmd::mailings::merge_state(s);
        MergeSource { headers: m.headers.clone(), source: m.source.clone(), filter: m.filter.clone(), sort: m.sort.clone() }
    }
    #[cfg(not(feature = "mailings"))]
    {
        let _ = s;
        MergeSource::default()
    }
}

/// The targets Cross-reference lists for the kind `to`.
pub fn ref_targets(s: &Session, to: &str) -> Vec<String> {
    #[cfg(feature = "references")]
    return wordcraft_engine::cmd::citations::ref_targets(s, to).into_iter().map(|(t, _)| t).collect();
    #[cfg(not(feature = "references"))]
    {
        let _ = (s, to);
        Vec::new()
    }
}

/// The mail-merge list's field names.
pub fn merge_fields(s: &Session) -> Vec<String> {
    merge_list(s).0
}

/// The mail-merge list has records.
pub fn has_recipients(s: &Session) -> bool {
    !merge_list(s).1.is_empty()
}

/// The next region this user may edit in a protected document, from `from`.
pub fn next_editable_region(s: &Session, from: &wordcraft_engine::doc::Pos) -> Option<(wordcraft_engine::doc::Pos, wordcraft_engine::doc::Pos)> {
    #[cfg(feature = "restrict-editing")]
    return wordcraft_engine::cmd::protect::next_region(s, from);
    #[cfg(not(feature = "restrict-editing"))]
    {
        let _ = (s, from);
        None
    }
}

/// A SmartArt graphic is selected.
pub fn smart_art_selected(s: &Session) -> bool {
    selected_smart_art(s).is_some()
}

/// The selected chart.
pub fn selected_chart(s: &Session) -> Option<std::sync::Arc<wordcraft_doc::chart::ChartSpec>> {
    #[cfg(feature = "charts")]
    return wordcraft_engine::cmd::chart::selected_chart(s);
    #[cfg(not(feature = "charts"))]
    {
        let _ = s;
        None
    }
}

/// The selected SmartArt graphic.
pub fn selected_smart_art(s: &Session) -> Option<std::sync::Arc<wordcraft_doc::smart_art::SmartArtSpec>> {
    #[cfg(feature = "smart-art")]
    return wordcraft_engine::cmd::smart_art::selected_smart_art(s);
    #[cfg(not(feature = "smart-art"))]
    {
        let _ = s;
        None
    }
}

/// A chart is selected.
pub fn chart_selected(s: &Session) -> bool {
    selected_chart(s).is_some()
}

/// Equations are typed in LaTeX.
pub fn equation_latex(s: &Session) -> bool {
    #[cfg(feature = "equation-editor")]
    return wordcraft_engine::cmd::equation::latex(s);
    #[cfg(not(feature = "equation-editor"))]
    {
        let _ = s;
        false
    }
}

/// Text typed into equations is normal text.
pub fn equation_normal_text(s: &Session) -> bool {
    #[cfg(feature = "equation-editor")]
    return wordcraft_engine::cmd::equation::normal_text(s);
    #[cfg(not(feature = "equation-editor"))]
    {
        let _ = s;
        false
    }
}

/// The document's styles as the Organizer lists them.
pub fn template_styles(doc: &wordcraft_doc::Document) -> serde_json::Value {
    #[cfg(feature = "templates")]
    return wordcraft_engine::cmd::templates::style_list(doc);
    #[cfg(not(feature = "templates"))]
    {
        let _ = doc;
        serde_json::Value::Null
    }
}
