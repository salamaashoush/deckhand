use gpui::{div, prelude::*, px, rgb, rgba, Context, Entity, Render, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    notification::NotificationType,
    v_flex, Sizable, WindowExt,
};

use crate::docker::NetworkInfo;
use crate::services::{self, dispatcher, DispatcherEvent};
use crate::state::{docker_state, DockerState, StateChanged};

use super::create_dialog::CreateNetworkDialog;
use super::detail::NetworkDetail;
use super::list::{NetworkList, NetworkListEvent};

/// Self-contained Networks view - handles list, detail, and all state
pub struct NetworksView {
    docker_state: Entity<DockerState>,
    network_list: Entity<NetworkList>,
    selected_network: Option<NetworkInfo>,
    active_tab: usize,
    pending_notifications: Vec<(NotificationType, String)>,
    create_dialog: Option<Entity<CreateNetworkDialog>>,
}

impl NetworksView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let docker_state = docker_state(cx);

        // Create network list entity
        let network_list = cx.new(|cx| NetworkList::new(window, cx));

        // Subscribe to network list events
        cx.subscribe(&network_list, |this, _list, event: &NetworkListEvent, cx| {
            match event {
                NetworkListEvent::Selected(network) => {
                    this.on_select_network(network, cx);
                }
                NetworkListEvent::CreateNetwork => {
                    this.show_create_dialog(cx);
                }
            }
        })
        .detach();

        // Subscribe to state changes
        cx.subscribe(&docker_state, |this, state, event: &StateChanged, cx| {
            match event {
                StateChanged::NetworksUpdated => {
                    // If selected network was deleted, clear selection
                    if let Some(ref selected) = this.selected_network {
                        let state = state.read(cx);
                        if !state.networks.iter().any(|n| n.id == selected.id) {
                            this.selected_network = None;
                            this.active_tab = 0;
                        } else {
                            // Update the selected network info
                            if let Some(updated) =
                                state.networks.iter().find(|n| n.id == selected.id)
                            {
                                this.selected_network = Some(updated.clone());
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
            network_list,
            selected_network: None,
            active_tab: 0,
            pending_notifications: Vec::new(),
            create_dialog: None,
        }
    }

    fn show_create_dialog(&mut self, cx: &mut Context<Self>) {
        self.create_dialog = Some(cx.new(|cx| CreateNetworkDialog::new(cx)));
        cx.notify();
    }

    fn on_select_network(&mut self, network: &NetworkInfo, cx: &mut Context<Self>) {
        self.selected_network = Some(network.clone());
        self.active_tab = 0;
        cx.notify();
    }

    fn on_tab_change(&mut self, tab: usize, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
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
                                    v_flex()
                                        .w_full()
                                        .py(px(16.))
                                        .px(px(20.))
                                        .gap(px(4.))
                                        .border_b_1()
                                        .border_color(rgb(0x414868))
                                        .child(
                                            Label::new("New Network")
                                                .text_color(rgb(0xc0caf5))
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(0x9aa5ce))
                                                .child("Networks are groups of containers in the same subnet (IP range) that can communicate with each other. They are typically used by Compose, and don't need to be manually created or deleted."),
                                        ),
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
                                                        services::create_network(
                                                            options.name,
                                                            options.enable_ipv6,
                                                            options.subnet,
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

impl Render for NetworksView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Push any pending notifications
        for (notification_type, message) in self.pending_notifications.drain(..) {
            use gpui::SharedString;
            window.push_notification((notification_type, SharedString::from(message)), cx);
        }

        let selected_network = self.selected_network.clone();
        let active_tab = self.active_tab;

        // Build detail panel
        let detail = NetworkDetail::new()
            .network(selected_network)
            .active_tab(active_tab)
            .on_tab_change(cx.listener(|this, tab: &usize, _window, cx| {
                this.on_tab_change(*tab, cx);
            }))
            .on_delete(cx.listener(|this, id: &str, _window, cx| {
                services::delete_network(id.to_string(), cx);
                this.selected_network = None;
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
                // Left: Network list - fixed width with border
                div()
                    .w(px(320.))
                    .h_full()
                    .flex_shrink_0()
                    .overflow_hidden()
                    .border_r_1()
                    .border_color(rgb(0x414868))
                    .child(self.network_list.clone()),
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
