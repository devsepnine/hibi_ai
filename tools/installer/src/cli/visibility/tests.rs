use super::*;
use crate::app::{Tab, TargetCli, View};
use crate::component::{Component, InstallStatus};
use crate::tree::{TreeNode, test_support::make_component};

struct Fixture(std::path::PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn external(name: &str, selected: bool) -> Component {
    let mut component = make_component(name);
    component.status = InstallStatus::External;
    component.selected = selected;
    component
}

fn fixture() -> (Fixture, App) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hibi_visibility_{}_{stamp}_{sequence}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = fresh_app();
    app.dest_dir = dir.join(".codex");
    app.visibility_path = dir.join("visibility.json");
    app.target_cli = Some(TargetCli::Codex);
    app.available_tabs = vec![Tab::Skills];
    app.finish_loading(
        vec![
            external(".system/tool/SKILL.md", true),
            make_component("demo/SKILL.md"),
        ],
        vec![],
        vec![],
        vec![],
    );
    (Fixture(dir), app)
}

fn key(app: &mut App, code: KeyCode) {
    let (tx, _rx) = std::sync::mpsc::channel();
    dispatch_key(app, code, &tx).unwrap();
}

fn press(app: &mut App, character: char) {
    key(app, KeyCode::Char(character));
}

fn hide(app: &mut App) {
    press(app, 'x');
    assert_eq!(app.current_view, View::ConfirmHide);
    press(app, 'y');
    assert_eq!(app.current_view, View::List);
}

fn focus_folder(app: &mut App, name: &str) {
    let tree = app.tree_views.get_mut(&app.tab).unwrap();
    tree.cursor = tree
        .visible_indices
        .iter()
        .position(
            |&index| matches!(&tree.nodes[index], TreeNode::Folder { path, .. } if path == name),
        )
        .unwrap();
}

fn screen_at_width(app: &App, width: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, 24)).unwrap();
    terminal.draw(|frame| crate::ui::draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

fn modal_at_size(app: &App, width: u16, height: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| crate::ui::confirm_hide::render(frame, app, frame.area()))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

fn screen(app: &App) -> String {
    screen_at_width(app, 140)
}

#[test]
fn cursor_hide_requests_confirmation_before_clearing_selection_and_preserves_disk_files() {
    let (_fixture, mut app) = fixture();
    let file = app.dest_dir.join("skills/.system/tool/SKILL.md");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "service owned").unwrap();
    press(&mut app, 'x');
    assert_eq!(app.current_view, View::ConfirmHide);
    let confirmation = app.hide_confirmation.as_ref().unwrap();
    assert_eq!(confirmation.folders, [".system"]);
    assert_eq!(confirmation.selected_files, 1);
    assert_eq!(confirmation.scroll, 0);
    assert!(app.components[0].selected);
    assert!(!app.visibility_path.exists());
    press(&mut app, 'y');
    assert_eq!(app.current_view, View::List);
    assert!(app.hide_confirmation.is_none());
    assert!(!screen(&app).contains(".system/"));
    assert!(screen(&app).contains("Hidden: 1"));
    assert!(screen(&app).contains("demo/"));
    assert_eq!(app.components.len(), 2);
    assert!(!app.components[0].selected);
    assert!(app.components[1].selected);
    assert_eq!(std::fs::read_to_string(file).unwrap(), "service owned");
    press(&mut app, 'r');
    assert_eq!(app.processing_queue, [1]);
}

