use super::Desktop;
use crate::app::{App, ComponentFilter, HiddenVisibility};
use crate::component::InstallStatus;
use crate::gui::actions::Action;

fn indices(app: &App) -> Vec<usize> {
    app.visible_components()
        .iter()
        .map(|(index, _)| *index)
        .collect()
}

#[test]
fn unified_filters_keep_visibility_in_the_overlay_and_count_active_conditions() {
    let mut desktop = Desktop::new();
    desktop
        .state
        .app
        .as_mut()
        .unwrap()
        .set_hidden_visibility(HiddenVisibility::Exclude);
    let labels = desktop.labels();
    assert!(!labels.0.iter().any(|(label, _)| label == "Visible only"));
    desktop.click("Filters · Default");
    assert!(
        desktop
            .labels()
            .0
            .iter()
            .any(|(label, _)| label == "Status")
    );
    desktop.click("Modified");
    desktop.click("New");
    desktop.click("Hidden only");
    assert_eq!(desktop.filters_label(), "Filters · 3");
    assert!(desktop.state.filter_choice.is_some());
    desktop.click("Default");
    assert_eq!(desktop.filters_label(), "Filters · 1");
    desktop.click("Visible only");
    assert_eq!(desktop.filters_label(), "Filters · Default");
    desktop.close_filters();
    assert!(
        !desktop
            .labels()
            .0
            .iter()
            .any(|(label, _)| label == "Hidden only")
    );
}

#[test]
fn unified_menu_stays_inside_a_small_window_without_moving_the_list() {
    let mut desktop = Desktop::new();
    desktop.state.window_size = iced::Size::new(760.0, 560.0);
    let before = desktop
        .labels()
        .0
        .iter()
        .find(|(label, _)| label == "Component")
        .unwrap()
        .1;
    desktop.open_filters();
    let labels = desktop.labels();
    let menu = labels
        .0
        .iter()
        .find(|(label, _)| label == "[filters-menu]")
        .unwrap()
        .1;
    let after = labels
        .0
        .iter()
        .find(|(label, _)| label == "Component")
        .unwrap()
        .1;
    assert_eq!(before, after);
    assert!(menu.x >= 0.0 && menu.y >= 0.0);
    assert!(menu.x + menu.width <= 760.0 && menu.y + menu.height <= 560.0);
}

#[test]
fn multiple_statuses_are_unioned_and_hidden_visibility_is_a_separate_intersection() {
    let mut desktop = Desktop::new();
    let app = desktop.state.app.as_mut().unwrap();
    app.components[0].status = InstallStatus::External;
    app.components[1].status = InstallStatus::Modified;
    app.components[3].status = InstallStatus::Unchanged;
    app.hidden_visibility = HiddenVisibility::Exclude;
    app.apply_components_refresh(app.components.clone());
    assert_eq!(indices(desktop.app()), [3]);
    desktop.choose_visibility("Include hidden");
    assert_eq!(indices(desktop.app()), [1, 2, 3]);
    desktop
        .state
        .dispatch(Action::SetFilter(ComponentFilter::External, true));
    desktop
        .state
        .dispatch(Action::SetFilter(ComponentFilter::New, true));
    assert_eq!(indices(desktop.app()), [0, 2]);
    desktop.choose_visibility("Visible only");
    assert_eq!(indices(desktop.app()), [0]);
    desktop
        .state
        .dispatch(Action::SetFilter(ComponentFilter::Default, true));
    assert_eq!(indices(desktop.app()), [3]);
}

#[test]
fn status_dropdown_keeps_multiple_checkboxes_open_and_default_resets_them() {
    let mut desktop = Desktop::new();
    desktop.open_filters();
    desktop.click("Modified");
    desktop.click("New");
    assert!(
        desktop
            .labels()
            .0
            .iter()
            .any(|(label, _)| label == "Filters · 3")
    );
    desktop.click("Default");
    assert!(
        desktop
            .labels()
            .0
            .iter()
            .any(|(label, _)| label == "Filters · 1")
    );
    desktop.click("New");
    desktop.click("New");
    assert!(
        desktop
            .labels()
            .0
            .iter()
            .any(|(label, _)| label == "Filters · 1")
    );
}

