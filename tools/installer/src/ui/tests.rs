use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::{
    LIST_HELP, confirm, env_input, help, layout, list, mcp_list, pane_border_style, pane_title,
    plugin_list, render_status_bar, tabs,
};
use crate::app::test_support::fresh_app;
use crate::app::{App, FocusArea, View};

/// Signature shared by every pane renderer, so a test can hold a table of them.
type PaneRenderer = fn(&mut Frame, &App, Rect);

/// One border block to check: a label, the renderer that draws it, and the
/// state the app needs for that renderer to reach this particular block.
type Case = (&'static str, PaneRenderer, fn(&mut App));

/// Both panes must be distinguishable at once: the focused one takes the
/// accent, the other the plain border. A theme where the two resolve to the
/// same color would erase the only cue the content pane has.
#[test]
fn only_the_focused_pane_gets_the_accent_border() {
    let mut app = fresh_app();
    let accent = Style::default().fg(app.theme.border_focused());
    let plain = Style::default().fg(app.theme.border());
    assert_ne!(accent, plain);

    app.focus = FocusArea::Content;
    assert_eq!(pane_border_style(&app, FocusArea::Content), accent);
    assert_eq!(pane_border_style(&app, FocusArea::Tabs), plain);

    app.focus = FocusArea::Tabs;
    assert_eq!(pane_border_style(&app, FocusArea::Tabs), accent);
    assert_eq!(pane_border_style(&app, FocusArea::Content), plain);
}

/// Paint `render` into a throwaway terminal and hand back the buffer.
///
/// Asserting on `pane_border_style`/`pane_title` alone would not catch a
/// renderer that never calls them; only the painted cells prove what the user
/// sees. An unpainted cell reads as `Color::Reset` and a blank symbol, so a
/// renderer that draws nothing fails rather than passing.
fn paint_at(app: &App, render: PaneRenderer, width: u16, height: u16) -> Buffer {
    use ratatui::{Terminal, backend::TestBackend};

    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| render(f, app, f.area())).unwrap();
    terminal.backend().buffer().clone()
}

fn paint(app: &App, render: PaneRenderer) -> Buffer {
    paint_at(app, render, 60, 10)
}

/// Color of the block's top-left border corner.
fn border_color(app: &App, render: PaneRenderer) -> Color {
    paint(app, render)[(0, 0)].fg
}

/// One painted row, borders included.
fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
}

/// One painted row without its left and right border cells.
fn inner_row(buf: &Buffer, y: u16) -> String {
    (1..buf.area.width - 1)
        .map(|x| buf[(x, y)].symbol())
        .collect()
}

/// The border row carrying the block title.
fn title_row(app: &App, render: PaneRenderer) -> String {
    row(&paint(app, render), 0)
}

fn no_setup(_: &mut App) {}

#[test]
fn source_error_scroll_reaches_the_cause_and_clamps_after_resize() {
    let app = fresh_app();
    let message = format!(
        "Cannot save source settings\n{}\nInvalid YAML",
        "긴 경로/".repeat(32)
    );
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(32, 8)).unwrap();
    let mut max_scroll = 0;
    terminal
        .draw(|f| max_scroll = confirm::render_source_error(f, &app, &message, 0))
        .unwrap();
    let first = terminal.backend().buffer();
    assert!(row(first, 1).contains("Cannot save source settings"));
    assert!(row(first, 7).contains("[j/k] Scroll"));
    assert!(max_scroll > 0);
    terminal
        .draw(|f| {
            confirm::render_source_error(f, &app, &message, usize::MAX);
        })
        .unwrap();
    assert!(row(terminal.backend().buffer(), 6).contains("Invalid YAML"));

    let mut wide = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 20)).unwrap();
    wide.draw(|f| {
        assert_eq!(
            confirm::render_source_error(f, &app, &message, max_scroll),
            0
        );
    })
    .unwrap();
    let buffer = wide.backend().buffer();
    for expected in [
        "Cannot save source settings",
        "Invalid YAML",
        "[Enter/Esc] Close",
    ] {
        assert!((0..buffer.area.height).any(|y| row(buffer, y).contains(expected)));
    }
}

