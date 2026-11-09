use super::*;
use crate::app::FocusArea;
use crate::app::test_support::fresh_app;

mod status_filter {
    use super::*;
    use crate::app::{Tab, TargetCli, View};
    use crate::component::{ComponentType, InstallStatus};
    use crate::tree::{TreeNode, test_support::make_component};

    fn fixture() -> App {
        let mut app = fresh_app();
        for (name, status) in [
            ("group/changed.md", InstallStatus::Modified),
            ("group/new.md", InstallStatus::New),
            ("group/local.md", InstallStatus::External),
            ("installed.md", InstallStatus::Unchanged),
            ("managed.md", InstallStatus::Managed),
        ] {
            let mut component = make_component(name);
            component.status = status;
            component.selected = true;
            app.components.push(component);
        }
        let mut config = make_component("config.md");
        config.component_type = ComponentType::ConfigFile;
        app.components.push(config);
        app.available_tabs = vec![Tab::Skills, Tab::Config, Tab::McpServers, Tab::Plugins];
        app
    }

    fn press(app: &mut App, key: char) {
        handle_list_input(app, KeyCode::Char(key)).unwrap();
    }

    fn listed(app: &App) -> Vec<&str> {
        app.current_components()
            .iter()
            .map(|(_, c)| c.name.as_str())
            .collect()
    }

    fn selected(app: &App) -> Vec<&str> {
        app.components
            .iter()
            .filter(|c| c.selected)
            .map(|c| c.name.as_str())
            .collect()
    }

    #[test]
    fn cycles_file_statuses_and_restores_the_full_list() {
        let mut app = fixture();
        press(&mut app, 'f');
        assert_eq!(listed(&app), ["group/changed.md"]);
        let tree = app.get_tree_view().unwrap();
        assert_eq!(tree.visible_indices.len(), 2);
        assert!(matches!(
            tree.nodes[tree.visible_indices[0]],
            TreeNode::Folder { .. }
        ));
        press(&mut app, 'f');
        assert_eq!(listed(&app), ["group/local.md"]);
        press(&mut app, 'f');
        assert_eq!(listed(&app), ["group/new.md"]);
        press(&mut app, 'f');
        assert_eq!(
            listed(&app),
            [
                "group/changed.md",
                "group/new.md",
                "group/local.md",
                "installed.md",
                "managed.md"
            ]
        );
    }

    #[test]
    fn select_all_replaces_only_the_current_tabs_selection() {
        let mut app = fixture();
        press(&mut app, 'f');
        press(&mut app, 'a');
        assert_eq!(selected(&app), ["group/changed.md", "config.md"]);
        press(&mut app, 'f');
        press(&mut app, 'a');
        assert_eq!(selected(&app), ["group/local.md", "config.md"]);
        press(&mut app, 'n');
        assert_eq!(selected(&app), ["config.md"]);
    }

    #[test]
    fn new_filter_selects_and_installs_only_uninstalled_files() {
        let mut app = fixture();
        for _ in 0..3 {
            press(&mut app, 'f');
        }
        press(&mut app, 'a');
        assert_eq!(selected(&app), ["group/new.md", "config.md"]);
        press(&mut app, 'i');
        assert_eq!(app.processing_queue, [1]);
        assert_eq!(app.current_view, View::Installing);
    }

    #[test]
    fn folder_selection_does_not_toggle_hidden_descendants() {
        let mut app = fixture();
        press(&mut app, 'f');
        press(&mut app, ' ');
        assert_eq!(
            selected(&app),
            [
                "group/new.md",
                "group/local.md",
                "installed.md",
                "managed.md",
                "config.md"
            ]
        );
        press(&mut app, ' ');
        assert_eq!(
            selected(&app),
            [
                "group/changed.md",
                "group/new.md",
                "group/local.md",
                "installed.md",
                "managed.md",
                "config.md"
            ]
        );
    }

