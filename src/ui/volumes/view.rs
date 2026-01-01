use gpui::{div, prelude::*, px, rgb, rgba, Context, Entity, Render, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    notification::NotificationType,
    v_flex, Sizable, WindowExt,
};

use crate::docker::VolumeInfo;
use crate::services::{self, dispatcher, DispatcherEvent};
use crate::state::{docker_state, DockerState, StateChanged};

use super::create_dialog::CreateVolumeDialog;
use super::detail::{VolumeDetail, VolumeTabState};
use super::list::{VolumeList, VolumeListEvent};

/// Self-contained Volumes view - handles list, detail, and all state
pub struct VolumesView {
    docker_state: Entity<DockerState>,
    volume_list: Entity<VolumeList>,
    selected_volume: Option<VolumeInfo>,
    active_tab: usize,
    volume_tab_state: VolumeTabState,
    pending_notifications: Vec<(NotificationType, String)>,
    create_dialog: Option<Entity<CreateVolumeDialog>>,
}

impl VolumesView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let docker_state = docker_state(cx);

        // Create volume list entity
        let volume_list = cx.new(|cx| VolumeList::new(window, cx));

        // Subscribe to volume list events
        cx.subscribe(&volume_list, |this, _list, event: &VolumeListEvent, cx| {
            match event {
                VolumeListEvent::Selected(volume) => {
                    this.on_select_volume(volume, cx);
                }
                VolumeListEvent::NewVolume => {
                    this.show_create_dialog(cx);
                }
            }
        })
        .detach();

        // Subscribe to state changes
        cx.subscribe(&docker_state, |this, state, event: &StateChanged, cx| {
            match event {
                StateChanged::VolumesUpdated => {
                    // If selected volume was deleted, clear selection
                    if let Some(ref selected) = this.selected_volume {
                        let state = state.read(cx);
                        if !state.volumes.iter().any(|v| v.name == selected.name) {
                            this.selected_volume = None;
                            this.active_tab = 0;
                            this.volume_tab_state = VolumeTabState::new();
                        } else {
                            // Update the selected volume info
                            if let Some(updated) =
                                state.volumes.iter().find(|v| v.name == selected.name)
                            {
                                this.selected_volume = Some(updated.clone());
                            }
                        }
                    }
                    cx.notify();
                }
                StateChanged::VolumeFilesLoaded { volume_name, path, files } => {
                    // Update file state if this is for the currently selected volume
                    if let Some(ref selected) = this.selected_volume {
                        if &selected.name == volume_name {
                            this.volume_tab_state.files = files.clone();
                            this.volume_tab_state.current_path = path.clone();
                            this.volume_tab_state.files_loading = false;
                            cx.notify();
                        }
                    }
                }
                StateChanged::VolumeFilesError { volume_name, error: _ } => {
                    // Handle error for the currently selected volume
                    if let Some(ref selected) = this.selected_volume {
                        if &selected.name == volume_name {
                            this.volume_tab_state.files_loading = false;
                            this.volume_tab_state.files = vec![];
                            cx.notify();
                        }
                    }
                }
                _ => {}
            }
        })
        .detach();

        // Subscribe to dispatcher events for notifications
        let disp = dispatcher(cx);
        cx.subscribe(&disp, |this, _disp, event: &DispatcherEvent, cx| {
            match event {
                DispatcherEvent::TaskCompleted { name: _, message } => {
                    this.pending_notifications
                        .push((NotificationType::Success, message.clone()));
                }
                DispatcherEvent::TaskFailed { name: _, error } => {
                    this.pending_notifications
                        .push((NotificationType::Error, error.clone()));
                }
            }
            cx.notify();
        })
        .detach();

        Self {
            docker_state,
            volume_list,
            selected_volume: None,
            active_tab: 0,
            volume_tab_state: VolumeTabState::new(),
            pending_notifications: Vec::new(),
            create_dialog: None,
        }
    }

    fn show_create_dialog(&mut self, cx: &mut Context<Self>) {
        self.create_dialog = Some(cx.new(|cx| CreateVolumeDialog::new(cx)));
        cx.notify();
    }

    fn on_select_volume(&mut self, volume: &VolumeInfo, cx: &mut Context<Self>) {
        self.selected_volume = Some(volume.clone());
        self.active_tab = 0;
        // Reset file state when selecting new volume
        self.volume_tab_state = VolumeTabState::new();
        cx.notify();
    }

    fn on_tab_change(&mut self, tab: usize, cx: &mut Context<Self>) {
        self.active_tab = tab;
        // Load files when switching to Files tab
        if tab == 1 {
            self.load_volume_files("/", cx);
        }
        cx.notify();
    }

    fn load_volume_files(&mut self, path: &str, cx: &mut Context<Self>) {
        if let Some(ref volume) = self.selected_volume {
            self.volume_tab_state.files_loading = true;
            self.volume_tab_state.current_path = path.to_string();
            cx.notify();

            let volume_name = volume.name.clone();
            let path = path.to_string();
            services::list_volume_files(volume_name, path, cx);
        }
    }

    fn on_navigate_path(&mut self, path: &str, cx: &mut Context<Self>) {
        self.load_volume_files(path, cx);
    }

    fn render_create_dialog(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        self.create_dialog.clone().map(|dialog_entity| {
            div()
                .id("dialog-overlay")
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(rgba(0x00000080))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .id("dialog-container")
                        .on_mouse_down_out(cx.listener(|this, _ev, _window, cx| {
                            this.create_dialog = None;
                            cx.notify();
                        }))
                        .child(
                            v_flex()
                                .w(px(500.))
                                .bg(rgb(0x24283b))
                                .rounded(px(12.))
                                .overflow_hidden()
                                .border_1()
                                .border_color(rgb(0x414868))
                                // Header
                                .child(
                                    div()
                                        .w_full()
                                        .py(px(16.))
                                        .px(px(20.))
                                        .border_b_1()
                                        .border_color(rgb(0x414868))
                                        .child(Label::new("New Volume").text_color(rgb(0xc0caf5))),
                                )
                                // Form content
                                .child(dialog_entity.clone())
                                // Footer buttons
                                .child(
                                    h_flex()
                                        .w_full()
                                        .py(px(16.))
                                        .px(px(20.))
                                        .justify_end()
                                        .gap(px(12.))
                                        .border_t_1()
                                        .border_color(rgb(0x414868))
                                        .child(
                                            Button::new("cancel")
                                                .label("Cancel")
                                                .ghost()
                                                .on_click(cx.listener(|this, _ev, _window, cx| {
                                                    this.create_dialog = None;
                                                    cx.notify();
                                                })),
                                        )
                                        .child({
                                            let dialog = dialog_entity.clone();
                                            Button::new("create")
                                                .label("Create")
                                                .primary()
                                                .on_click(cx.listener(move |this, _ev, _window, cx| {
                                                    let options = dialog.read(cx).get_options(cx);
                                                    if !options.name.is_empty() {
                                                        services::create_volume(
                                                            options.name,
                                                            options.driver.as_docker_arg().to_string(),
                                                            options.labels,
                                                            cx,
                                                        );
                                                        this.create_dialog = None;
                                                        cx.notify();
                                                    }
                                                }))
                                        }),
                                ),
                        ),
                )
        })
    }
}