#[test]
fn searched_catalogs_display_only_matches_and_mark_the_visible_cursor() {
    use crate::app::Tab;
    use crate::app::test_support::{make_mcp_server, make_plugin};
    for tab in [Tab::McpServers, Tab::Plugins] {
        let mut app = fresh_app();
        app.tab = tab;
        for name in ["excluded", "match-1", "match-2"] {
            let mut server = make_mcp_server();
            server.def.name = name.into();
            app.mcp_servers.push(server);
            let mut plugin = make_plugin();
            plugin.def.name = name.into();
            app.plugins.push(plugin);
        }
        app.set_search_query("match".into());
        app.next_item();
        let buf = paint(&app, super::render_tab_list);
        assert!(row(&buf, 0).contains("/ match"));
        let rows: Vec<_> = (1..buf.area.height).map(|y| row(&buf, y)).collect();
        assert!(!rows.iter().any(|line| line.contains("excluded")));
        assert!(rows.iter().any(|line| line.contains("> [ ] match-2")));
        app.set_search_query("missing".into());
        let buf = paint(&app, super::render_tab_list);
        assert!((1..buf.area.height).any(|y| row(&buf, y).contains("No matches.")));
    }
}

#[test]
fn completed_progress_titles_distinguish_failure_cancellation_and_success() {
    let mut app = fresh_app();
    app.processing_complete = true;
    for (log, title) in [
        ("[OK] file", "✓ Complete"),
        ("[ERR] Cannot install", "Completed with errors"),
        ("[ERROR] Refresh failed", "Completed with errors"),
        ("[WARN] Cancelled by user", "Operation cancelled"),
    ] {
        app.processing_log = vec![log.into()];
        let buf = paint_at(&app, super::installing::render, 80, 20);
        assert!((0..buf.area.height).any(|y| row(&buf, y).contains(title)));
    }
}

#[test]
fn filter_summary_keeps_the_statusline_default_visible_at_80_columns() {
    let mut app = fresh_app();
    app.tab = crate::app::Tab::Statusline;
    app.current_statusline = Some("statusline_linux".into());
    let buf = paint_at(&app, list::render, 80, 10);
    let title = row(&buf, 0);
    assert!(title.contains("[Default: statusline_linux]"), "{title}");
    assert!(title.contains("Filter: Default"), "{title}");
}

#[test]
fn list_labels_the_filter_count_and_bulk_selection_scope() {
    use crate::component::InstallStatus;
    let mut app = fresh_app();
    for (name, status) in [
        ("changed.md", InstallStatus::Modified),
        ("local.md", InstallStatus::External),
        ("new.md", InstallStatus::New),
        ("installed.md", InstallStatus::Unchanged),
    ] {
        let mut component = crate::tree::test_support::make_component(name);
        component.status = status;
        app.components.push(component);
    }
    use crate::app::ComponentFilter::*;
    for (filter, name) in [
        (Default, "changed.md"),
        (Modified, "changed.md"),
        (External, "local.md"),
        (New, "new.md"),
        (Installed, "installed.md"),
    ] {
        app.set_component_filter(Default, true);
        app.set_component_filter(filter, true);
        let label = filter.display();
        let buf = paint_at(&app, list::render, 100, 10);
        let screen: String = (0..10).map(|y| row(&buf, y)).collect();
        assert!(screen.contains(&format!("Filter: {label}")), "{screen}");
        assert!(screen.contains(name), "{screen}");
        assert!(screen.contains("[f] Filter"), "{screen}");
        assert!(screen.contains("[a] Toggle all"), "{screen}");
        assert!(screen.contains("[E] Fold all"), "{screen}");
        assert!(
            screen.contains(if filter == Default {
                "Files: 3"
            } else {
                "Files: 1"
            }),
            "{screen}"
        );
    }
}

