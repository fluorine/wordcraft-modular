//! Recipient-list data the Mailings commands and their dialogs share: file types, filters, sorts.

/// File extensions Select Recipients › Use an Existing List… opens.
pub const RECIPIENT_EXTENSIONS: &[&str] = &["csv", "tsv", "txt", "xlsx", "xlsm", "xls", "ods"];

/// Whether a file name is a spreadsheet read as a recipient list (.xlsx, .xlsm, .xls, .ods).
pub fn is_spreadsheet_name(name: &str) -> bool {
    std::path::Path::new(name).extension().is_some_and(|e| wordcraft_formats::sheet::is_extension(&e.to_string_lossy()))
}

/// A Filter Recipients comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterOp {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    Blank,
    NotBlank,
    Contains,
    NotContains,
}

impl FilterOp {
    /// Every comparison, in the order the dialog lists them.
    pub const ALL: [FilterOp; 10] = [
        FilterOp::Equal,
        FilterOp::NotEqual,
        FilterOp::Less,
        FilterOp::Greater,
        FilterOp::LessOrEqual,
        FilterOp::GreaterOrEqual,
        FilterOp::Blank,
        FilterOp::NotBlank,
        FilterOp::Contains,
        FilterOp::NotContains,
    ];

    /// The id used in command params.
    pub fn id(self) -> &'static str {
        match self {
            FilterOp::Equal => "equal",
            FilterOp::NotEqual => "notEqual",
            FilterOp::Less => "less",
            FilterOp::Greater => "greater",
            FilterOp::LessOrEqual => "lessOrEqual",
            FilterOp::GreaterOrEqual => "greaterOrEqual",
            FilterOp::Blank => "blank",
            FilterOp::NotBlank => "notBlank",
            FilterOp::Contains => "contains",
            FilterOp::NotContains => "notContains",
        }
    }

    /// The dialog's label (English source text).
    pub fn label(self) -> &'static str {
        match self {
            FilterOp::Equal => "Equal to",
            FilterOp::NotEqual => "Not equal to",
            FilterOp::Less => "Less than",
            FilterOp::Greater => "Greater than",
            FilterOp::LessOrEqual => "Less than or equal",
            FilterOp::GreaterOrEqual => "Greater than or equal",
            FilterOp::Blank => "Is blank",
            FilterOp::NotBlank => "Is not blank",
            FilterOp::Contains => "Contains",
            FilterOp::NotContains => "Does not contain",
        }
    }

    /// An id, a symbol (`=`, `<>`, `<`, `>`, `<=`, `>=`) or a spelled-out alias, any case.
    pub fn parse(t: &str) -> Option<FilterOp> {
        let k: String = t.trim().chars().filter(|c| !matches!(c, ' ' | '_' | '-')).collect::<String>().to_ascii_lowercase();
        Some(match k.as_str() {
            "equal" | "equals" | "equalto" | "=" | "==" | "eq" => FilterOp::Equal,
            "notequal" | "notequalto" | "<>" | "!=" | "ne" => FilterOp::NotEqual,
            "less" | "lessthan" | "<" | "lt" => FilterOp::Less,
            "greater" | "greaterthan" | ">" | "gt" => FilterOp::Greater,
            "lessorequal" | "lessthanorequal" | "<=" | "le" => FilterOp::LessOrEqual,
            "greaterorequal" | "greaterthanorequal" | ">=" | "ge" => FilterOp::GreaterOrEqual,
            "blank" | "isblank" | "empty" => FilterOp::Blank,
            "notblank" | "isnotblank" | "notempty" => FilterOp::NotBlank,
            "contains" => FilterOp::Contains,
            "notcontains" | "doesnotcontain" => FilterOp::NotContains,
            _ => return None,
        })
    }

    /// Whether the comparison takes a value (Is blank / Is not blank don't).
    pub fn has_value(self) -> bool {
        !matches!(self, FilterOp::Blank | FilterOp::NotBlank)
    }
}

