use gpui::{Context, Entity, Render, Styled, Window, div, prelude::*, px};
use gpui_component::{
  WindowExt,
  button::{Button, ButtonVariants},
  input::InputState,
  theme::ActiveTheme,
};

use crate::colima::ColimaVm;
use crate::services;
use crate::state::{DockerState, MachineTabState, StateChanged, docker_state};
use crate::terminal::TerminalView;

use super::create_dialog::CreateMachineDialog;
use super::detail::MachineDetail;
use super::edit_dialog::EditMachineDialog;
use super::list::{MachineList, MachineListEvent};

/// Self-contained Machines view - handles list, detail, terminal, and all state
pub struct MachinesView {
  _docker_state: Entity<DockerState>,
  machine_list: Entity<MachineList>,
  selected_machine: Option<ColimaVm>,
  active_tab: usize,
  terminal_view: Option<Entity<TerminalView>>,
  machine_tab_state: MachineTabState,
  logs_editor: Option<Entity<InputState>>,
  last_synced_logs: String,
}

impl MachinesView {
  pub fn new(window: &mut Window, cx: &mut Context<'_, Self>) -> Self {
    let docker_state = docker_state(cx);

    // Create machine list entity
    let machine_list = cx.new(|cx| MachineList::new(window, cx));

    // Subscribe to machine list events
    cx.subscribe_in(
      &machine_list,
      window,
      |this, _list, event: &MachineListEvent, window, cx| match event {
        MachineListEvent::Selected(machine) => {
          this.on_select_machine(machine, window, cx);
        }
        MachineListEvent::NewMachine => {
          this.show_create_dialog(window, cx);
        }
      },
    )
    .detach();

    // Subscribe to state changes
    cx.subscribe_in(
      &docker_state,
      window,
      |this, state, event: &StateChanged, window, cx| {
        match event {
          StateChanged::MachinesUpdated => {
            // If selected machine was deleted, clear selection
            if let Some(ref selected) = this.selected_machine {
              let state = state.read(cx);
              if !state.colima_vms.iter().any(|vm| vm.name == selected.name) {
                this.selected_machine = None;
                this.active_tab = 0;
                this.terminal_view = None;
              }
            }
            cx.notify();
          }
          StateChanged::MachineTabRequest { machine_name, tab } => {
            // Find the machine and select it with the specified tab
            let machine = {
              let state = state.read(cx);
              state.colima_vms.iter().find(|vm| vm.name == *machine_name).cloned()
            };
            if let Some(machine) = machine {
              this.on_select_machine(&machine, window, cx);
              this.on_tab_change(*tab, window, cx);
            }
          }
          _ => {}
        }
      },
    )
    .detach();

    Self {
      _docker_state: docker_state,
      machine_list,
      selected_machine: None,
      active_tab: 0,
      terminal_view: None,
      machine_tab_state: MachineTabState::default(),
      logs_editor: None,
      last_synced_logs: String::new(),
    }
  }

  fn show_create_dialog(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
    let dialog_entity = cx.new(CreateMachineDialog::new);

    window.open_dialog(cx, move |dialog, _window, _cx| {
      let dialog_clone = dialog_entity.clone();

      dialog
        .title("New Machine")
        .min_w(px(500.))
        .child(dialog_entity.clone())
        .footer(move |_dialog_state, _, _window, _cx| {
          let dialog_for_create = dialog_clone.clone();

          vec![
            Button::new("create")
              .label("Create")
              .primary()
              .on_click({
                let dialog = dialog_for_create.clone();
                move |_ev, window, cx| {
                  let options = dialog.read(cx).get_options(cx);
                  services::create_machine(options, cx);
                  window.close_dialog(cx);
                }
              })
              .into_any_element(),
          ]
        })
    });
  }