    #[test]
    fn queues_exclude_selected_files_hidden_by_the_filter() {
        for key in ['i', 'r'] {
            let mut app = fixture();
            press(&mut app, 'f');
            press(&mut app, key);
            assert_eq!(app.processing_queue, [0]);
            assert_eq!(app.current_view, View::Installing);
        }
        let mut app = fixture();
        press(&mut app, 'f');
        press(&mut app, 'f');
        press(&mut app, 'i');
        assert_eq!(app.current_view, View::List);
        assert_eq!(app.status_message.as_deref(), Some("No items selected"));
        press(&mut app, 'r');
        assert_eq!(app.processing_queue, [2]);
    }

    #[test]
    fn empty_filter_has_no_cursor_or_processing_target() {
        let mut app = fixture();
        app.components[0].status = InstallStatus::Unchanged;
        app.list_index = 4;
        press(&mut app, 'f');
        assert_eq!(listed(&app), Vec::<&str>::new());
        for key in ['j', 'k', ' ', 'i', 'r'] {
            press(&mut app, key);
        }
        assert_eq!(app.selected_component_index(), None);
        assert_eq!(app.current_view, View::List);
        assert!(app.processing_queue.is_empty());
        press(&mut app, 'f');
        press(&mut app, 'j');
        assert_eq!(app.selected_component_index(), Some(2));
    }

    #[test]
    fn filter_survives_tab_switches_and_status_refresh() {
        let mut app = fixture();
        press(&mut app, 'f');
        app.next_tab();
        assert_eq!(listed(&app), Vec::<&str>::new());
        app.prev_tab();
        assert_eq!(listed(&app), ["group/changed.md"]);
        let mut refreshed = app.components.clone();
        refreshed[0].status = InstallStatus::Unchanged;
        refreshed[1].status = InstallStatus::Modified;
        app.apply_components_refresh(refreshed);
        assert_eq!(listed(&app), ["group/new.md"]);
        app.next_item();
        assert_eq!(app.selected_component_index(), Some(1));
        app.select_cli(TargetCli::Codex).unwrap();
        assert_eq!(listed(&app).len(), 5);
    }

    #[test]
    fn non_file_tabs_ignore_filter_and_keep_bulk_selection() {
        let mut app = fixture();
        app.mcp_servers
            .push(crate::app::test_support::make_mcp_server());
        app.plugins.push(crate::app::test_support::make_plugin());
        for tab in [Tab::McpServers, Tab::Plugins] {
            app.tab = tab;
            press(&mut app, 'f');
            press(&mut app, 'a');
        }
        assert!(app.mcp_servers[0].selected);
        assert!(app.plugins[0].selected);
        app.tab = Tab::Skills;
        assert_eq!(listed(&app).len(), 5);
        app.focus = FocusArea::Tabs;
        press(&mut app, 'f');
        assert_eq!(listed(&app).len(), 5);
    }

    struct DiskFixture(std::path::PathBuf);

    impl Drop for DiskFixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn disk_fixture(cli: TargetCli) -> (DiskFixture, App) {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture = DiskFixture(std::env::temp_dir().join(format!("hibi_filter_{stamp}")));
        let mut app = fresh_app();
        app.target_cli = Some(cli);
        app.available_tabs = Tab::for_cli(cli);
        app.source_dir = fixture.0.join("source");
        app.dest_dir = fixture.0.join("dest");
        for dir in [&app.source_dir, &app.dest_dir] {
            std::fs::create_dir_all(dir.join("skills/demo")).unwrap();
        }
        for (name, content) in [
            ("changed.md", "updated"),
            ("new.md", "new"),
            ("same.md", "same"),
        ] {
            std::fs::write(app.source_dir.join("skills/demo").join(name), content).unwrap();
        }
        for (name, content) in [
            ("changed.md", "original"),
            ("local.md", "local"),
            ("same.md", "same"),
        ] {
            std::fs::write(app.dest_dir.join("skills/demo").join(name), content).unwrap();
        }
        let sources = [crate::source::ResolvedSource::bundled(&app.source_dir)];
        let components =
            crate::fs::scanner::scan_all_sources(&sources, &app.dest_dir, cli).unwrap();
        app.finish_loading(components, vec![], vec![], vec![]);
        (fixture, app)
    }