#[test]
fn hidden_folder_survives_restart_and_unhide_does_not_restore_checks() {
    let (_fixture, mut app) = fixture();
    hide(&mut app);
    let mut restarted = fresh_app();
    restarted.dest_dir = app.dest_dir.clone();
    restarted.visibility_path = app.visibility_path.clone();
    restarted.finish_loading(app.components.clone(), vec![], vec![], vec![]);
    assert!(!screen(&restarted).contains(".system/"));
    press(&mut restarted, 'H');
    assert!(screen(&restarted).contains(".system/ [hidden]"));
    focus_folder(&mut restarted, ".system");
    press(&mut restarted, 'x');
    assert_eq!(restarted.current_view, View::List);
    assert!(restarted.hide_confirmation.is_none());
    press(&mut restarted, 'H');
    assert!(screen(&restarted).contains(".system/"));
    assert!(!restarted.components[0].selected);
    assert_eq!(
        crate::fs::installer::visibility::load(&app.visibility_path)
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn cancel_keys_preserve_files_selections_and_preferences() {
    for cancel in [KeyCode::Esc, KeyCode::Char('n'), KeyCode::Char('q')] {
        let (_fixture, mut app) = fixture();
        press(&mut app, 'x');
        key(&mut app, cancel);
        assert_eq!(app.current_view, View::List);
        assert!(app.hide_confirmation.is_none());
        assert!(app.components[0].selected);
        assert!(app.hidden_folders.is_empty());
        assert!(!app.visibility_path.exists());
        assert!(!app.should_quit);
        assert!(screen(&app).contains(".system/"));
    }
}

#[test]
fn confirmation_ignores_other_action_keys_including_enter_and_space() {
    let (_fixture, mut app) = fixture();
    press(&mut app, 'x');
    for ignored in [
        KeyCode::Enter,
        KeyCode::Char(' '),
        KeyCode::Char('i'),
        KeyCode::Char('r'),
        KeyCode::Char('a'),
        KeyCode::Char('H'),
        KeyCode::Char('x'),
        KeyCode::Char('X'),
        KeyCode::Char('E'),
        KeyCode::Tab,
    ] {
        key(&mut app, ignored);
        assert_eq!(app.current_view, View::ConfirmHide);
        assert!(app.components[0].selected);
        assert!(app.processing_queue.is_empty());
        assert!(app.hidden_folders.is_empty());
    }
    press(&mut app, 'y');
    assert!(!app.components[0].selected);
}

#[test]
fn save_failure_keeps_selection_folder_and_existing_preferences() {
    let (_fixture, mut app) = fixture();
    app.visibility_path = app.visibility_path.parent().unwrap().to_path_buf();
    press(&mut app, 'x');
    assert_eq!(app.current_view, View::ConfirmHide);
    press(&mut app, 'y');
    assert!(app.components[0].selected);
    assert!(app.hidden_folders.is_empty());
    assert!(
        app.status_message
            .as_deref()
            .unwrap()
            .starts_with("Failed to save folder visibility:")
    );
}

#[test]
fn corrupt_preferences_are_reported_and_preserved() {
    let (_fixture, mut app) = fixture();
    std::fs::write(&app.visibility_path, "invalid JSON").unwrap();
    assert!(
        crate::fs::installer::visibility::load(&app.visibility_path)
            .unwrap_err()
            .to_string()
            .starts_with("Invalid visibility settings at ")
    );
    press(&mut app, 'x');
    press(&mut app, 'y');
    assert!(app.components[0].selected);
    assert_eq!(
        std::fs::read_to_string(&app.visibility_path).unwrap(),
        "invalid JSON"
    );
}

#[test]
fn hide_rejects_mixed_folders_even_when_filter_only_shows_external_children() {
    let (_fixture, mut app) = fixture();
    app.components.push(make_component(".system/managed.md"));
    app.finish_loading(app.components.clone(), vec![], vec![], vec![]);
    press(&mut app, 'f');
    press(&mut app, 'f');
    press(&mut app, 'x');
    assert_eq!(app.current_view, View::List);
    assert_eq!(
        app.status_message.as_deref(),
        Some("Only folders containing exclusively external files can be hidden")
    );
    assert!(!app.visibility_path.exists());
}

#[test]
fn batch_hides_fully_selected_external_roots_and_deduplicates_descendants() {
    let (_fixture, mut app) = fixture();
    app.finish_loading(
        vec![
            external("parent/one/SKILL.md", true),
            external("parent/two/SKILL.md", true),
            external("synced/SKILL.md", true),
            external("unchecked/SKILL.md", false),
        ],
        vec![],
        vec![],
        vec![],
    );
    press(&mut app, 'X');
    assert_eq!(app.current_view, View::ConfirmHide);
    let confirmation = app.hide_confirmation.as_ref().unwrap();
    assert_eq!(confirmation.folders, ["parent", "synced"]);
    assert_eq!(confirmation.selected_files, 3);
    press(&mut app, 'y');
    assert_eq!(app.hidden_folder_count(), 2);
    assert_eq!(
        app.components
            .iter()
            .map(|component| component.selected)
            .collect::<Vec<_>>(),
        [false, false, false, false]
    );
    let saved = crate::fs::installer::visibility::load(&app.visibility_path).unwrap();
    assert_eq!(
        saved.into_iter().collect::<Vec<_>>(),
        [
            app.dest_dir.join("skills/parent"),
            app.dest_dir.join("skills/synced")
        ]
    );
    assert!(screen(&app).contains("unchecked/"));
}

#[test]
fn batch_skips_partial_and_mixed_parents_but_includes_eligible_child_folders() {
    let (_fixture, mut app) = fixture();
    app.finish_loading(
        vec![
            external("partial/selected.md", true),
            external("partial/unselected.md", false),
            external("mixed/local/SKILL.md", true),
            make_component("mixed/managed.md"),
            external("unselected/SKILL.md", false),
        ],
        vec![],
        vec![],
        vec![],
    );
    press(&mut app, 'X');
    assert_eq!(app.current_view, View::ConfirmHide);
    assert_eq!(
        app.hide_confirmation.as_ref().unwrap().folders,
        ["mixed/local"]
    );
    assert_eq!(app.hide_confirmation.as_ref().unwrap().selected_files, 1);
    press(&mut app, 'y');
    assert!(app.components[0].selected);
    assert!(!app.components[2].selected);
    assert!(app.components[3].selected);
    assert_eq!(app.hidden_folder_count(), 1);
    assert!(screen(&app).contains("partial/"));
}

#[test]
fn batch_respects_status_filter_and_ignores_non_component_tabs() {
    let (_fixture, mut app) = fixture();
    press(&mut app, 'f');
    press(&mut app, 'X');
    assert_eq!(app.current_view, View::List);
    assert!(app.hide_confirmation.is_none());
    assert!(app.components[0].selected);
    press(&mut app, 'f');
    press(&mut app, 'X');
    assert_eq!(app.current_view, View::ConfirmHide);
    assert_eq!(app.hide_confirmation.as_ref().unwrap().folders, [".system"]);
    press(&mut app, 'n');
    app.tab = Tab::McpServers;
    press(&mut app, 'X');
    press(&mut app, 'x');
    press(&mut app, 'H');
    assert_eq!(app.current_view, View::List);
    assert!(app.hide_confirmation.is_none());
    assert!(!app.show_hidden);
}

#[test]
fn hidden_items_are_not_bulk_selected_or_queued_until_explicitly_shown() {
    let (_fixture, mut app) = fixture();
    hide(&mut app);
    press(&mut app, 'a');
    assert_eq!(
        app.components
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        [false, false]
    );
    press(&mut app, 'a');
    assert_eq!(
        app.components
            .iter()
            .map(|component| component.selected)
            .collect::<Vec<_>>(),
        [false, true]
    );
    app.components[0].selected = true;
    press(&mut app, 'r');
    assert_eq!(app.processing_queue, [1]);
    app.close_processing();
    press(&mut app, 'i');
    assert_eq!(app.processing_queue, [1]);
    app.close_processing();
    press(&mut app, 'H');
    press(&mut app, 'a');
    assert_eq!(
        app.components
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        [false, false]
    );
    press(&mut app, 'a');
    assert_eq!(
        app.components
            .iter()
            .map(|component| component.selected)
            .collect::<Vec<_>>(),
        [true, true]
    );
    press(&mut app, 'r');
    assert_eq!(app.processing_queue, [0, 1]);
}

#[test]
fn concealing_hidden_folders_clears_their_checks_without_restoring_them() {
    let (_fixture, mut app) = fixture();
    hide(&mut app);
    let saved = std::fs::read(&app.visibility_path).unwrap();
    press(&mut app, 'H');
    press(&mut app, 'a');
    assert_eq!(
        app.components
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        [true, true]
    );
    press(&mut app, 'H');
    assert!(!app.show_hidden);
    assert_eq!(
        app.components
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        [false, true]
    );
    assert_eq!(app.hidden_selected_count(), 0);
    assert!(screen(&app).contains("Selected: 1 [0 hidden]"));
    press(&mut app, 'r');
    assert_eq!(app.processing_queue, [1]);
    app.close_processing();
    press(&mut app, 'H');
    assert!(!app.components[0].selected);
    assert!(screen(&app).contains(".system/ [hidden]"));
    assert_eq!(std::fs::read(&app.visibility_path).unwrap(), saved);
}

#[test]
fn concealing_clears_hidden_checks_across_tabs_and_status_filters_only() {
    use crate::component::ComponentType;
    let (_fixture, mut app) = fixture();
    hide(&mut app);
    let mut hidden_rule = external("private/rule.md", true);
    hidden_rule.component_type = ComponentType::Rules;
    let mut managed = make_component(".system/managed.md");
    managed.status = InstallStatus::Modified;
    app.components.extend([
        hidden_rule,
        managed,
        external(".system-other/SKILL.md", true),
    ]);
    app.hidden_folders
        .insert(app.dest_dir.join("rules/private"));
    press(&mut app, 'H');
    press(&mut app, 'a');
    app.tab = Tab::Rules;
    press(&mut app, 'f');
    press(&mut app, 'H');
    assert_eq!(
        app.components
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        [false, true, false, true, true]
    );
    app.tab = Tab::Skills;
    press(&mut app, 'H');
    assert_eq!(
        app.components
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        [false, true, false, true, true]
    );
}

#[test]
fn confirmation_names_counts_and_keys_fit_an_eighty_column_terminal() {
    let (_fixture, mut app) = fixture();
    press(&mut app, 'x');
    let rendered = screen_at_width(&app, 80);
    for expected in [
        "Confirm hide",
        "Hide 1 folder(s)?",
        "Clear 1 selected file(s).",
        "Files remain on disk.",
        ".system",
        "[y] Hide",
        "[Esc/n/q] Cancel",
    ] {
        assert!(
            rendered.contains(expected),
            "{expected:?} missing from {rendered}"
        );
    }
}

#[test]
fn batch_confirmation_scrolls_to_last_folder_and_clamps_at_both_ends() {
    let (_fixture, mut app) = fixture();
    let components = (0..30)
        .map(|index| external(&format!("folder{index:02}/SKILL.md"), true))
        .collect();
    app.finish_loading(components, vec![], vec![], vec![]);
    press(&mut app, 'X');
    assert_eq!(app.hide_confirmation.as_ref().unwrap().folders.len(), 30);
    assert_eq!(app.hide_confirmation.as_ref().unwrap().selected_files, 30);
    assert!(screen_at_width(&app, 80).contains("folder00"));
    assert!(!screen_at_width(&app, 80).contains("folder29"));
    for _ in 0..40 {
        press(&mut app, 'j');
    }
    let bottom = app.hide_confirmation.as_ref().unwrap().scroll;
    assert!(bottom > 0);
    assert!(screen_at_width(&app, 80).contains("folder29"));
    for _ in 0..40 {
        press(&mut app, 'j');
    }
    assert_eq!(app.hide_confirmation.as_ref().unwrap().scroll, bottom);
    for _ in 0..40 {
        key(&mut app, KeyCode::Up);
    }
    assert_eq!(app.hide_confirmation.as_ref().unwrap().scroll, 0);
    assert!(screen_at_width(&app, 80).contains("Hide 30 folder(s)?"));
    assert!(screen_at_width(&app, 80).contains("folder00"));
    press(&mut app, 'q');
    assert!(app.components.iter().all(|component| component.selected));
    assert!(app.hidden_folders.is_empty());
}

#[test]
fn confirmation_wraps_long_shared_prefixes_so_folder_suffixes_are_readable() {
    let (_fixture, mut app) = fixture();
    let prefix = "shared".repeat(14);
    app.finish_loading(
        vec![
            external(&format!("{prefix}/alpha/SKILL.md"), true),
            external(&format!("{prefix}/beta/SKILL.md"), true),
            external(&format!("{prefix}/unchecked.md"), false),
        ],
        vec![],
        vec![],
        vec![],
    );
    press(&mut app, 'X');
    assert_eq!(app.hide_confirmation.as_ref().unwrap().folders.len(), 2);
    let mut frames = String::new();
    for _ in 0..40 {
        frames.push_str(&modal_at_size(&app, 80, 24));
        press(&mut app, 'j');
    }
    assert!(frames.contains("alpha/"), "{frames}");
    assert!(frames.contains("beta/"), "{frames}");
}

#[test]
fn tiny_confirmation_can_scroll_from_summary_through_every_folder() {
    let (_fixture, mut app) = fixture();
    app.finish_loading(
        vec![
            external("alpha/SKILL.md", true),
            external("beta/SKILL.md", true),
        ],
        vec![],
        vec![],
        vec![],
    );
    press(&mut app, 'X');
    let max = crate::ui::confirm_hide::max_scroll(&app, 80, 6);
    assert!(max > 0);
    let mut frames = String::new();
    for _ in 0..max + 3 {
        frames.push_str(&modal_at_size(&app, 80, 6));
        app.scroll_folder_hide_down(max);
    }
    for expected in [
        "Hide 2 folder(s)?",
        "Clear 2 selected file(s).",
        "Files remain on disk.",
        "alpha/",
        "beta/",
    ] {
        assert!(
            frames.contains(expected),
            "{expected:?} missing from {frames}"
        );
    }
    let bottom = app.hide_confirmation.as_ref().unwrap().scroll;
    app.scroll_folder_hide_down(max);
    assert_eq!(app.hide_confirmation.as_ref().unwrap().scroll, bottom);
    for _ in 0..max + 3 {
        app.scroll_folder_hide_up(max);
    }
    assert_eq!(app.hide_confirmation.as_ref().unwrap().scroll, 0);
    assert!(modal_at_size(&app, 80, 6).contains("Hide 2 folder(s)?"));
}

#[test]
fn scrolling_up_after_terminal_grows_moves_the_currently_visible_content() {
    let (_fixture, mut app) = fixture();
    let components = (0..30)
        .map(|index| external(&format!("folder{index:02}/SKILL.md"), true))
        .collect();
    app.finish_loading(components, vec![], vec![], vec![]);
    press(&mut app, 'X');
    let small_max = crate::ui::confirm_hide::max_scroll(&app, 80, 6);
    let large_max = crate::ui::confirm_hide::max_scroll(&app, 80, 24);
    assert!(small_max > large_max);
    for _ in 0..small_max + 3 {
        app.scroll_folder_hide_down(small_max);
    }
    assert_eq!(app.hide_confirmation.as_ref().unwrap().scroll, small_max);
    let before = modal_at_size(&app, 80, 24);
    assert!(before.contains("folder29"));
    app.scroll_folder_hide_up(large_max);
    assert_eq!(
        app.hide_confirmation.as_ref().unwrap().scroll,
        large_max.saturating_sub(1)
    );
    let after = modal_at_size(&app, 80, 24);
    assert_ne!(before, after);
    assert!(after.contains("folder28"));
    assert!(!after.contains("folder29"));
}

#[test]
fn cursor_hide_of_unchecked_folder_confirms_zero_checks_without_selecting_it() {
    let (_fixture, mut app) = fixture();
    app.components[0].selected = false;
    press(&mut app, 'x');
    assert_eq!(app.hide_confirmation.as_ref().unwrap().selected_files, 0);
    press(&mut app, 'y');
    assert!(!app.components[0].selected);
    assert_eq!(app.hidden_folder_count(), 1);
}

#[test]
fn hidden_paths_are_target_scoped_and_do_not_hide_new_managed_files() {
    let (_fixture, mut app) = fixture();
    hide(&mut app);
    app.components[0].status = InstallStatus::Modified;
    app.apply_components_refresh(app.components.clone());
    assert!(screen(&app).contains(".system/"));
    app.components[0].status = InstallStatus::External;
    app.dest_dir = app.dest_dir.with_file_name(".claude");
    app.apply_components_refresh(app.components.clone());
    assert!(screen(&app).contains(".system/"));
}

#[test]
fn nested_hidden_folder_requires_unhiding_the_parent_first() {
    let (_fixture, mut app) = fixture();
    focus_folder(&mut app, ".system/tool");
    hide(&mut app);
    press(&mut app, 'H');
    focus_folder(&mut app, ".system");
    hide(&mut app);
    focus_folder(&mut app, ".system/tool");
    press(&mut app, 'x');
    assert!(
        app.status_message
            .as_deref()
            .unwrap()
            .ends_with("Unhide that parent folder first.")
    );
    assert_eq!(app.hidden_folder_count(), 2);
    focus_folder(&mut app, ".system");
    press(&mut app, 'x');
    focus_folder(&mut app, ".system/tool");
    press(&mut app, 'x');
    assert_eq!(app.hidden_folder_count(), 0);
    assert!(!app.components[0].selected);
}

#[test]
fn changed_preferences_require_reload_and_load_errors_reach_the_list() {
    let (_fixture, mut app) = fixture();
    let other_folder = app.dest_dir.join("skills/synced");
    std::fs::write(
        &app.visibility_path,
        serde_json::to_vec(&[other_folder.clone()]).unwrap(),
    )
    .unwrap();
    press(&mut app, 'x');
    press(&mut app, 'y');
    assert!(
        app.status_message
            .as_deref()
            .unwrap()
            .contains("changed in another session")
    );
    assert!(app.components[0].selected);
    app.finish_loading(app.components.clone(), vec![], vec![], vec![]);
    assert_eq!(
        app.hidden_folders.iter().collect::<Vec<_>>(),
        [&other_folder]
    );
    hide(&mut app);
    assert_eq!(app.hidden_folder_count(), 2);
    std::fs::write(&app.visibility_path, "invalid").unwrap();
    app.finish_loading(app.components.clone(), vec![], vec![], vec![]);
    assert!(
        app.status_message
            .as_deref()
            .unwrap()
            .contains("Failed to load folder visibility:")
    );
}

#[test]
fn empty_hidden_list_recovers_and_file_rows_do_not_open_confirmation() {
    let (_fixture, mut app) = fixture();
    app.components.truncate(1);
    app.apply_components_refresh(app.components.clone());
    hide(&mut app);
    assert!(screen(&app).contains("Press H to show hidden folders."));
    press(&mut app, 'x');
    assert_eq!(app.current_view, View::List);
    press(&mut app, 'H');
    press(&mut app, 'j');
    press(&mut app, 'j');
    assert_eq!(app.selected_component_index(), Some(0));
    press(&mut app, 'x');
    assert_eq!(app.current_view, View::List);
    assert!(app.hide_confirmation.is_none());
    assert_eq!(
        app.status_message.as_deref(),
        Some("Select an external folder to hide or unhide")
    );
}

#[test]
fn visibility_hints_account_for_status_filter_and_already_shown_state() {
    let (_fixture, mut app) = fixture();
    hide(&mut app);
    assert!(screen_at_width(&app, 80).contains("[H] Hidden"));
    press(&mut app, 'f');
    press(&mut app, 'H');
    assert!(
        app.status_message
            .as_deref()
            .unwrap()
            .contains("select All or External")
    );
    assert!(!screen(&app).contains("Press H to show hidden folders."));
}

#[test]
fn batch_preserves_already_hidden_selected_descendants() {
    let (_fixture, mut app) = fixture();
    app.finish_loading(
        vec![
            external("parent/hidden/SKILL.md", true),
            external("parent/visible/SKILL.md", true),
        ],
        vec![],
        vec![],
        vec![],
    );
    focus_folder(&mut app, "parent/hidden");
    hide(&mut app);
    press(&mut app, 'H');
    press(&mut app, 'a');
    assert!(app.components[0].selected);
    press(&mut app, 'X');
    assert_eq!(
        app.hide_confirmation.as_ref().unwrap().folders,
        ["parent/visible"]
    );
    press(&mut app, 'y');
    assert!(app.components[0].selected);
    assert!(!app.components[1].selected);
}

#[test]
fn collapsed_batch_includes_children_and_preserves_prefix_siblings() {
    let (_fixture, mut app) = fixture();
    app.finish_loading(
        vec![
            external("same/inner/SKILL.md", true),
            external("same2/SKILL.md", false),
        ],
        vec![],
        vec![],
        vec![],
    );
    focus_folder(&mut app, "same");
    press(&mut app, 'h');
    press(&mut app, 'X');
    assert_eq!(app.hide_confirmation.as_ref().unwrap().folders, ["same"]);
    assert_eq!(app.hide_confirmation.as_ref().unwrap().selected_files, 1);
    press(&mut app, 'y');
    assert_eq!(app.hidden_folder_count(), 1);
    assert!(screen(&app).contains("same2/"));
    assert!(!app.is_component_hidden(&app.components[1]));
}

#[test]
fn batch_handles_two_thousand_external_folders() {
    let (_fixture, mut app) = fixture();
    let components = (0..2000)
        .map(|index| external(&format!("folder{index:04}/SKILL.md"), true))
        .collect();
    app.finish_loading(components, vec![], vec![], vec![]);
    let started = std::time::Instant::now();
    press(&mut app, 'X');
    let elapsed = started.elapsed();
    let pending = app.hide_confirmation.as_ref().unwrap();
    assert_eq!(pending.folders.len(), 2000);
    assert_eq!(pending.selected_files, 2000);
    assert_eq!(
        pending.folders.first().map(String::as_str),
        Some("folder0000")
    );
    assert_eq!(
        pending.folders.last().map(String::as_str),
        Some("folder1999")
    );
    eprintln!("batch selection for 2000 folders: {elapsed:?}");
    let started = std::time::Instant::now();
    press(&mut app, 'y');
    assert_eq!(app.current_view, View::List);
    assert_eq!(app.components.iter().filter(|c| c.selected).count(), 0);
    assert_eq!(
        crate::fs::installer::visibility::load(&app.visibility_path)
            .unwrap()
            .len(),
        2000
    );
    eprintln!(
        "batch confirmation for 2000 folders: {:?}",
        started.elapsed()
    );
}

#[test]
fn changed_second_batch_folder_cancels_entire_transaction() {
    let (_fixture, mut app) = fixture();
    app.finish_loading(
        vec![
            external("one/SKILL.md", true),
            external("two/SKILL.md", true),
        ],
        vec![],
        vec![],
        vec![],
    );
    press(&mut app, 'X');
    assert_eq!(
        app.hide_confirmation.as_ref().unwrap().folders,
        ["one", "two"]
    );
    app.components[1].status = InstallStatus::Modified;
    press(&mut app, 'y');
    assert_eq!(app.hidden_folder_count(), 0);
    assert!(app.components[0].selected);
    assert!(app.components[1].selected);
    assert!(!app.visibility_path.exists());
    assert_eq!(app.current_view, View::List);
    assert!(
        app.status_message
            .as_deref()
            .unwrap()
            .contains("Folder contents changed")
    );
}
