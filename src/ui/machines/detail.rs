use gpui::{div, prelude::*, px, App, Entity, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    tab::{Tab, TabBar},
    theme::ActiveTheme,
    v_flex, Icon, IconName, Sizable,
};
use std::rc::Rc;

use crate::colima::{ColimaVm, VmFileEntry};
use crate::state::MachineTabState;
use crate::terminal::TerminalView;

type MachineActionCallback = Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
type TabChangeCallback = Rc<dyn Fn(&usize, &mut Window, &mut App) + 'static>;
type FileNavigateCallback = Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
type RefreshCallback = Rc<dyn Fn(&(), &mut Window, &mut App) + 'static>;

pub struct MachineDetail {
    machine: Option<ColimaVm>,
    active_tab: usize,
    machine_state: Option<MachineTabState>,
    terminal_view: Option<Entity<TerminalView>>,
    on_start: Option<MachineActionCallback>,
    on_stop: Option<MachineActionCallback>,
    on_restart: Option<MachineActionCallback>,
    on_delete: Option<MachineActionCallback>,
    on_tab_change: Option<TabChangeCallback>,
    on_navigate_path: Option<FileNavigateCallback>,
    on_refresh_logs: Option<RefreshCallback>,
}

impl MachineDetail {
    pub fn new() -> Self {
        Self {
            machine: None,
            active_tab: 0,
            machine_state: None,
            terminal_view: None,
            on_start: None,
            on_stop: None,
            on_restart: None,
            on_delete: None,
            on_tab_change: None,
            on_navigate_path: None,
            on_refresh_logs: None,
        }
    }

    pub fn machine(mut self, machine: Option<ColimaVm>) -> Self {
        self.machine = machine;
        self
    }

    pub fn active_tab(mut self, tab: usize) -> Self {
        self.active_tab = tab;
        self
    }

    pub fn machine_state(mut self, state: MachineTabState) -> Self {
        self.machine_state = Some(state);
        self
    }

    pub fn terminal_view(mut self, view: Option<Entity<TerminalView>>) -> Self {
        self.terminal_view = view;
        self
    }

