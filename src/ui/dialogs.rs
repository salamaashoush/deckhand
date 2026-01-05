//! Centralized dialog helpers
//!
//! This module provides simple helper functions to open dialogs with all buttons
//! and actions pre-configured. Call these functions from anywhere (views, command
//! palette, menu bar) to open a fully functional dialog.

use gpui::{App, AppContext, IntoElement, ParentElement, Styled, Window, px};
use gpui_component::{
  WindowExt,
  button::{Button, ButtonVariants},
  theme::ActiveTheme,
  v_flex,
};

use crate::services;
use crate::ui::deployments::create_dialog::CreateDeploymentDialog;
use crate::ui::images::pull_dialog::PullImageDialog;
use crate::ui::machines::MachineDialog;
use crate::ui::networks::create_dialog::CreateNetworkDialog;
use crate::ui::prune_dialog::PruneDialog;
use crate::ui::services::create_dialog::CreateServiceDialog;
use crate::ui::volumes::create_dialog::CreateVolumeDialog;

/// Opens the Pull Image dialog with Pull button configured
pub fn open_pull_image_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(PullImageDialog::new);

  window.open_dialog(cx, move |dialog, _window, _cx| {
    let dialog_clone = dialog_entity.clone();

    dialog
      .title("Pull Image")
      .min_w(px(500.))
      .child(dialog_entity.clone())
      .footer(move |_dialog_state, _, _window, _cx| {
        let dialog_for_pull = dialog_clone.clone();
        vec![
          Button::new("pull")
            .label("Pull")
            .primary()
            .on_click({
              let dialog = dialog_for_pull.clone();
              move |_ev, window, cx| {
                let options = dialog.read(cx).get_options(cx);
                if !options.image.is_empty() {
                  services::pull_image(options.image, options.platform.as_docker_arg().map(String::from), cx);
                  window.close_dialog(cx);
                }
              }
            })
            .into_any_element(),
        ]
      })
  });
}

/// Opens the Create Volume dialog with Create button configured
pub fn open_create_volume_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(CreateVolumeDialog::new);

  window.open_dialog(cx, move |dialog, _window, _cx| {
    let dialog_clone = dialog_entity.clone();

    dialog
      .title("Create Volume")
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
                if !options.name.is_empty() {
                  services::create_volume(
                    options.name,
                    options.driver.as_docker_arg().to_string(),
                    options.labels,
                    cx,
                  );
                  window.close_dialog(cx);
                }
              }
            })
            .into_any_element(),
        ]
      })
  });
}

/// Opens the Create Network dialog with Create button configured
pub fn open_create_network_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(CreateNetworkDialog::new);

  window.open_dialog(cx, move |dialog, _window, cx| {
    let colors = cx.theme().colors;

    dialog
      .title("Create Network")
      .min_w(px(500.))
      .child(
        v_flex()
          .gap(px(8.))
          .child(gpui::div().text_sm().text_color(colors.muted_foreground).child(
            "Networks are groups of containers in the same subnet (IP range) that can communicate with each other.",
          ))
          .child(dialog_entity.clone()),
      )
      .footer({
        let dialog_clone = dialog_entity.clone();
        move |_dialog_state, _, _window, _cx| {
          let dialog_for_create = dialog_clone.clone();
          vec![
            Button::new("create")
              .label("Create")
              .primary()
              .on_click({
                let dialog = dialog_for_create.clone();
                move |_ev, window, cx| {
                  let options = dialog.read(cx).get_options(cx);
                  if !options.name.is_empty() {
                    services::create_network(options.name, options.enable_ipv6, options.subnet, cx);
                    window.close_dialog(cx);
                  }
                }
              })
              .into_any_element(),
          ]
        }
      })
  });
}

/// Opens the Create Machine (Colima) dialog with Create button configured
pub fn open_create_machine_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(MachineDialog::new_create);

  window.open_dialog(cx, move |dialog, _window, _cx| {
    let dialog_clone = dialog_entity.clone();

    dialog
      .title("Create Colima Machine")
      .min_w(px(550.))
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
                let profile = dialog.read(cx).get_profile_name(cx);
                let config = dialog.read(cx).get_config(cx);
                services::create_machine(profile, config, cx);
                window.close_dialog(cx);
              }
            })
            .into_any_element(),
        ]
      })
  });
}

/// Opens the Create Deployment (Kubernetes) dialog with Create button configured
pub fn open_create_deployment_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(CreateDeploymentDialog::new);

  window.open_dialog(cx, move |dialog, _window, _cx| {
    let dialog_clone = dialog_entity.clone();

    dialog
      .title("Create Deployment")
      .min_w(px(550.))
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
                if !options.name.is_empty() && !options.image.is_empty() {
                  services::create_deployment(options, cx);
                  window.close_dialog(cx);
                }
              }
            })
            .into_any_element(),
        ]
      })
  });
}

/// Opens the Create Service (Kubernetes) dialog with Create button configured
pub fn open_create_service_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(CreateServiceDialog::new);

  window.open_dialog(cx, move |dialog, _window, _cx| {
    let dialog_clone = dialog_entity.clone();

    dialog
      .title("Create Service")
      .min_w(px(550.))
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
                if !options.name.is_empty() && !options.ports.is_empty() {
                  services::create_service(options, cx);
                  window.close_dialog(cx);
                }
              }
            })
            .into_any_element(),
        ]
      })
  });
}

/// Opens the Prune Docker Resources dialog with Prune button configured
pub fn open_prune_dialog(window: &mut Window, cx: &mut App) {
  let dialog_entity = cx.new(PruneDialog::new);

  window.open_dialog(cx, move |dialog, _window, _cx| {
    let dialog_clone = dialog_entity.clone();

    dialog
      .title("Prune Docker Resources")
      .min_w(px(500.))
      .child(dialog_entity.clone())
      .footer(move |_dialog_state, _, _window, _cx| {
        let dialog_for_prune = dialog_clone.clone();
        vec![
          Button::new("prune")
            .label("Prune")
            .primary()
            .on_click({
              let dialog = dialog_for_prune.clone();
              move |_ev, window, cx| {
                let options = dialog.read(cx).get_options();
                if !options.is_empty() {
                  services::prune_docker(&options, cx);
                  window.close_dialog(cx);
                }
              }
            })
            .into_any_element(),
        ]
      })
  });
}
