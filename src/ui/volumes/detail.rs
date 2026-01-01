use gpui::{div, prelude::*, px, App, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    tab::{Tab, TabBar},
    theme::ActiveTheme,
    v_flex, Icon, IconName, Selectable, Sizable,
};
use std::rc::Rc;

use crate::assets::AppIcon;
use crate::docker::VolumeInfo;

type VolumeActionCallback = Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
type TabChangeCallback = Rc<dyn Fn(&usize, &mut Window, &mut App) + 'static>;

pub struct VolumeDetail {
    volume: Option<VolumeInfo>,
    active_tab: usize,
    on_delete: Option<VolumeActionCallback>,
    on_tab_change: Option<TabChangeCallback>,
}

impl VolumeDetail {
    pub fn new() -> Self {
        Self {
            volume: None,
            active_tab: 0,
            on_delete: None,
            on_tab_change: None,
        }
    }

    pub fn volume(mut self, volume: Option<VolumeInfo>) -> Self {
        self.volume = volume;
        self
    }

    pub fn active_tab(mut self, tab: usize) -> Self {
        self.active_tab = tab;
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

    fn render_empty(&self, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        div()
            .size_full()
            .bg(colors.sidebar)
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .items_center()
                    .gap(px(16.))
                    .child(
                        Icon::new(AppIcon::Volume)
                            .size(px(48.))
                            .text_color(colors.muted_foreground),
                    )
                    .child(
                        div()
                            .text_color(colors.muted_foreground)
                            .child("Select a volume to view details"),
                    ),
            )
    }

    fn render_info_tab(&self, volume: &VolumeInfo, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        // Basic info rows
        let mut basic_info = vec![
            ("Name", volume.name.clone()),
            ("Size", volume.display_size()),
        ];

        if let Some(created) = volume.created {
            basic_info.insert(
                1,
                ("Created", created.format("%Y-%m-%d %H:%M:%S").to_string()),
            );
        }

        v_flex()
            .flex_1()
            .w_full()
            .p(px(16.))
            .gap(px(12.))
            .child(self.render_section(None, basic_info, cx))
            // Action buttons
            .child(
                v_flex()
                    .gap(px(1.))
                    .child(self.render_action_row("Export", IconName::ArrowUp, cx))
                    .child(self.render_action_row("Clone", IconName::Copy, cx)),
            )
            // Labels section if not empty
            .when(!volume.labels.is_empty(), |el| {
                el.child(self.render_labels_section(volume, cx))
            })
            // Additional info
            .child(self.render_section(
                Some("Details"),
                vec![
                    ("Driver", volume.driver.clone()),
                    ("Mountpoint", volume.mountpoint.clone()),
                    ("Scope", volume.scope.clone()),
                ],
                cx,
            ))
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
            .child(
                div()
                    .text_sm()
                    .text_color(colors.foreground)
                    .max_w(px(200.))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(value),
            );

        if !is_first {
            row = row.border_t_1().border_color(colors.border);
        }

        row
    }

    fn render_action_row(&self, label: &str, icon: IconName, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        h_flex()
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .items_center()
            .justify_between()
            .bg(colors.background)
            .rounded(px(8.))
            .cursor_pointer()
            .hover(|s| s.bg(colors.sidebar))
            .child(
                h_flex()
                    .gap(px(12.))
                    .items_center()
                    .child(Icon::new(icon).text_color(colors.secondary_foreground))
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.foreground)
                            .child(label.to_string()),
                    ),
            )
            .child(Icon::new(IconName::ChevronRight).text_color(colors.muted_foreground))
    }

    fn render_labels_section(&self, volume: &VolumeInfo, cx: &App) -> gpui::Div {
        let colors = &cx.theme().colors;

        let mut labels: Vec<_> = volume.labels.iter().collect();
        labels.sort_by(|a, b| a.0.cmp(b.0));

        v_flex()
            .gap(px(1.))
            .child(
                div()
                    .py(px(8.))
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child("Labels"),
            )
            .child(
                v_flex()
                    .bg(colors.background)
                    .rounded(px(8.))
                    .overflow_hidden()
                    // Header row
                    .child(
                        h_flex()
                            .w_full()
                            .px(px(16.))
                            .py(px(8.))
                            .bg(colors.sidebar)
                            .child(
                                div()
                                    .flex_1()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(colors.muted_foreground)
                                    .child("Key"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(colors.muted_foreground)
                                    .child("Value"),
                            ),
                    )
                    // Label rows
                    .children(labels.iter().enumerate().map(|(i, (key, value))| {
                        let mut row = h_flex()
                            .w_full()
                            .px(px(16.))
                            .py(px(10.))
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(colors.foreground)
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(key.to_string()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(colors.secondary_foreground)
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(value.to_string()),
                            );

                        if i > 0 {
                            row = row.border_t_1().border_color(colors.border);
                        }
                        row
                    })),
            )
    }

    pub fn render(self, _window: &mut Window, cx: &App) -> gpui::AnyElement {
        let colors = &cx.theme().colors;

        let Some(volume) = &self.volume else {
            return self.render_empty(cx).into_any_element();
        };

        let volume_name = volume.name.clone();
        let volume_name_for_delete = volume_name.clone();

        let on_delete = self.on_delete.clone();
        let on_tab_change = self.on_tab_change.clone();

        let tabs = vec!["Info"];

        // Toolbar with tabs and actions
        let toolbar = h_flex()
            .w_full()
            .px(px(16.))
            .py(px(8.))
            .gap(px(12.))
            .items_center()
            .border_b_1()
            .border_color(colors.border)
            .child(
                TabBar::new("volume-tabs")
                    .flex_1()
                    .children(tabs.iter().enumerate().map(|(i, label)| {
                        let on_tab_change = on_tab_change.clone();
                        Tab::new()
                            .label(label.to_string())
                            .selected(self.active_tab == i)
                            .on_click(move |_ev, window, cx| {
                                if let Some(ref cb) = on_tab_change {
                                    cb(&i, window, cx);
                                }
                            })
                    })),
            )
            .child(
                h_flex().gap(px(8.)).child({
                    let on_delete = on_delete.clone();
                    let name = volume_name_for_delete.clone();
                    Button::new("delete")
                        .icon(Icon::new(AppIcon::Trash))
                        .ghost()
                        .small()
                        .on_click(move |_ev, window, cx| {
                            if let Some(ref cb) = on_delete {
                                cb(&name, window, cx);
                            }
                        })
                }),
            );

        // Content based on active tab
        let content = match self.active_tab {
            0 => self.render_info_tab(volume, cx),
            _ => self.render_info_tab(volume, cx),
        };

        div()
            .size_full()
            .bg(colors.sidebar)
            .flex()
            .flex_col()
            .child(toolbar)
            .child(
                div()
                    .id("volume-detail-scroll")
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(content)
                    .child(div().h(px(100.))),
            )
            .into_any_element()
    }
}