/// One Filter Recipients row: `field op value`, joined to the row before by And (`or` false) or
/// Or. And binds tighter than Or: `a And b Or c` keeps records matching both a and b, or c.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterRule {
    pub field: String,
    pub op: FilterOp,
    pub value: String,
    pub or: bool,
}

/// One Sort Recipients key.
#[derive(Clone, Debug, PartialEq)]
pub struct SortKey {
    pub field: String,
    pub descending: bool,
}

/// Two cell values compared: as numbers when both are numbers, else as text ignoring case.
/// Orders two cells: numbers (compared numerically) before text (compared
/// case-insensitively). This is a total order, so `sort_by` stays consistent
/// (and can't panic) on columns that mix numbers and text, such as `9`, `10`, `1A`.
pub fn compare_cells(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering::{Greater, Less};
    let number = |s: &str| s.parse::<f64>().ok().filter(|x| x.is_finite());
    let (a, b) = (a.trim(), b.trim());
    match (number(a), number(b)) {
        (Some(x), Some(y)) => x.total_cmp(&y),
        (Some(_), None) => Less,
        (None, Some(_)) => Greater,
        (None, None) => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

fn rule_holds(r: &FilterRule, cell: &str) -> bool {
    use std::cmp::Ordering::{Equal, Greater, Less};
    let ord = || compare_cells(cell, &r.value);
    let lower = || (cell.to_lowercase(), r.value.trim().to_lowercase());
    match r.op {
        FilterOp::Equal => ord() == Equal,
        FilterOp::NotEqual => ord() != Equal,
        FilterOp::Less => ord() == Less,
        FilterOp::Greater => ord() == Greater,
        FilterOp::LessOrEqual => ord() != Greater,
        FilterOp::GreaterOrEqual => ord() != Less,
        FilterOp::Blank => cell.trim().is_empty(),
        FilterOp::NotBlank => !cell.trim().is_empty(),
        FilterOp::Contains => {
            let (c, v) = lower();
            c.contains(&v)
        }
        FilterOp::NotContains => {
            let (c, v) = lower();
            !c.contains(&v)
        }
    }
}

/// The records (indices into `rows`) that `filter` keeps, in `sort` order (stable: ties keep the
/// list's order). Rules and keys naming a field that isn't in `headers` are ignored.
pub fn select_records(headers: &[String], rows: &[Vec<String>], filter: &[FilterRule], sort: &[SortKey]) -> Vec<usize> {
    let col = |f: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(f.trim()));
    let rules: Vec<(Option<usize>, &FilterRule)> = filter.iter().map(|r| (col(&r.field), r)).filter(|(c, _)| c.is_some()).collect();
    let keep = |row: &Vec<String>| {
        if rules.is_empty() {
            return true;
        }
        // Or-groups of And-ed rules.
        let mut any = false;
        let mut all = true;
        for (k, (c, r)) in rules.iter().enumerate() {
            if k > 0 && r.or {
                any |= all;
                all = true;
            }
            let cell = c.and_then(|c| row.get(c)).map(String::as_str).unwrap_or("");
            all = all && rule_holds(r, cell);
        }
        any || all
    };
    let mut out: Vec<usize> = rows.iter().enumerate().filter(|(_, r)| keep(r)).map(|(i, _)| i).collect();
    let keys: Vec<(usize, bool)> = sort.iter().filter_map(|k| col(&k.field).map(|c| (c, k.descending))).collect();
    if !keys.is_empty() {
        out.sort_by(|a, b| {
            for (c, desc) in &keys {
                let get = |i: &usize| rows.get(*i).and_then(|r| r.get(*c)).map(String::as_str).unwrap_or("");
                let o = compare_cells(get(a), get(b));
                let o = if *desc { o.reverse() } else { o };
                if o != std::cmp::Ordering::Equal {
                    return o;
                }
            }
            std::cmp::Ordering::Equal
        });
    }
    out
}