  fn show_edit_dialog(&mut self, machine: &ColimaVm, window: &mut Window, cx: &mut Context<'_, Self>) {
    let machine_clone = machine.clone();
    let dialog_entity = cx.new(|cx| EditMachineDialog::new(&machine_clone, cx));

    window.open_dialog(cx, move |dialog, _window, _cx| {
      let dialog_clone = dialog_entity.clone();
      let machine = machine_clone.clone();

      dialog
        .title(format!("Edit Machine: {}", machine.name))
        .min_w(px(500.))
        .child(dialog_entity.clone())
        .footer(move |_dialog_state, _, _window, _cx| {
          let dialog_for_save = dialog_clone.clone();

          vec![
            Button::new("save")
              .label("Save & Restart")
              .primary()
              .on_click({
                let dialog = dialog_for_save.clone();
                move |_ev, window, cx| {
                  let options = dialog.read(cx).get_options(cx);
                  services::edit_machine(options, cx);
                  window.close_dialog(cx);
                }
              })
              .into_any_element(),
          ]
        })
    });
  }

  fn on_select_machine(&mut self, machine: &ColimaVm, window: &mut Window, cx: &mut Context<'_, Self>) {
    self.selected_machine = Some(machine.clone());
    self.active_tab = 0;
    self.terminal_view = None;

    // Clear synced tracking for new machine
    self.last_synced_logs.clear();

    // Create logs editor
    self.logs_editor = Some(cx.new(|cx| {
      InputState::new(window, cx)
        .multi_line(true)
        .code_editor("log")
        .line_number(true)
        .searchable(true)
        .soft_wrap(false)
    }));

    // Load OS info, logs, files for the selected machine
    self.load_machine_data(&machine.name, cx);

    cx.notify();
  }

  fn on_tab_change(&mut self, tab: usize, window: &mut Window, cx: &mut Context<'_, Self>) {
    self.active_tab = tab;

    // If switching to terminal tab, create terminal view
    if tab == 2
      && self.terminal_view.is_none()
      && let Some(ref machine) = self.selected_machine
    {
      let profile = if machine.name == "default" {
        None
      } else {
        Some(machine.name.clone())
      };
      self.terminal_view = Some(cx.new(|cx| TerminalView::for_colima(profile, window, cx)));
    }

    cx.notify();
  }

  fn load_machine_data(&mut self, name: &str, cx: &mut Context<'_, Self>) {
    // Load OS info
    self.machine_tab_state.logs_loading = true;
    self.machine_tab_state.files_loading = true;

    let machine_name = name.to_string();
    let machine_name2 = machine_name.clone();
    let machine_name3 = machine_name.clone();

    // Load OS info in background
    cx.spawn(async move |this, cx| {
      let os_info = cx
        .background_executor()
        .spawn(async move {
          let colima = crate::colima::ColimaClient::new();
          let name_opt = if machine_name == "default" {
            None
          } else {
            Some(machine_name.as_str())
          };
          colima.get_os_info(name_opt).ok()
        })
        .await;

      let _ = this.update(cx, |this, cx| {
        if let Some(info) = os_info {
          this.machine_tab_state.os_info = Some(info);
          cx.notify();
        }
      });
    })
    .detach();

    // Load logs in background
    cx.spawn(async move |this, cx| {
      let logs = cx
        .background_executor()
        .spawn(async move {
          let colima = crate::colima::ColimaClient::new();
          let name_opt = if machine_name2 == "default" {
            None
          } else {
            Some(machine_name2.as_str())
          };
          colima
            .get_system_logs(name_opt, 100)
            .unwrap_or_else(|_| "Failed to load logs".to_string())
        })
        .await;

      let _ = this.update(cx, |this, cx| {
        this.machine_tab_state.logs = logs;
        this.machine_tab_state.logs_loading = false;
        cx.notify();
      });
    })
    .detach();

    // Load files in background
    cx.spawn(async move |this, cx| {
      let files = cx
        .background_executor()
        .spawn(async move {
          let colima = crate::colima::ColimaClient::new();
          let name_opt = if machine_name3 == "default" {
            None
          } else {
            Some(machine_name3.as_str())
          };
          colima.list_files(name_opt, "/").unwrap_or_default()
        })
        .await;

      let _ = this.update(cx, |this, cx| {
        this.machine_tab_state.files = files;
        this.machine_tab_state.files_loading = false;
        this.machine_tab_state.current_path = "/".to_string();
        cx.notify();
      });
    })
    .detach();
  }