    fn execute_queue(app: &App) -> Vec<String> {
        app.processing_queue
            .iter()
            .map(|&index| {
                let (_tx, rx) = std::sync::mpsc::channel();
                crate::process_exec::execute(
                    crate::process_exec::prepare(app, index).unwrap(),
                    app.is_removing,
                    app.target_cli.unwrap(),
                    rx,
                )
                .unwrap()
            })
            .collect()
    }

    #[test]
    fn filtered_keys_render_and_process_real_files_for_both_clis() {
        for cli in [TargetCli::Claude, TargetCli::Codex] {
            let (_fixture, mut app) = disk_fixture(cli);
            let dest = app.dest_dir.join("skills/demo");
            for (name, action, log) in [
                ("changed.md", 'i', "[OK] Installed demo/changed.md"),
                ("local.md", 'r', "[OK] Removed demo/local.md"),
                ("new.md", 'i', "[OK] Installed demo/new.md"),
            ] {
                press(&mut app, 'f');
                press(&mut app, 'a');
                let mut terminal =
                    ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 24)).unwrap();
                terminal.draw(|f| crate::ui::draw(f, &app)).unwrap();
                let screen: String = terminal
                    .backend()
                    .buffer()
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect();
                assert!(screen.contains(name), "{screen}");
                assert!(!screen.contains("same.md"), "{screen}");
                press(&mut app, action);
                assert_eq!(execute_queue(&app), [log]);
                assert_eq!(
                    std::fs::read_to_string(dest.join("same.md")).unwrap(),
                    "same"
                );
                app.current_view = View::List;
            }
            assert_eq!(
                std::fs::read_to_string(dest.join("changed.md")).unwrap(),
                "updated"
            );
            assert_eq!(std::fs::read_to_string(dest.join("new.md")).unwrap(), "new");
            assert!(!dest.join("local.md").exists());
        }
    }

    #[test]
    fn filtered_install_reports_a_missing_source_without_overwriting_destination() {
        let (_fixture, mut app) = disk_fixture(TargetCli::Codex);
        press(&mut app, 'f');
        press(&mut app, 'a');
        std::fs::remove_file(app.source_dir.join("skills/demo/changed.md")).unwrap();
        press(&mut app, 'i');
        let logs = execute_queue(&app);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].starts_with("[ERR] demo/changed.md:"), "{logs:?}");
        assert_eq!(
            std::fs::read_to_string(app.dest_dir.join("skills/demo/changed.md")).unwrap(),
            "original"
        );
        assert!(!app.dest_dir.join("skills/demo/new.md").exists());
    }
}

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
        assert_eq!(
            app.focus,
            FocusArea::Content,
            "`2` pressed from {:?}",
            start
        );
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
        (app::View::SourceAddBranch, |a| {
            a.source_input_buffer.as_str()
        }),
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

        assert_eq!(
            buffer(&app),
            "12",
            "digits must reach the buffer in {view:?}"
        );
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
            assert_eq!(
                app.current_view,
                app::View::List,
                "{:?} must close help",
                exit
            );
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
        assert!(
            app.has_selection(),
            "{}: asking must not already discard",
            case.0
        );
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
    assert_eq!(
        app.current_view,
        app::View::List,
        "the tab bar must not leave the screen"
    );
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
            assert!(
                app.has_selection(),
                "{}: {cancel:?} must keep the tick",
                case.0
            );
            app.current_view = app::View::ConfirmExit;
        }

        handle_confirm_exit(&mut app, KeyCode::Char('y'));
        assert_eq!(app.current_view, app::View::CliSelection, "{}", case.0);
        assert!(
            !app.has_selection(),
            "{}: leaving must discard what it warned about",
            case.0
        );
    }
}