#[test]
fn filter_modal_shows_the_checked_statuses_and_scrolls_to_the_cursor_after_resize() {
    use crate::app::{ComponentFilter, FilterChoice, HiddenVisibility};
    let mut app = fresh_app();
    app.set_component_filter(ComponentFilter::External, true);
    app.set_component_filter(ComponentFilter::Installed, true);
    let cursor = FilterChoice::Visibility(HiddenVisibility::Only);
    app.current_view = View::Filters(cursor);
    let draw: PaneRenderer = |f, app, _| super::draw(f, app);
    let buf = paint_at(&app, draw, 80, 24);
    let lines: String = (0..24).map(|y| row(&buf, y)).collect();
    for label in [
        "Filters",
        "[ ] Default",
        "[x] External",
        "[x] Installed",
        "> ( ) Hidden only",
        "Esc Close",
    ] {
        assert!(lines.contains(label), "{label} missing: {lines}");
    }
    let buf = paint_at(&app, draw, 36, 7);
    assert!((0..7).any(|y| row(&buf, y).contains("> ( ) Hidden only")));
}

#[test]
fn empty_filter_explains_how_to_show_other_files() {
    let mut app = fresh_app();
    app.components
        .push(crate::tree::test_support::make_component("new.md"));
    app.set_component_filter(crate::app::ComponentFilter::Modified, true);
    for use_tree in [true, false] {
        if !use_tree {
            app.tree_views.clear();
        }
        let buf = paint_at(&app, list::render, 80, 10);
        let screen: String = (0..10).map(|y| row(&buf, y)).collect();
        assert!(
            screen.contains("No Modified files. Press f to change filter."),
            "{screen}"
        );
        assert!(screen.contains("Files: 0"), "{screen}");
        assert!(!screen.contains("new.md"), "{screen}");
    }
}

/// Give `list::render` a tree so it takes its `render_tree` branch; with
/// `tree_views` empty it falls through to `render_flat` instead.
fn with_tree(app: &mut App) {
    app.components = vec![crate::tree::test_support::make_component("a.md")];
    let filtered: Vec<(usize, &crate::component::Component)> =
        app.components.iter().enumerate().collect();
    let tree = crate::tree::TreeView::build_from_components(&app.components, &filtered);
    app.tree_views.insert(app.tab, tree);
}

fn with_mcp_server(app: &mut App) {
    app.mcp_servers = vec![crate::app::test_support::make_mcp_server()];
}

fn with_plugin(app: &mut App) {
    app.plugins = vec![crate::app::test_support::make_plugin()];
}

/// Each renderer draws its border in two places — an empty-state block and a
/// populated one — and every one of them must carry the cue. The table covers
/// both branches per renderer so reverting a single call site fails here.
const CASES: [Case; 6] = [
    ("list (flat)", list::render, no_setup),
    ("list (tree)", list::render, with_tree),
    ("mcp_list (empty)", mcp_list::render, no_setup),
    ("mcp_list (populated)", mcp_list::render, with_mcp_server),
    ("plugin_list (empty)", plugin_list::render, no_setup),
    ("plugin_list (populated)", plugin_list::render, with_plugin),
];

/// Every content renderer must take the accent border while the content pane
/// is focused. The tab bar is the counterexample checked in the same frame:
/// it stays plain, so a renderer hardcoding the accent fails too.
#[test]
fn content_renderers_paint_the_focus_border() {
    for (name, render, setup) in CASES {
        let mut app = fresh_app();
        setup(&mut app);

        app.focus = FocusArea::Content;
        assert_eq!(
            border_color(&app, render),
            app.theme.border_focused(),
            "{name} must show the accent border when focused",
        );
        assert_eq!(
            border_color(&app, tabs::render),
            app.theme.border(),
            "{name}: tab bar must stay plain while content is focused",
        );

        app.focus = FocusArea::Tabs;
        assert_eq!(
            border_color(&app, render),
            app.theme.border(),
            "{name} must drop the accent border when focus leaves",
        );
    }
}