impl Render for VolumesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Push any pending notifications
        for (notification_type, message) in self.pending_notifications.drain(..) {
            use gpui::SharedString;
            window.push_notification((notification_type, SharedString::from(message)), cx);
        }

        let selected_volume = self.selected_volume.clone();
        let active_tab = self.active_tab;

        // Build detail panel
        let detail = VolumeDetail::new()
            .volume(selected_volume)
            .active_tab(active_tab)
            .volume_state(self.volume_tab_state.clone())
            .on_tab_change(cx.listener(|this, tab: &usize, _window, cx| {
                this.on_tab_change(*tab, cx);
            }))
            .on_navigate_path(cx.listener(|this, path: &str, _window, cx| {
                this.on_navigate_path(path, cx);
            }))
            .on_delete(cx.listener(|this, name: &str, _window, cx| {
                services::delete_volume(name.to_string(), cx);
                this.selected_volume = None;
                this.active_tab = 0;
                cx.notify();
            }));

        // Render dialog overlay if open
        let create_dialog = self.render_create_dialog(cx);

        div()
            .size_full()
            .flex()
            .overflow_hidden()
            .child(
                // Left: Volume list - fixed width with border
                div()
                    .w(px(320.))
                    .h_full()
                    .flex_shrink_0()
                    .overflow_hidden()
                    .border_r_1()
                    .border_color(rgb(0x414868))
                    .child(self.volume_list.clone()),
            )
            .child(
                // Right: Detail panel - flexible width
                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(detail.render(window, cx)),
            )
            .children(create_dialog)
    }
}
