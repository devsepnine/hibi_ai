use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::Frame;

use super::{
    confirm_exit, help, list, mcp_list, pane_border_style, pane_title, plugin_list,
    render_status_bar, tabs, LIST_HELP,
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
    use ratatui::{backend::TestBackend, Terminal};

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
    (1..buf.area.width - 1).map(|x| buf[(x, y)].symbol()).collect()
}

/// The border row carrying the block title.
fn title_row(app: &App, render: PaneRenderer) -> String {
    row(&paint(app, render), 0)
}

fn no_setup(_: &mut App) {}

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
/// `View::Help` and `View::ConfirmExit` keep the list visible behind their box,
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
        assert!(content.contains("[2]-"), "{name} title must show [2]-: {content}");
        assert!(!content.contains("[1]-"), "{name} title must not show [1]-: {content}");

        let tabs = title_row(&app, tabs::render);
        assert!(tabs.contains("[1]-"), "tab bar title must show [1]-: {tabs}");
        assert!(!tabs.contains("[2]-"), "tab bar title must not show [2]-: {tabs}");
    }
}

/// The prefix is prepended, not substituted: the pane's own name has to survive
/// or the number would cost the user the label it was meant to annotate.
#[test]
fn pane_title_keeps_the_original_text() {
    assert_eq!(pane_title(FocusArea::Tabs, " Config Installer "), " [1]-Config Installer ");
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
    assert!(LIST_HELP.chars().count() <= 30, "{LIST_HELP}");
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

/// The overlay is the only documentation these keys have — `a` and `n` were
/// never listed anywhere before it — so a binding missing here is a binding the
/// user cannot discover at all.
#[test]
fn the_overlay_documents_the_keys_the_status_bar_no_longer_lists() {
    let rows = help_rows(&fresh_app(), 80, 40);
    for desc in [
        "toggle between panes",
        "next tab",
        "toggle selection",
        "select all / none",
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
    assert!(hidden(&app), "scrolling to {max} must bring the last row into view");
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
        assert!(painted < height, "{height} rows: no margin left around the box");
    }

    // Past the height where 80% clears the table, the box stops growing.
    assert_eq!(help::box_height(40), help::box_height(60));
}

/// The prompt has to name both answers on screen: it is reached by a key that
/// is not written in the status bar, so a user who arrives by accident needs the
/// way out printed in front of them.
#[test]
fn the_exit_prompt_names_both_of_its_answers() {
    let buf = paint_at(&fresh_app(), confirm_exit::render, 80, 24);
    let painted: String = (0..buf.area.height)
        .map(|y| row(&buf, y))
        .collect::<Vec<_>>()
        .join("\n");

    for expected in ["Confirm", "Leave for the CLI picker?", "[y] ", "[Esc] "] {
        assert!(painted.contains(expected), "prompt is missing {expected:?}:\n{painted}");
    }
}

