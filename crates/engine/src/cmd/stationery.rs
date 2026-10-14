//! Envelope sizes and label sheets: data the Mailings commands and their dialogs share.

/// Common envelope sizes: name, width and height in points (long side first).
pub const ENVELOPE_SIZES: [(&str, f32, f32); 8] = [
    ("Envelope #10", 684.0, 297.0),
    ("Envelope #9", 639.0, 279.0),
    ("Envelope Monarch", 540.0, 279.0),
    ("Envelope DL", 623.6, 311.8),
    ("Envelope C4", 918.4, 649.1),
    ("Envelope C5", 649.1, 459.2),
    ("Envelope C6", 459.2, 323.1),
    ("Envelope B5", 708.7, 498.9),
];

/// A sheet of labels: the label size, how many across and down, and where the first one sits.
/// Lengths in points; the pitch is from one label's edge to the next one's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelProduct {
    pub id: &'static str,
    /// What the labels are for: `address`, `shipping` or `return`.
    pub kind: &'static str,
    /// Letter or A4.
    pub paper: &'static str,
    pub cols: usize,
    pub rows: usize,
    pub width: f32,
    pub height: f32,
    pub top: f32,
    pub side: f32,
    pub h_pitch: f32,
    pub v_pitch: f32,
}

pub(crate) const MM: f32 = 72.0 / 25.4;

/// Common label sheets (the sizes are the industry's; the names are ours).
pub const LABEL_PRODUCTS: [LabelProduct; 9] = [
    LabelProduct {
        id: "letter-3x10",
        kind: "address",
        paper: "Letter",
        cols: 3,
        rows: 10,
        width: 189.0,
        height: 72.0,
        top: 36.0,
        side: 13.5,
        h_pitch: 198.0,
        v_pitch: 72.0,
    },
    LabelProduct {
        id: "letter-2x10",
        kind: "address",
        paper: "Letter",
        cols: 2,
        rows: 10,
        width: 288.0,
        height: 72.0,
        top: 36.0,
        side: 11.25,
        h_pitch: 301.5,
        v_pitch: 72.0,
    },
    LabelProduct {
        id: "letter-2x7",
        kind: "address",
        paper: "Letter",
        cols: 2,
        rows: 7,
        width: 288.0,
        height: 96.0,
        top: 60.0,
        side: 11.25,
        h_pitch: 301.5,
        v_pitch: 96.0,
    },
    LabelProduct {
        id: "letter-2x5",
        kind: "shipping",
        paper: "Letter",
        cols: 2,
        rows: 5,
        width: 288.0,
        height: 144.0,
        top: 36.0,
        side: 11.25,
        h_pitch: 301.5,
        v_pitch: 144.0,
    },
    LabelProduct {
        id: "letter-4x20",
        kind: "return",
        paper: "Letter",
        cols: 4,
        rows: 20,
        width: 126.0,
        height: 36.0,
        top: 36.0,
        side: 21.6,
        h_pitch: 147.6,
        v_pitch: 36.0,
    },
    LabelProduct {
        id: "a4-3x7",
        kind: "address",
        paper: "A4",
        cols: 3,
        rows: 7,
        width: 63.5 * MM,
        height: 38.1 * MM,
        top: 15.15 * MM,
        side: 7.21 * MM,
        h_pitch: 66.04 * MM,
        v_pitch: 38.1 * MM,
    },
    LabelProduct {
        id: "a4-2x7",
        kind: "address",
        paper: "A4",
        cols: 2,
        rows: 7,
        width: 99.1 * MM,
        height: 38.1 * MM,
        top: 15.15 * MM,
        side: 4.65 * MM,
        h_pitch: 101.6 * MM,
        v_pitch: 38.1 * MM,
    },
    LabelProduct {
        id: "a4-3x8",
        kind: "address",
        paper: "A4",
        cols: 3,
        rows: 8,
        width: 63.5 * MM,
        height: 33.9 * MM,
        top: 12.9 * MM,
        side: 7.21 * MM,
        h_pitch: 66.04 * MM,
        v_pitch: 33.9 * MM,
    },
    LabelProduct {
        id: "a4-2x4",
        kind: "shipping",
        paper: "A4",
        cols: 2,
        rows: 4,
        width: 99.1 * MM,
        height: 67.7 * MM,
        top: 13.1 * MM,
        side: 4.65 * MM,
        h_pitch: 101.6 * MM,
        v_pitch: 67.7 * MM,
    },
];

impl LabelProduct {
    pub fn by_id(id: &str) -> Option<&'static LabelProduct> {
        LABEL_PRODUCTS.iter().find(|p| p.id == id)
    }
    /// Label size as people say it: inches on Letter, millimetres on A4.
    pub fn size_text(&self) -> String {
        if self.paper == "A4" {
            format!("{:.1} × {:.1} mm", self.width / MM, self.height / MM)
        } else {
            let i = |pt: f32| format!("{}", ((pt / 72.0) * 1000.0).round() / 1000.0);
            format!("{}\" × {}\"", i(self.width), i(self.height))
        }
    }
}
