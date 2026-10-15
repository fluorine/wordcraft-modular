//! Format crates registered with the I/O dispatcher.

use wordcraft_doc::Document;

/// Open a format other than plain text / JSON; files that give no page size (Markdown, HTML)
/// start on `page`.
pub fn open(ext: &str, bytes: &[u8], page: &wordcraft_doc::SectionProps) -> Option<Result<Document, String>> {
    match ext {
        "docx" | "docm" | "dotx" | "dotm" => Some(wordcraft_docx::read(bytes).map_err(|e| e.to_string())),
        "doc" | "dot" => Some(read_doc(bytes, None).map(|(d, _)| d)),
        // Word XML Document (Flat OPC) or Word 2003 XML, told apart by the root element.
        "xml" => Some(wordcraft_docx::read_xml(bytes).map_err(|e| e.to_string())),
        // HTML, Markdown, ODT, RTF…: pictures fit the text width (see `io::fit_pictures`).
        other => wordcraft_formats::import_on(other, bytes, page).map(|r| {
            r.map(|mut doc| {
                crate::io::fit_pictures(&mut doc);
                doc
            })
        }),
    }
}

/// Save to a format other than plain text / JSON.
pub fn save(ext: &str, doc: &Document, opts: &crate::io::ExportOptions) -> Option<Result<Vec<u8>, String>> {
    // .docx/.docm/.dotx/.dotm: the extension picks the package flavour Word expects.
    if let Some(flavor) = wordcraft_docx::Flavor::from_ext(ext) {
        return Some(wordcraft_docx::write_as(doc, flavor).map_err(|e| e.to_string()));
    }
    match ext {
        "xml" if opts.word2003 => Some(wordcraft_docx::write_wordml2003(doc).map_err(|e| e.to_string())),
        "xml" => Some(wordcraft_docx::write_flat(doc).map_err(|e| e.to_string())),
        "png" => Some(render_png(doc, 0, 2.0, opts)),
        "pdf" => Some(export_pdf(doc, opts)),
        other => wordcraft_formats::export_with(other, doc, &wordcraft_formats::ExportOptions { comments: opts.markup && opts.comments }),
    }
}

/// A Word 97-2003 document (`doc-legacy`), and whether it was encrypted.
pub fn read_doc(bytes: &[u8], password: Option<&str>) -> Result<(Document, bool), String> {
    #[cfg(feature = "doc-legacy")]
    return read_docbin(bytes, password);
    #[cfg(not(feature = "doc-legacy"))]
    {
        let _ = (bytes, password);
        Err(crate::extensions::missing_format("doc"))
    }
}

#[cfg(feature = "doc-legacy")]
fn read_docbin(bytes: &[u8], password: Option<&str>) -> Result<(Document, bool), String> {
    use wordcraft_docbin::DocbinError;
    let encrypted = wordcraft_docbin::is_encrypted(bytes);
    match wordcraft_docbin::read_with_password(bytes, password) {
        Ok(doc) => Ok((doc, encrypted)),
        Err(DocbinError::PasswordRequired) => Err(wordcraft_docx::PASSWORD_REQUIRED.to_string()),
        Err(DocbinError::WrongPassword) => Err(wordcraft_docx::WRONG_PASSWORD.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// The document as PDF (`pdf-export`).
fn export_pdf(doc: &Document, opts: &crate::io::ExportOptions) -> Result<Vec<u8>, String> {
    #[cfg(feature = "pdf-export")]
    {
        let pdf = wordcraft_pdf::PdfOptions { include_markup: opts.markup, comments: opts.markup && opts.comments, ..Default::default() };
        wordcraft_pdf::export(doc, &pdf).map_err(|e| e.to_string())
    }
    #[cfg(not(feature = "pdf-export"))]
    {
        let _ = (doc, opts);
        Err(crate::extensions::missing_format("pdf"))
    }
}

/// Render page `page` as PNG at `scale` px/pt. With Print Markup's comments the page has its
/// markup area with the comments in balloons (page images always show tracked changes).
pub fn render_png(doc: &Document, page: usize, scale: f32, opts: &crate::io::ExportOptions) -> Result<Vec<u8>, String> {
    let lo = wordcraft_layout::LayoutOptions { comment_balloons: opts.markup && opts.comments, ..Default::default() };
    let l = wordcraft_layout::layout(doc, &mut wordcraft_layout::LayoutCache::new(), &lo);
    let p = l.pages.get(page).ok_or_else(|| format!("no page {}", page + 1))?;
    let img = wordcraft_render::render_page(doc, p, scale, &Default::default());
    Ok(img.to_png())
}