/// `EnvInput`/`ProjectPath` draw a focused modal over the content pane, so a
/// lit pane behind it would put two accent borders on screen while only the
/// modal takes keys. `View::Diff` replaces the pane outright and borders it
/// plainly, so there the stale cue would sit on the tab bar instead — same
/// lie about which pane the next keystroke reaches, so the same expectation.
/// Help and confirmation dialogs keep the list behind their box,
/// and `Help` is the sharpest case: the box teaches `1`/`2` while itself
/// answering neither.
#[test]
fn a_modal_view_leaves_both_panes_plain() {
    for view in [
        View::EnvInput,
        View::ProjectPath,
        View::Diff,
        View::Help,
        View::ConfirmExit,
        View::ConfirmRemove,
        View::ConfirmHide,
    ] {
        for (name, render, setup) in CASES {
            let mut app = fresh_app();
            setup(&mut app);
            app.current_view = view;

            for focus in [FocusArea::Content, FocusArea::Tabs] {
                app.focus = focus;
                assert_eq!(
                    border_color(&app, render),
                    app.theme.border(),
                    "{name} must stay plain in {view:?} (focus {focus:?})",
                );
                assert_eq!(
                    border_color(&app, tabs::render),
                    app.theme.border(),
                    "tab bar must stay plain in {view:?} (focus {focus:?})",
                );
            }
        }
    }
}

#[test]
fn short_picker_keeps_all_targets_and_keyboard_hints_visible() {
    let app = fresh_app();
    let buffer = paint_at(&app, super::cli_selection::render, 40, 10);
    let content = (0..buffer.area.height)
        .map(|y| row(&buffer, y))
        .collect::<Vec<_>>()
        .join("\n");
    for label in [
        "HIBI AI",
        "Select target",
        "Claude Code",
        "Codex CLI",
        "Manage Sources",
        "[Enter] Select",
    ] {
        assert!(content.contains(label), "Missing {label} in {content}");
    }
}