  fn on_navigate_path(&mut self, path: &str, cx: &mut Context<'_, Self>) {
    if let Some(ref machine) = self.selected_machine.clone() {
      self.machine_tab_state.files_loading = true;
      let machine_name = machine.name.clone();
      let path = path.to_string();

      cx.spawn(async move |this, cx| {
        let (files, current_path) = cx
          .background_executor()
          .spawn(async move {
            let colima = crate::colima::ColimaClient::new();
            let name_opt = if machine_name == "default" {
              None
            } else {
              Some(machine_name.as_str())
            };
            let files = colima.list_files(name_opt, &path).unwrap_or_default();
            (files, path)
          })
          .await;

        let _ = this.update(cx, |this, cx| {
          this.machine_tab_state.files = files;
          this.machine_tab_state.files_loading = false;
          this.machine_tab_state.current_path = current_path;
          cx.notify();
        });
      })
      .detach();

      cx.notify();
    }
  }

  fn on_refresh_logs(&mut self, cx: &mut Context<'_, Self>) {
    if let Some(ref machine) = self.selected_machine.clone() {
      self.machine_tab_state.logs_loading = true;
      let machine_name = machine.name.clone();

      cx.spawn(async move |this, cx| {
        let logs = cx
          .background_executor()
          .spawn(async move {
            let colima = crate::colima::ColimaClient::new();
            let name_opt = if machine_name == "default" {
              None
            } else {
              Some(machine_name.as_str())
            };
            colima
              .get_system_logs(name_opt, 100)
              .unwrap_or_else(|_| "Failed to load logs".to_string())
          })
          .await;

        let _ = this.update(cx, |this, cx| {
          this.machine_tab_state.logs = logs;
          this.machine_tab_state.logs_loading = false;
          cx.notify();
        });
      })
      .detach();

      cx.notify();
    }
  }
}

impl Render for MachinesView {
  fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
    // Sync logs editor content
    if let Some(ref editor) = self.logs_editor {
      let logs = &self.machine_tab_state.logs;
      if !logs.is_empty() && !self.machine_tab_state.logs_loading && self.last_synced_logs != *logs {
        let logs_clone = logs.clone();
        editor.update(cx, |state, cx| {
          state.replace(&logs_clone, window, cx);
        });
        self.last_synced_logs = logs.clone();
      }
    }

    let colors = cx.theme().colors;
    let selected_machine = self.selected_machine.clone();
    let active_tab = self.active_tab;
    let machine_tab_state = self.machine_tab_state.clone();
    let terminal_view = self.terminal_view.clone();
    let logs_editor = self.logs_editor.clone();

    // Build detail panel
    let detail = MachineDetail::new()
      .machine(selected_machine)
      .active_tab(active_tab)
      .machine_state(machine_tab_state)
      .terminal_view(terminal_view)
      .logs_editor(logs_editor)
      .on_tab_change(cx.listener(|this, tab: &usize, window, cx| {
        this.on_tab_change(*tab, window, cx);
      }))
      .on_navigate_path(cx.listener(|this, path: &str, _window, cx| {
        this.on_navigate_path(path, cx);
      }))
      .on_refresh_logs(cx.listener(|this, _: &(), _window, cx| {
        this.on_refresh_logs(cx);
      }))
      .on_start(cx.listener(|_this, name: &str, _window, cx| {
        services::start_machine(name.to_string(), cx);
      }))
      .on_stop(cx.listener(|_this, name: &str, _window, cx| {
        services::stop_machine(name.to_string(), cx);
      }))
      .on_restart(cx.listener(|_this, name: &str, _window, cx| {
        services::restart_machine(name.to_string(), cx);
      }))
      .on_delete(cx.listener(|this, name: &str, _window, cx| {
        services::delete_machine(name.to_string(), cx);
        this.selected_machine = None;
        this.active_tab = 0;
        this.terminal_view = None;
        cx.notify();
      }))
      .on_edit(cx.listener(|this, machine: &ColimaVm, window, cx| {
        this.show_edit_dialog(machine, window, cx);
      }));

    div()
      .size_full()
      .flex()
      .overflow_hidden()
      .child(
        // Left: Machine list - fixed width with border
        div()
          .w(px(320.))
          .h_full()
          .flex_shrink_0()
          .overflow_hidden()
          .border_r_1()
          .border_color(colors.border)
          .child(self.machine_list.clone()),
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
