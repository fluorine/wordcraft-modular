# Knowledge-graph summary (graphify)

> **Last updated:** 2026-10-13 · **Change:** major (first version) · **Source:** a graphify run over the whole repository. The full output is local only (`graphify-out/`, ignored by git). Rebuild it with `/graphify`.

This file supports [modularity.md](modularity.md). It keeps the parts of the graph report that
matter for the extension design. The graph has two parts: an AST extraction of the code, and a
semantic extraction of the docs and the screenshots. Calls between Rust files are resolved by name,
so some cross-crate edges are false matches. Use the communities as hints, not as proof.

## How to read it for modularity

- **`sel_result()`** (`crates/engine/src/cmd/mod.rs:131`), the common command result helper, has the highest betweenness. It connects 30
  engine command communities. The `cmd/mod.rs` helpers are the core API that extensions keep using.
- **`run()`, `Session` and `WordApp`** are god nodes. They are the integration points that
  Phases 2, 3 and 6 change.
- **`big()`** (`crates/ui-egui/src/widgets.rs:129`, called by `ribbon.rs`) places a ribbon button. It is the place
  that Phase 6 makes data-driven.
- **Feature communities cross crates** (equations, content controls, WordArt, protect). So an
  extension is a vertical slice.

## Summary
- 12609 nodes · 33247 edges · 459 communities (441 shown, 18 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 695 edges (avg confidence: 0.8)
- Token cost: 0 input · 232,583 output

## God Nodes (most connected - your core abstractions)
1. `run()` - 178 edges
2. `s()` - 149 edges
3. `lay()` - 135 edges
4. `sel_result()` - 113 edges
5. `Session` - 86 edges
6. `WordApp` - 86 edges
7. `Document` - 71 edges
8. `app()` - 61 edges
9. `FontDb` - 58 edges
10. `FontFace` - 55 edges

## Surprising Connections (you probably didn't know these)
- `CLAUDE.md (agent instructions)` --semantically_similar_to--> `AGENTS.md (agent instructions)`  [INFERRED] [semantically similar]
  CLAUDE.md → AGENTS.md
- `WebGPU to WebGL2 fallback (?webgl flag)` --semantically_similar_to--> `Windows wgpu backend selection (DX12 default, FXC, GL fallback, WGPU_BACKEND)`  [INFERRED] [semantically similar]
  packaging/web/README.md → README.md
- `Layout measured against Word for Windows cases` --semantically_similar_to--> `Page-by-page comparison harness against Word`  [INFERRED] [semantically similar]
  ROADMAP.md → docs/layout-parity.md
- `serve()` --calls--> `reply()`  [INFERRED]
  apps/wordcraft/src/control_server.rs → crates/engine/src/cmd/review.rs
- `legacy_key()` --calls--> `rotate()`  [INFERRED]
  crates/docx/src/crypt.rs → apps/wordcraft/src/logging.rs

## Hyperedges (group relationships)
- **Workflows that delegate to craft-common-actions @main** — workflows_freebsd, workflows_packaging_lint, workflows_release, workflows_windows_arm64, workflows_release_craft_common_actions [EXTRACTED 1.00]
- **Linux release packaging from one tarball (Flatpak bundle, deb/rpm via nfpm, package.sh)** — flatpak_ai_storyteller_wordcraft_bundle, linux_nfpm, linux_nfpm_package_sh, readme_release_artifacts [INFERRED 0.85]
- **Command-id dispatch surfaces (registry, control channel, MCP, parity catalog)** — agents_everything_is_a_command, agents_control_channel, agents_mcp_server, agents_parity_catalog [EXTRACTED 1.00]
- **Every surface dispatches through the command registry** — docs_architecture_command_registry, docs_control_protocol_methods, docs_mcp_tools, docs_architecture_session_run, docs_zotero_commands [EXTRACTED 0.95]
- **Beta-blocking gaps #1-#7** — docs_gaps_docx_corpus, docs_gaps_preserve_unrenderable, docs_gaps_stability, docs_gaps_pagination, docs_gaps_print_dialog_depth, docs_gaps_dialog_depth, docs_file_format_parity_content_controls [EXTRACTED 1.00]
- **craftrules progress-docs set** — roadmap, docs_roadmap, docs_gaps, target_app_parity_doc, docs_file_format_parity, docs_layout_parity, docs_hardware_parity [INFERRED 0.85]
- **Parity checklist docs measured against Word for Mac 16.113.4** — docs_target_app_parity, docs_typing_parity, docs_ui_parity, docs_localization_parity, docs_target_app_parity_word_for_mac_16_113_4 [EXTRACTED 1.00]
- **Readiness numbers from depth, dimensions and discounts** — docs_target_app_parity_feature_depth, docs_target_app_parity_dimension_table, docs_target_app_parity_readiness_discounts, docs_target_app_parity_ready_for_real_work, docs_target_app_parity_mainstream_practitioner, docs_target_app_parity_essentials_user [EXTRACTED 1.00]
- **Interface catalogs keyed to the Russian reference and checked by shared tests** — docs_localization_parity_i18n_catalogs, docs_localization_parity_russian_reference_catalog, docs_localization_hu_hu_tsv, docs_localization_tr_tr_tsv, docs_localization_parity_every_tab_and_command_is_translated, docs_localization_parity_untranslated_interface_text_audit [INFERRED 0.85]
- **Word-style Ribbon Tabs Shown in Docs Screenshots** — images_hero_home_ribbon, images_dark_insert_ribbon, images_design_design_tab, images_hero_ribbon_tabs [INFERRED 0.85]
- **Open Studio Handbook Sample Rendered Across Screenshots** — images_hero_open_studio_handbook_sample, images_hero_main_window, images_dark_dark_mode_insert_tab, images_design_design_tab, images_backstage_template_gallery [EXTRACTED 1.00]
- **Main Window Chrome Layout** — images_hero_title_bar, images_hero_ribbon_tabs, images_hero_ruler, images_hero_navigation_pane, images_hero_status_bar [INFERRED 0.85]
- **Ribbon tab screenshots demonstrating Word parity on the shared sample document** — images_layout_layout_ribbon_tab, images_references_references_ribbon_tab, images_review_review_ribbon_tab, images_open_studio_handbook_sample, images_word_style_ribbon_ui [INFERRED 0.85]
- **Review collaboration flow: comment, track, accept/reject** — images_review_comments_pane, images_review_track_changes, images_review_accept_reject_changes [INFERRED 0.85]
- **Generated reference tables (TOC, index, authorities, figures)** — images_references_table_of_contents, images_references_index_group, images_references_table_of_authorities, images_references_captions_group [INFERRED 0.75]

## Largest communities (labelled)

- Engine Command Tests
- Math Layout & Fonts
- SmartArt, Effects & Paste
- LaTeX Import/Export
- DOCX Story Writer
- PDF Export (krilla)
- Equation Model & OMML
- Paragraph & Template Commands
- Metafile (EMF/WMF)
- Layout Tests
- RTF Format
- DOCX Crypto, Print & Versions
- Table Resize & WordArt Layout
- Objects & Shapes Commands
- UI Caret & Scrolling
- Doc Edit Primitives & Icons
- DOCX Fixture Tests
- Lists & Numbering
- Legacy .doc Drawings
- Equation Tab & Colours UI
- Layout/Notes/Keyboard Dialogs
- Raster Renderer
- ODT Format
- DOCX Round-trip & Wrap
- Engine I/O & Macro Packages
- Ink & Draw Commands
- HTML Format
- UI Table Mouse & Resize
- Restrict Editing (Protect)
- Previews & Column Dispatch
- Proofing (Spelling/Grammar)
- Tools Catch-all & .doc FIB
- Table Mouse Commands
- Content Controls (SDT)
- Metafile Canvas
- WordArt
- Typing Parity Tests
- UI App Shell
- Insert Dialogs
- Mail Merge
- Sections Model
- Spreadsheet Import
- Read Aloud (Speech)
- UI Localization (i18n)
- DOCX Diagrams & Package

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Dark UI Theme (theme::Tokens)` and `Themes, Colors, Fonts, Paragraph Spacing, Effects, Set as Default`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `sel_result()` connect `Engine Caret` to `SmartArt, Effects & Paste`, `Engine Canvas`, `Paragraph & Template Commands`, `Engine Fields`, `Objects & Shapes Commands`, `Table Resize & WordArt Layout`, `Lists & Numbering`, `Engine Chart`, `Engine Index`, `Engine Outline`, `Engine Design`, `Tools Catch-all & .doc FIB`, `Table Mouse Commands`, `Engine Speech`, `WordArt`, `Mail Merge`, `Engine Equation`, `Engine Controls`, `Engine Table`, `Engine Edit`, `Engine References`, `Engine Format`, `Engine File`, `Engine Mod`, `Engine Review`, `Engine Table Draw`, `Engine Inspector`, `Engine Insert`, `Engine Page`, `Engine Text`?**
  _High betweenness centrality (0.201) - this node is a cross-community bridge._
- **Why does `para_block()` connect `Doc Lib` to `Doc Lib`, `Pdf Tests`, `Mail Merge`, `Docbin Table`, `Doc Lib`, `Formats Tests`, `Doc Edit Primitives & Icons`, `Layout Tests Pages`, `Engine Insert`, `Formats Model`, `DOCX Round-trip & Wrap`, `Docx Story`, `Engine Review`, `Docx Mod`, `Tools Catch-all & .doc FIB`?**
  _High betweenness centrality (0.100) - this node is a cross-community bridge._
- **Why does `big()` connect `Ui-Egui Ribbon` to `Ui-Egui Ribbon`, `Pdf Tests`, `Metafile (EMF/WMF)`, `Ui-Egui Widgets`, `Proof Hyphen`, `Layout Tests`?**
  _High betweenness centrality (0.045) - this node is a cross-community bridge._
- **Are the 110 inferred relationships involving `sel_result()` (e.g. with `insert()` and `drag_unit()`) actually correct?**
  _`sel_result()` has 110 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Self`, `Iterator`, `Item` to the rest of the system?**
  _1518 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Engine Command Tests` be split into smaller, more focused modules?**
  _Cohesion score 0.04210039324543141 - nodes in this community are weakly interconnected._