#[test]
fn removal_dialog_lists_only_eligible_items_and_scrolls_to_later_names() {
    use crate::component::InstallStatus;

    let mut app = fresh_app();
    app.components = (1..=12)
        .map(|index| {
            let mut file =
                crate::tree::test_support::make_component(&format!("target-{index:02}.md"));
            file.status = InstallStatus::Modified;
            file
        })
        .collect();
    app.components
        .push(crate::tree::test_support::make_component("uninstalled.md"));
    app.request_removal();

    let content = |app: &App| {
        let buffer = paint_at(app, super::confirm::render_remove, 52, 10);
        (0..buffer.area.height)
            .map(|y| row(&buffer, y))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let top = content(&app);
    assert!(top.contains("Remove 12 selected items?"));
    assert!(top.contains("target-01.md  [modified]"));
    assert!(!top.contains("uninstalled.md"));
    assert!(!top.contains("target-12.md"));
    assert!(top.contains("[Esc] Cancel"));

    app.removal_scroll = 11;
    let end = content(&app);
    assert!(end.contains("target-12.md  [modified]"));
    assert!(!end.contains("target-01.md"));
    assert!(end.contains("[y] Remove"));
}

#[test]
fn removal_scroll_reaches_the_end_of_a_single_wrapped_name() -> anyhow::Result<()> {
    use crate::component::InstallStatus;
    use crossterm::event::KeyCode;

    let mut app = fresh_app();
    let name = format!("start-{}/tail.md", "nested/".repeat(38));
    let mut file = crate::tree::test_support::make_component(&name);
    file.status = InstallStatus::External;
    app.components = vec![file];
    app.set_component_filter(crate::app::ComponentFilter::External, true);
    let (tx, _rx) = std::sync::mpsc::channel();
    let area = Rect::new(0, 0, 40, 10);
    let press = |app: &mut App, key, area| crate::cli::dispatch_terminal_key(app, key, area, &tx);
    let content = |app: &App| {
        let buffer = paint_at(app, super::confirm::render_remove, area.width, area.height);
        (0..area.height)
            .map(|y| row(&buffer, y))
            .collect::<Vec<_>>()
            .join("\n")
    };

    press(&mut app, KeyCode::Char('r'), area)?;
    assert!(content(&app).contains("start-"));
    assert!(!content(&app).contains("tail.md"));
    for _ in 0..12 {
        press(&mut app, KeyCode::Down, area)?;
    }
    assert!(content(&app).contains("tail.md  [external]"));
    let last_offset = app.removal_scroll;
    assert!(last_offset > 0);
    press(&mut app, KeyCode::Char('j'), area)?;
    assert_eq!(app.removal_scroll, last_offset);
    press(&mut app, KeyCode::Up, area)?;
    assert_eq!(app.removal_scroll, last_offset - 1);
    press(&mut app, KeyCode::Char('k'), Rect::new(0, 0, 80, 24))?;
    assert_eq!(app.removal_scroll, 0);
    assert!(content(&app).contains("start-"));
    Ok(())
}

/// The digits are spelled out here rather than read from `FocusArea::shortcut`,
/// which is what makes these tests worth having: production derives the prefix
/// from that method, so if it ever changed, the label would follow silently and
/// only a literal expectation would notice. `cli`'s dispatch test pins the same
/// two literals from the keyboard side.
///
/// Every pane block must carry its own prefix and only its own — a copy-pasted
/// `FocusArea::Tabs` in a content renderer would teach the wrong key.
#[test]
fn every_pane_title_carries_its_own_shortcut() {
    for (name, render, setup) in CASES {
        let mut app = fresh_app();
        setup(&mut app);

        let content = title_row(&app, render);
        assert!(
            content.contains("[2]-"),
            "{name} title must show [2]-: {content}"
        );
        assert!(
            !content.contains("[1]-"),
            "{name} title must not show [1]-: {content}"
        );

        let tabs = title_row(&app, tabs::render);
        assert!(
            tabs.contains("[1]-"),
            "tab bar title must show [1]-: {tabs}"
        );
        assert!(
            !tabs.contains("[2]-"),
            "tab bar title must not show [2]-: {tabs}"
        );
    }
}

/// The prefix is prepended, not substituted: the pane's own name has to survive
/// or the number would cost the user the label it was meant to annotate.
#[test]
fn pane_title_keeps_the_original_text() {
    assert_eq!(
        pane_title(FocusArea::Tabs, " Config Installer "),
        " [1]-Config Installer "
    );
    assert_eq!(pane_title(FocusArea::Content, "Plugins"), " [2]-Plugins ");
}

/// Every key the status bar names must survive to the right edge.
///
/// This is the regression the fixed line exists for: the old per-tab help ran to
/// 102 columns, and `Paragraph` clips instead of wrapping, so at the standard 80
/// the tail — `[q] Quit` among it — simply was not painted. 32 is the narrowest
/// width that still holds the whole line inside the borders.
#[test]
fn the_status_bar_keeps_every_key_it_names_at_any_usable_width() {
    let app = fresh_app();
    for width in [120, 80, 40, 32] {
        let line = row(&paint_at(&app, render_status_bar, width, 3), 1);
        for key in ["[1/2] Pane", "[?] Keys", "[q] Quit"] {
            assert!(line.contains(key), "width {width} dropped {key}: {line}");
        }
    }
}

/// The line is the whole List-view help now, so its length is a budget rather
/// than a detail — the `?` overlay is where anything longer belongs.
#[test]
fn the_list_help_fits_the_narrowest_width_it_claims() {
    assert!(layout::columns(LIST_HELP) <= 30, "{LIST_HELP}");
}

/// The version is right-aligned by padding, and the padding is measured in
/// columns: the Diff help's arrow glyphs are three bytes and one column each, so
/// a byte-length measure moved the version one column inward per arrow.
#[test]
fn the_version_sits_flush_right_past_arrow_glyphs() {
    let mut app = fresh_app();
    app.current_view = View::Diff;
    let buf = paint_at(&app, render_status_bar, 80, 3);
    let line = inner_row(&buf, 1);
    assert!(line.ends_with(crate::fs::VERSION), "{line}");
}

/// The status message interpolates runtime strings — the path typed into the
/// project-path modal, a component filename, error text — so nothing restricts it
/// to ASCII. A codepoint measure under-counts a double-width glyph by one column,
/// so the padding overshoots and `Paragraph` clips the version it was meant to
/// align — the opposite direction of the arrow bug above, which is why both are
/// pinned.
#[test]
fn the_version_stays_flush_right_under_a_cjk_status_message() {
    let mut app = fresh_app();
    app.status_message = Some(String::from("한글 상태 메시지"));
    let buf = paint_at(&app, render_status_bar, 80, 3);
    let line = inner_row(&buf, 1);
    assert!(line.ends_with(crate::fs::VERSION), "{line}");
}

#[test]
fn a_long_status_keeps_the_help_and_version_visible_above_it() {
    let mut app = fresh_app();
    app.status_message = Some("No selected items are installed and removable in this view.".into());
    let area = Rect::new(0, 0, 54, 24);
    let height = 3 + super::status_message_rows(&app, area);
    let buffer = paint_at(&app, render_status_bar, area.width, height);
    let keys = inner_row(&buffer, 1);
    assert!(keys.contains("[q] Quit"));
    assert!(keys.ends_with(crate::fs::VERSION));
    let message = format!(
        "{}{}",
        inner_row(&buffer, 2),
        inner_row(&buffer, 3).trim_end()
    );
    assert_eq!(message, app.status_message.unwrap());
}

/// Rows painted by the help overlay, borders included.
fn help_rows(app: &App, width: u16, height: u16) -> Vec<String> {
    let buf = paint_at(app, help::render, width, height);
    (0..buf.area.height).map(|y| row(&buf, y)).collect()
}

fn help_row_for(rows: &[String], desc: &str) -> String {
    rows.iter()
        .find(|r| r.contains(desc))
        .unwrap_or_else(|| panic!("no overlay row for {desc:?}:\n{}", rows.join("\n")))
        .clone()
}

/// The last key character painted to the left of `desc` on its row. Reading it
/// this way also pins the column order — a key painted after its description
/// would fail rather than merely look wrong.
fn help_key_before(rows: &[String], desc: &str) -> char {
    let row = help_row_for(rows, desc);
    let keys = row.split(desc).next().unwrap().trim_end().to_string();
    keys.chars()
        .next_back()
        .unwrap_or_else(|| panic!("no key painted before {desc:?}: {row}"))
}

#[test]
fn the_overlay_documents_the_keys_the_status_bar_no_longer_lists() {
    let rows = help_rows(&fresh_app(), 80, 50);
    for desc in [
        "toggle between panes",
        "next tab",
        "toggle selection",
        "toggle all shown selections",
        "toggle all folders open / closed",
        "open file filters",
        "hide / unhide folder",
        "hide checked folders",
        "show / conceal hidden folders",
        "diff against installed",
        "install selected",
        "remove selected",
        "set / unset default",
        "MCP scope",
        "switch theme",
        "quit",
        "back to the CLI picker",
    ] {
        help_row_for(&rows, desc);
    }
}

/// The digits are spelled out from `FocusArea::shortcut` here, unlike the pane
/// titles: this box is a reference table, and a table that taught a key the
/// dispatch does not answer would be worse than no table.
#[test]
fn the_overlay_teaches_the_pane_digits_the_dispatch_answers() {
    let rows = help_rows(&fresh_app(), 80, 40);
    assert_eq!(
        help_key_before(&rows, "focus the tab bar"),
        FocusArea::Tabs.shortcut(),
    );
    assert_eq!(
        help_key_before(&rows, "focus the content list"),
        FocusArea::Content.shortcut(),
    );
}

/// A terminal tall enough for the table must not scroll, and one too short must
/// still reach the last row. Truncation is exactly the failure this whole change
/// removes, so an unreachable row here would reintroduce it one level down.
#[test]
fn the_overlay_reaches_its_last_row_in_a_short_terminal() {
    let mut app = fresh_app();
    assert_eq!(help::max_scroll(60), 0, "a tall terminal must not scroll");

    let short = 24;
    let max = help::max_scroll(short);
    assert!(max > 0, "{short} rows should not fit the table");

    let hidden = |app: &App| help_rows(app, 80, short).iter().any(|r| r.contains("quit"));
    assert!(!hidden(&app), "the last row should start below the fold");

    app.help_scroll = max;
    assert!(
        hidden(&app),
        "scrolling to {max} must bring the last row into view"
    );
}

/// Height of the box the overlay actually painted, borders included.
///
/// Measured from the buffer rather than asked of the sizing function, because
/// the claim under test is that the renderer uses it. The box is contiguous, so
/// its first and last painted rows are its borders — the blank separator rows
/// sit safely between them.
fn help_box_height(app: &App, width: u16, height: u16) -> u16 {
    let rows = help_rows(app, width, height);
    let first = rows
        .iter()
        .position(|r| !r.trim().is_empty())
        .expect("the overlay painted nothing");
    let last = rows.iter().rposition(|r| !r.trim().is_empty()).unwrap();
    (last - first + 1) as u16
}

/// 80% is a ceiling on the box, not a target for it, and the renderer and the
/// scroll clamp have to read the same one.
///
/// Sizing the box straight from the table filled a 24-row terminal edge to edge
/// with no margin, which read as a screen that had replaced the list rather than
/// one floating over it. Sizing it straight from the terminal would pad a tall
/// one with empty rows instead — hence both bounds here, on the painted cells.
#[test]
fn the_overlay_caps_its_height_at_80_percent_and_pads_no_taller_terminal() {
    let app = fresh_app();

    for height in [24u16, 30, 40, 60] {
        let painted = help_box_height(&app, 80, height);
        assert_eq!(
            painted,
            help::box_height(height),
            "{height} rows: the painted box must be the height the scroll clamp assumes",
        );
        assert!(
            u32::from(painted) * 100 <= u32::from(height) * 80,
            "{height} rows: a box of {painted} breaks the 80% ceiling",
        );
        assert!(
            painted < height,
            "{height} rows: no margin left around the box"
        );
    }

    // Past the height where 80% clears the table, the box stops growing.
    assert_eq!(help::box_height(50), help::box_height(60));
}

/// The prompt has to name both answers on screen: it is reached by a key that
/// is not written in the status bar, so a user who arrives by accident needs the
/// way out printed in front of them.
#[test]
fn the_exit_prompt_names_both_of_its_answers() {
    let mut app = fresh_app();
    app.current_view = View::ConfirmExit;
    let buf = paint_at(&app, confirm::render, 80, 24);
    let painted: String = (0..buf.area.height)
        .map(|y| row(&buf, y))
        .collect::<Vec<_>>()
        .join("\n");

    for expected in ["Confirm", "Leave for the CLI picker?", "[y] ", "[Esc] "] {
        assert!(
            painted.contains(expected),
            "prompt is missing {expected:?}:\n{painted}"
        );
    }
}

/// A collected env value is shown as its first four characters, so a token
/// typed in Korean, whose fourth byte sits inside a character, must still
/// paint instead of taking the installer down mid-dialog.
#[test]
fn collected_env_values_mask_multibyte_input_by_character() {
    let mut app = fresh_app();
    app.env_input_vars = vec![String::from("API_KEY"), String::from("REGION")];
    app.env_input_current = 1;
    app.env_input_values = vec![(String::from("API_KEY"), String::from("한글토큰값"))];

    let buf = paint_at(&app, env_input::render, 80, 30);
    let painted: String = (0..buf.area.height)
        .map(|y| row(&buf, y))
        .collect::<String>()
        .split_whitespace()
        .collect();

    assert!(painted.contains("API_KEY=한글토큰..."), "{painted}");
}