#[test]
fn escape_and_selecting_a_row_close_the_status_menu_without_losing_selection() {
    let mut desktop = Desktop::new();
    desktop.open_filters();
    assert!(desktop.state.filter_choice.is_some());
    desktop
        .state
        .dispatch(Action::Key(crossterm::event::KeyCode::Esc));
    assert!(desktop.state.filter_choice.is_none());
    assert_eq!(desktop.app().current_view, crate::app::View::List);
    desktop.open_filters();
    desktop.click("visible");
    assert!(desktop.state.filter_choice.is_none());
    assert!(desktop.app().components[0].selected);
}

#[test]
fn hidden_only_bounds_work_queues_and_search_without_touching_visible_files() {
    let mut desktop = Desktop::new();
    desktop.choose_visibility("Hidden only");
    let app = desktop.state.app.as_mut().unwrap();
    app.install_selected().unwrap();
    assert_eq!(app.processing_queue, [1, 2]);
    app.close_processing();
    app.components[1].status = crate::component::InstallStatus::Modified;
    app.components[2].status = crate::component::InstallStatus::Modified;
    app.request_removal();
    assert_eq!(app.pending_removal_items().len(), 2);
    app.confirm_removal().unwrap();
    assert_eq!(app.processing_queue, [1, 2]);
    app.close_processing();
    app.set_search_query("other-hidden".into());
    assert_eq!(indices(app), [2]);
    app.set_search_query("visible/a".into());
    assert!(app.visible_components().is_empty());
}

#[test]
fn hidden_only_does_not_label_an_empty_mcp_category_as_hidden() {
    let mut desktop = Desktop::new();
    desktop.choose_visibility("Hidden only");
    desktop.state.app.as_mut().unwrap().tab = crate::app::Tab::McpServers;
    let labels = desktop.labels();
    assert!(
        labels
            .0
            .iter()
            .any(|(label, _)| label == "Nothing to show here yet")
    );
    assert!(
        !labels
            .0
            .iter()
            .any(|(label, _)| label == "No hidden items in this view")
    );
}

#[test]
fn opening_status_dropdown_does_not_move_the_component_list() {
    let mut desktop = Desktop::new();
    let before = desktop
        .labels()
        .0
        .iter()
        .find(|(label, _)| label == "Component")
        .unwrap()
        .1;
    desktop.open_filters();
    let after = desktop
        .labels()
        .0
        .iter()
        .find(|(label, _)| label == "Component")
        .unwrap()
        .1;
    assert_eq!(before, after);
}

#[test]
fn clicking_outside_the_dropdown_closes_it_without_selecting_the_row_underneath() {
    let mut desktop = Desktop::new();
    desktop.open_filters();
    desktop.click("other-hidden");
    assert!(desktop.state.filter_choice.is_none());
    assert_eq!(desktop.app().get_tree_view().unwrap().cursor, 0);
}

#[test]
fn hidden_only_excludes_visible_items_and_view_changes_do_not_save_settings() {
    let mut desktop = Desktop::new();
    let before = desktop.saved();
    desktop.choose_visibility("Hidden only");
    assert_eq!(indices(desktop.app()), [1, 2]);
    desktop.choose_visibility("Visible only");
    assert_eq!(indices(desktop.app()), [0, 3]);
    desktop.choose_visibility("Include hidden");
    assert_eq!(indices(desktop.app()), [0, 1, 2, 3]);
    assert_eq!(desktop.saved(), before);
}

#[test]
fn hidden_only_intersects_status_filter_and_unhide_removes_items_from_the_view() {
    let mut desktop = Desktop::new();
    desktop.state.app.as_mut().unwrap().components[2].status = InstallStatus::External;
    desktop.choose_visibility("Hidden only");
    desktop
        .state
        .dispatch(Action::SetFilter(ComponentFilter::External, true));
    assert_eq!(indices(desktop.app()), [2]);
    desktop.click("Unhide selected · 1");
    assert!(desktop.app().visible_components().is_empty());
    assert_eq!(desktop.saved().len(), 1);
    assert!(
        desktop
            .labels()
            .0
            .iter()
            .any(|(label, _)| label == "No hidden items in this view")
    );
}
