use gpui::{div, prelude::*, px, rgb, Context, Entity, Render, Styled, Window};
use gpui_component::notification::NotificationType;
use gpui_component::WindowExt;

use crate::docker::VolumeInfo;
use crate::services::{self, dispatcher, DispatcherEvent};
use crate::state::{docker_state, DockerState, StateChanged};

use super::detail::VolumeDetail;
use super::list::{VolumeList, VolumeListEvent};

/// Self-contained Volumes view - handles list, detail, and all state
pub struct VolumesView {
    docker_state: Entity<DockerState>,
    volume_list: Entity<VolumeList>,
    selected_volume: Option<VolumeInfo>,
    active_tab: usize,
    pending_notifications: Vec<(NotificationType, String)>,
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
            pending_notifications: Vec::new(),
        }
    }

    fn on_select_volume(&mut self, volume: &VolumeInfo, cx: &mut Context<Self>) {
        self.selected_volume = Some(volume.clone());
        self.active_tab = 0;
        cx.notify();
    }

    fn on_tab_change(&mut self, tab: usize, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
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
            .on_tab_change(cx.listener(|this, tab: &usize, _window, cx| {
                this.on_tab_change(*tab, cx);
            }))
            .on_delete(cx.listener(|this, name: &str, _window, cx| {
                services::delete_volume(name.to_string(), cx);
                this.selected_volume = None;
                this.active_tab = 0;
                cx.notify();
            }));

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
    }
}