    pub fn on_start<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_start = Some(Rc::new(callback));
        self
    }

    pub fn on_stop<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_stop = Some(Rc::new(callback));
        self
    }

    pub fn on_restart<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_restart = Some(Rc::new(callback));
        self
    }

    pub fn on_delete<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_delete = Some(Rc::new(callback));
        self
    }

    pub fn on_tab_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&usize, &mut Window, &mut App) + 'static,
    {
        self.on_tab_change = Some(Rc::new(callback));
        self
    }

    pub fn on_navigate_path<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_navigate_path = Some(Rc::new(callback));
        self
    }

    pub fn on_refresh_logs<F>(mut self, callback: F) -> Self
    where
        F: Fn(&(), &mut Window, &mut App) + 'static,
    {
        self.on_refresh_logs = Some(Rc::new(callback));
        self
    }

    fn render_toolbar(&self, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;
        let tabs = vec!["Info", "Logs", "Terminal", "Files"];

        let mut inner_toolbar = h_flex()
            .h(px(48.))
            .w_full()
            .px(px(16.))
            .border_b_1()
            .border_color(colors.border)
            .items_center()
            .justify_center();

        if self.machine.is_some() {
            let on_tab_change = self.on_tab_change.clone();
            let tab_bar = TabBar::new("machine-detail-tabs")
                .small()
                .selected_index(self.active_tab)
                .on_click(move |index, window, cx| {
                    if let Some(cb) = on_tab_change.clone() {
                        cb(index, window, cx);
                    }
                })
                .children(tabs.into_iter().map(|label| Tab::new().label(label)));

            inner_toolbar = inner_toolbar.child(tab_bar);
        }

        v_flex()
            .w_full()
            .pt(px(4.))
            .bg(colors.sidebar)
            .child(inner_toolbar)
    }

    fn render_empty(&self, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        div().flex_1().flex().items_center().justify_center().child(
            div()
                .text_xl()
                .text_color(colors.muted_foreground)
                .child("No Selection"),
        )
    }

    fn render_info_tab(&self, machine: &ColimaVm, cx: &App) -> gpui::Div {
        let status_text = machine.status.to_string();
        let domain = format!("{}.local", machine.name);

        // Basic info rows
        let mut basic_info = vec![
            ("Name", machine.name.clone()),
            ("Status", status_text),
            ("Domain", domain),
        ];

        if let Some(addr) = &machine.address {
            basic_info.push(("IP Address", addr.clone()));
        }

        // Get real OS info from state if available
        let os_info = self.machine_state.as_ref().and_then(|s| s.os_info.as_ref());

        // Image section - use real OS info
        let image_info = if let Some(os) = os_info {
            vec![
                ("Distro", os.pretty_name.clone()),
                ("Kernel", os.kernel.clone()),
                ("Architecture", os.arch.clone()),
            ]
        } else {
            vec![
                ("Distro", "Loading...".to_string()),
                ("Kernel", "-".to_string()),
                ("Architecture", machine.arch.display_name().to_string()),
            ]
        };

        // Settings section
        let mut settings_info = vec![
            ("CPUs", machine.cpus.to_string()),
            ("Memory", format!("{:.0} GB", machine.memory_gb())),
            ("Disk", format!("{:.0} GB", machine.disk_gb())),
            ("Driver", machine.display_driver()),
            ("Mount Type", machine.display_mount_type()),
            ("Runtime", machine.runtime.to_string()),
        ];

        if machine.kubernetes {
            settings_info.push(("Kubernetes", "Enabled".to_string()));
        }

        if let Some(socket) = &machine.docker_socket {
            settings_info.push(("Docker Socket", socket.clone()));
        }

        v_flex()
            .flex_1()
            .w_full()
            .p(px(16.))
            .gap(px(12.))
            .child(self.render_section(None, basic_info, cx))
            .child(self.render_section(Some("Image"), image_info, cx))
            .child(self.render_section(Some("Settings"), settings_info, cx))
    }

    fn render_section(
        &self,
        header: Option<&str>,
        rows: Vec<(&str, String)>,
        cx: &App,
    ) -> gpui::Div {
        let colors = &cx.theme().colors;

        let mut section = v_flex().gap(px(1.));

        if let Some(title) = header {
            section = section.child(
                div()
                    .py(px(8.))
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child(title.to_string()),
            );
        }

        let rows_container = v_flex()
            .bg(colors.background)
            .rounded(px(8.))
            .overflow_hidden()
            .children(
                rows.into_iter()
                    .enumerate()
                    .map(|(i, (label, value))| self.render_section_row(label, value, i == 0, cx)),
            );

        section.child(rows_container)
    }

    fn render_section_row(
        &self,
        label: &str,
        value: String,
        is_first: bool,
        cx: &App,
    ) -> gpui::Div {
        let colors = &cx.theme().colors;

        let mut row = h_flex()
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .text_color(colors.secondary_foreground)
                    .child(label.to_string()),
            )
            .child(div().text_sm().text_color(colors.foreground).child(value));

        if !is_first {
            row = row.border_t_1().border_color(colors.border);
        }

        row
    }

    fn render_logs_tab(&self, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        let logs_content = self
            .machine_state
            .as_ref()
            .map(|s| s.logs.clone())
            .unwrap_or_else(|| "Loading logs...".to_string());

        let is_loading = self
            .machine_state
            .as_ref()
            .map(|s| s.logs_loading)
            .unwrap_or(false);

        let on_refresh = self.on_refresh_logs.clone();

        v_flex()
            .flex_1()
            .w_full()
            .p(px(16.))
            .gap(px(8.))
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(colors.foreground)
                            .child("System Logs"),
                    )
                    .child(
                        Button::new("refresh-logs")
                            .icon(IconName::Redo)
                            .ghost()
                            .compact()
                            .when_some(on_refresh, |btn, cb| {
                                btn.on_click(move |_ev, window, cx| {
                                    cb(&(), window, cx);
                                })
                            }),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .bg(gpui::rgb(0x1a1b26))
                    .rounded(px(8.))
                    .p(px(12.))
                    .overflow_y_scrollbar()
                    .font_family("monospace")
                    .text_xs()
                    .text_color(gpui::rgb(0xa9b1d6))
                    .when(is_loading, |el| el.child("Loading logs..."))
                    .when(!is_loading, |el| el.child(logs_content)),
            )
    }

    fn render_terminal_tab(&self, cx: &App) -> gpui::Div {
        // If we have a terminal view, render it full size
        if let Some(terminal) = &self.terminal_view {
            return div()
                .size_full()
                .flex_1()
                .min_h_0()
                .p(px(8.))
                .child(terminal.clone());
        }

        let colors = &cx.theme().colors;

        // Fallback: show message
        let terminal_output = self
            .machine_state
            .as_ref()
            .map(|s| s.terminal_output.clone())
            .unwrap_or_default();

        let history = self
            .machine_state
            .as_ref()
            .map(|s| s.terminal_history.clone())
            .unwrap_or_default();

        v_flex()
            .flex_1()
            .w_full()
            .p(px(16.))
            .gap(px(8.))
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child("Terminal"),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .bg(gpui::rgb(0x1a1b26))
                    .rounded(px(8.))
                    .p(px(12.))
                    .overflow_hidden()
                    .font_family("monospace")
                    .text_sm()
                    .text_color(gpui::rgb(0xa9b1d6))
                    .child(
                        v_flex()
                            .gap(px(4.))
                            .children(history.iter().map(|cmd| {
                                div()
                                    .text_color(gpui::rgb(0x7aa2f7))
                                    .child(format!("$ {}", cmd))
                            }))
                            .when(!terminal_output.is_empty(), |el| {
                                el.child(
                                    div()
                                        .text_color(gpui::rgb(0xa9b1d6))
                                        .child(terminal_output.clone()),
                                )
                            })
                            .child(
                                h_flex()
                                    .items_center()
                                    .child(div().text_color(gpui::rgb(0x7aa2f7)).child("$ "))
                                    .child(
                                        div()
                                            .w(px(8.))
                                            .h(px(16.))
                                            .bg(gpui::rgb(0xa9b1d6))
                                            .child(""),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.muted_foreground)
                    .child("Click on the Terminal tab to connect"),
            )
    }

    fn render_files_tab(&self, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        let current_path = self
            .machine_state
            .as_ref()
            .map(|s| s.current_path.clone())
            .unwrap_or_else(|| "/".to_string());

        let files = self
            .machine_state
            .as_ref()
            .map(|s| s.files.clone())
            .unwrap_or_default();

        let is_loading = self
            .machine_state
            .as_ref()
            .map(|s| s.files_loading)
            .unwrap_or(false);

        let on_navigate = self.on_navigate_path.clone();
        let on_navigate_up = self.on_navigate_path.clone();

        // Calculate parent path
        let parent_path = if current_path == "/" {
            "/".to_string()
        } else {
            let parts: Vec<&str> = current_path.trim_end_matches('/').split('/').collect();
            if parts.len() <= 2 {
                "/".to_string()
            } else {
                parts[..parts.len() - 1].join("/")
            }
        };

        let mut file_list = v_flex().gap(px(2.));

        for file in files.iter() {
            let file_path = file.path.clone();
            let is_dir = file.is_dir;
            let cb = on_navigate.clone();

            file_list = file_list.child(self.render_file_entry(file, cx).when(is_dir, |el| {
                el.cursor_pointer().when_some(cb, move |el, cb| {
                    let path = file_path.clone();
                    el.on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                        cb(&path, window, cx);
                    })
                })
            }));
        }

        v_flex()
            .flex_1()
            .w_full()
            .p(px(16.))
            .gap(px(8.))
            .child(
                h_flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        Button::new("up")
                            .icon(IconName::ArrowUp)
                            .ghost()
                            .compact()
                            .when_some(on_navigate_up, move |btn, cb| {
                                let path = parent_path.clone();
                                btn.on_click(move |_ev, window, cx| {
                                    cb(&path, window, cx);
                                })
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .px(px(12.))
                            .py(px(8.))
                            .bg(colors.background)
                            .rounded(px(6.))
                            .text_sm()
                            .text_color(colors.secondary_foreground)
                            .child(current_path),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .bg(colors.background)
                    .rounded(px(8.))
                    .p(px(8.))
                    .overflow_y_scrollbar()
                    .when(is_loading, |el| {
                        el.child(
                            div()
                                .p(px(16.))
                                .text_sm()
                                .text_color(colors.muted_foreground)
                                .child("Loading..."),
                        )
                    })
                    .when(!is_loading && files.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(16.))
                                .text_sm()
                                .text_color(colors.muted_foreground)
                                .child("Directory is empty"),
                        )
                    })
                    .when(!is_loading && !files.is_empty(), |el| el.child(file_list)),
            )
    }

    fn render_file_entry(&self, file: &VmFileEntry, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;
        let icon = if file.is_dir {
            IconName::Folder
        } else if file.is_symlink {
            IconName::ExternalLink
        } else {
            IconName::File
        };

        let icon_color = if file.is_dir {
            colors.warning
        } else if file.is_symlink {
            colors.info
        } else {
            colors.secondary_foreground
        };

        h_flex()
            .w_full()
            .px(px(12.))
            .py(px(8.))
            .rounded(px(4.))
            .items_center()
            .gap(px(10.))
            .hover(|s| s.bg(colors.sidebar))
            .child(Icon::new(icon).text_color(icon_color))
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(colors.foreground)
                    .child(file.name.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.muted_foreground)
                    .child(file.display_size()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.muted_foreground)
                    .w(px(80.))
                    .child(file.permissions.clone()),
            )
    }
}

impl MachineDetail {
    pub fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;

        let toolbar = self.render_toolbar(cx);

        let content = match &self.machine {
            None => self.render_empty(cx),
            Some(machine) => match self.active_tab {
                0 => self.render_info_tab(machine, cx),
                1 => self.render_logs_tab(cx),
                2 => self.render_terminal_tab(cx),
                3 => self.render_files_tab(cx),
                _ => self.render_info_tab(machine, cx),
            },
        };

        // Terminal tab needs full height without scroll, other tabs need scroll
        let is_terminal_tab = self.machine.is_some() && self.active_tab == 2;

        let mut container = div()
            .size_full()
            .bg(colors.sidebar)
            .flex()
            .flex_col()
            .child(toolbar);

        if is_terminal_tab {
            // Terminal tab: flex_1 for full height, terminal handles its own scroll
            container = container.child(
                div()
                    .flex_1()
                    .min_h_0() // Allow shrinking for scroll to work
                    .w_full()
                    .child(content),
            );
        } else {
            // Other tabs: scrollable with bottom padding
            container = container.child(
                div()
                    .id("machine-detail-scroll")
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(content)
                    .child(div().h(px(100.))),
            );
        }

        container
    }
}
