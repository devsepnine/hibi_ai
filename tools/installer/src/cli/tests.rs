use super::*;
use crate::app::test_support::fresh_app;
use crate::app::FocusArea;

/// A view that takes typed text, and the buffer its keystrokes land in.
type EntryCase = (app::View, fn(&App) -> &str);

/// The pane jumps sit in the global block *above* the focus-specific
/// handlers, and the status bar advertises them in both panes. If they
/// ever slip below the dispatch, they would silently work from only one
/// pane while the help line kept promising both.
#[test]
fn digit_keys_address_panes_from_either_pane() {
    for start in [FocusArea::Content, FocusArea::Tabs] {
        let mut app = fresh_app();
        app.focus = start;

        handle_list_input(&mut app, KeyCode::Char('1')).unwrap();
        assert_eq!(app.focus, FocusArea::Tabs, "`1` pressed from {:?}", start);

        handle_list_input(&mut app, KeyCode::Char('2')).unwrap();
        assert_eq!(app.focus, FocusArea::Content, "`2` pressed from {:?}", start);
    }
}

/// The pane digits are guarded inside `handle_list_input`, one level below
/// `dispatch_key`, and that placement is what keeps typing possible: every
/// text-entry view needs `1` and `2` as characters. Hoisting the guard up to
/// `dispatch_key` to make the jumps "global" would swallow them mid-word — a
/// `2` in an env value, a git branch, a path — and nothing else in the suite
/// would notice, because every other digit test starts from the List view.
#[test]
fn digits_type_themselves_in_every_text_entry_view() {
    let cases: [EntryCase; 3] = [
        (app::View::EnvInput, |a| a.env_input_buffer.as_str()),
        (app::View::ProjectPath, |a| a.project_path_buffer.as_str()),
        (app::View::SourceAddBranch, |a| a.source_input_buffer.as_str()),
    ];

    for (view, buffer) in cases {
        let (tx, _rx) = std::sync::mpsc::channel::<Result<RefreshResult>>();
        let mut app = fresh_app();
        app.current_view = view;
        app.env_input_buffer.clear();
        app.project_path_buffer.clear();
        app.source_input_buffer.clear();

        for c in ['1', '2'] {
            dispatch_key(&mut app, KeyCode::Char(c), &tx).unwrap();
        }

        assert_eq!(buffer(&app), "12", "digits must reach the buffer in {view:?}");
        assert_eq!(app.current_view, view, "{view:?} must not jump panes");
    }
}

/// `?` is the only route to the full binding list now, so it has to open
/// from either pane — and every key the box advertises has to close it,
/// including the `?` that opened it.
#[test]
fn question_mark_opens_help_from_either_pane_and_every_exit_closes_it() {
    for start in [FocusArea::Content, FocusArea::Tabs] {
        for exit in [KeyCode::Char('?'), KeyCode::Char('q'), KeyCode::Esc] {
            let mut app = fresh_app();
            app.focus = start;

            handle_list_input(&mut app, KeyCode::Char('?')).unwrap();
            assert_eq!(app.current_view, app::View::Help, "`?` from {:?}", start);

            handle_help_input(&mut app, exit).unwrap();
            assert_eq!(app.current_view, app::View::List, "{:?} must close help", exit);
        }
    }
}

/// `q` closes the overlay instead of quitting; leaving the global quit in
/// place would make the reference impossible to read without restarting.
#[test]
fn q_closes_help_rather_than_quitting() {
    let mut app = fresh_app();
    app.current_view = app::View::Help;

    handle_help_input(&mut app, KeyCode::Char('q')).unwrap();
    assert!(!app.should_quit);
}

/// A list an item can be ticked in, and the tick.
type SelectionCase = (&'static str, fn(&mut App));

/// Ticks live in three separate lists and survive tab switches, so each one has
/// to reach the exit guard. Every case leaves `app.tab` on its `fresh_app`
/// default: a guard scoped to the visible tab — which is what every other
/// selection helper here does — would still pass the component case while waving
/// the user out of the screen with their MCP and plugin choices pending.
const SELECTION_CASES: [SelectionCase; 3] = [
    ("component", |app| {
        let mut c = crate::tree::test_support::make_component("a.md");
        c.selected = true;
        app.components = vec![c];
    }),
    ("mcp server", |app| {
        let mut m = crate::app::test_support::make_mcp_server();
        m.selected = true;
        app.mcp_servers = vec![m];
    }),
    ("plugin", |app| {
        let mut p = crate::app::test_support::make_plugin();
        p.selected = true;
        app.plugins = vec![p];
    }),
];

/// An app on the List view with one item ticked in `case`'s list.
fn app_with_selection(case: SelectionCase) -> App {
    let mut app = fresh_app();
    case.1(&mut app);
    assert!(app.has_selection(), "{}: fixture ticked nothing", case.0);
    app
}

/// With nothing ticked the trip back has no cost, so it must not ask: a prompt
/// on an empty selection is the one that teaches the user to answer `y` without
/// reading, and the next prompt is the one that matters.
#[test]
fn esc_leaves_for_the_picker_at_once_when_nothing_is_selected() {
    let mut app = fresh_app();

    handle_list_input(&mut app, KeyCode::Esc).unwrap();
    assert_eq!(app.current_view, app::View::CliSelection);
}

/// With ticks on screen the same key has to stop and ask. Leaving re-enters
/// through the picker, which re-scans and replaces all three lists, so there is
/// nothing to undo afterwards.
#[test]
fn esc_asks_before_it_discards_a_selection() {
    for case in SELECTION_CASES {
        let mut app = app_with_selection(case);

        handle_list_input(&mut app, KeyCode::Esc).unwrap();
        assert_eq!(app.current_view, app::View::ConfirmExit, "{}", case.0);
        assert!(app.has_selection(), "{}: asking must not already discard", case.0);
    }
}

/// The exit binding lives in the content pane's handler, not the global block
/// above it, and this is why: in the tab bar `Esc` already means "back to the
/// list". Hoisting the exit up one level would cost the user that inner step
/// and no other test would notice, since every other `Esc` case starts from the
/// content pane.
#[test]
fn esc_in_the_tab_bar_still_only_returns_to_the_list() {
    let mut app = app_with_selection(SELECTION_CASES[0]);
    app.focus = FocusArea::Tabs;

    handle_list_input(&mut app, KeyCode::Esc).unwrap();
    assert_eq!(app.focus, FocusArea::Content);
    assert_eq!(app.current_view, app::View::List, "the tab bar must not leave the screen");
}

/// The prompt's own two answers, and the state each one owes the user: `y`
/// leaves and takes the ticks with it, `Esc`/`n` come back with every tick
/// intact. A cancel that cleared the selection anyway would be the worse bug of
/// the two, because the user asked for the opposite.
#[test]
fn the_exit_prompt_commits_on_y_and_keeps_the_selection_on_cancel() {
    for case in SELECTION_CASES {
        let mut app = app_with_selection(case);
        app.current_view = app::View::ConfirmExit;

        for cancel in [KeyCode::Esc, KeyCode::Char('n')] {
            handle_confirm_exit(&mut app, cancel);
            assert_eq!(app.current_view, app::View::List, "{}: {cancel:?}", case.0);
            assert!(app.has_selection(), "{}: {cancel:?} must keep the tick", case.0);
            app.current_view = app::View::ConfirmExit;
        }

        handle_confirm_exit(&mut app, KeyCode::Char('y'));
        assert_eq!(app.current_view, app::View::CliSelection, "{}", case.0);
        assert!(!app.has_selection(), "{}: leaving must discard what it warned about", case.0);
    }
}
