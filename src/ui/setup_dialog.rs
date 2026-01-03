use gpui::{App, Context, FocusHandle, Focusable, Render, Styled, Window, div, prelude::*, px};
use gpui_component::{
  Icon, IconName, Sizable,
  button::{Button, ButtonVariants},
  h_flex,
  label::Label,
  scroll::ScrollableElement,
  theme::ActiveTheme,
  v_flex,
};
use std::process::Command;

/// Common paths for Homebrew binaries on macOS
const BREW_PATHS: &[&str] = &[
  "/opt/homebrew/bin", // Apple Silicon
  "/usr/local/bin",    // Intel
];

/// Find a command in PATH or common locations
fn find_command(name: &str) -> Option<std::path::PathBuf> {
  // First check if it's in PATH
  if let Ok(path) = which::which(name) {
    return Some(path);
  }

  // Check common Homebrew locations
  for base in BREW_PATHS {
    let path = std::path::Path::new(base).join(name);
    if path.exists() {
      return Some(path);
    }
  }

  None
}

/// K8s diagnostic result
#[derive(Debug, Clone, Default)]
pub struct K8sDiagnostic {
  pub kubectl_installed: bool,
  pub current_context: Option<String>,
  pub expected_context: Option<String>,
  pub context_mismatch: bool,
  pub api_reachable: bool,
  pub error_message: Option<String>,
}

/// Run K8s diagnostics
pub fn diagnose_k8s() -> K8sDiagnostic {
  let mut diag = K8sDiagnostic::default();

  // Check if kubectl is installed
  let kubectl_path = find_command("kubectl");
  diag.kubectl_installed = kubectl_path.is_some();

  if !diag.kubectl_installed {
    diag.error_message = Some("kubectl not installed".to_string());
    return diag;
  }

  let kubectl = kubectl_path.unwrap();

  // Get current context
  if let Ok(output) = Command::new(&kubectl).args(["config", "current-context"]).output()
    && output.status.success()
  {
    diag.current_context = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
  }

  // Check if colima is running and has K8s
  if let Some(colima_path) = find_command("colima")
    && let Ok(output) = Command::new(&colima_path).args(["status", "--json"]).output()
    && output.status.success()
  {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) {
      let has_k8s = value["kubernetes"].as_bool().unwrap_or(false);
      if has_k8s {
        // Expected context is "colima" for default profile
        diag.expected_context = Some("colima".to_string());
      }
    }
  }

  // Check for context mismatch
  if let (Some(current), Some(expected)) = (&diag.current_context, &diag.expected_context) {
    diag.context_mismatch = current != expected && !current.starts_with("colima");
  }

  // Try to reach K8s API
  if let Ok(output) = Command::new(&kubectl)
    .args(["cluster-info", "--request-timeout=3s"])
    .output()
  {
    diag.api_reachable = output.status.success();
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      if stderr.contains("refused") {
        diag.error_message = Some("K8s API connection refused".to_string());
      } else if stderr.contains("unreachable") {
        diag.error_message = Some("K8s API network unreachable".to_string());
      } else {
        diag.error_message = Some("K8s API not responding".to_string());
      }
    }
  }

  diag
}

/// Switch kubectl context
pub fn switch_kubectl_context(context: &str) -> Result<(), String> {
  let kubectl = find_command("kubectl").ok_or("kubectl not found")?;
  let output = Command::new(&kubectl)
    .args(["config", "use-context", context])
    .output()
    .map_err(|e| e.to_string())?;

  if output.status.success() {
    Ok(())
  } else {
    Err(String::from_utf8_lossy(&output.stderr).to_string())
  }
}

/// Reset K8s on colima
pub fn reset_colima_k8s() -> Result<(), String> {
  let colima = find_command("colima").ok_or("colima not found")?;
  let output = Command::new(&colima)
    .args(["kubernetes", "reset"])
    .output()
    .map_err(|e| e.to_string())?;

  if output.status.success() {
    Ok(())
  } else {
    Err(String::from_utf8_lossy(&output.stderr).to_string())
  }
}

/// Check if Colima CLI is installed
pub fn is_colima_installed() -> bool {
  if let Some(path) = find_command("colima") {
    Command::new(path)
      .arg("version")
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  } else {
    false
  }
}

/// Check if Docker CLI is installed
pub fn is_docker_installed() -> bool {
  if let Some(path) = find_command("docker") {
    Command::new(path)
      .arg("--version")
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  } else {
    false
  }
}

/// Check if Homebrew is installed
pub fn is_homebrew_installed() -> bool {
  // Check common Homebrew locations directly
  for base in BREW_PATHS {
    let brew_path = std::path::Path::new(base).join("brew");
    if brew_path.exists() {
      return Command::new(brew_path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    }
  }
  false
}

/// Setup dialog shown when Docker/Colima are not detected
pub struct SetupDialog {
  focus_handle: FocusHandle,
  colima_installed: bool,
  docker_installed: bool,
  homebrew_installed: bool,
  k8s_diagnostic: K8sDiagnostic,
  action_message: Option<String>,
}

impl SetupDialog {
  pub fn new(cx: &mut Context<'_, Self>) -> Self {
    let focus_handle = cx.focus_handle();

    Self {
      focus_handle,
      colima_installed: is_colima_installed(),
      docker_installed: is_docker_installed(),
      homebrew_installed: is_homebrew_installed(),
      k8s_diagnostic: diagnose_k8s(),
      action_message: None,
    }
  }

  pub fn refresh_status(&mut self, _cx: &mut Context<'_, Self>) {
    self.colima_installed = is_colima_installed();
    self.docker_installed = is_docker_installed();
    self.homebrew_installed = is_homebrew_installed();
    self.k8s_diagnostic = diagnose_k8s();
    self.action_message = None;
  }

  fn render_status_item(name: &'static str, installed: bool, cx: &Context<'_, Self>) -> impl IntoElement {
    let colors = &cx.theme().colors;

    h_flex()
      .w_full()
      .py(px(8.))
      .px(px(12.))
      .gap(px(12.))
      .items_center()
      .rounded(px(6.))
      .bg(if installed {
        colors.success.opacity(0.1)
      } else {
        colors.danger.opacity(0.1)
      })
      .child(
        Icon::new(if installed { IconName::Check } else { IconName::Close }).text_color(if installed {
          colors.success
        } else {
          colors.danger
        }),
      )
      .child(
        Label::new(name)
          .text_color(colors.foreground)
          .font_weight(gpui::FontWeight::MEDIUM),
      )
      .child(
        div()
          .flex_1()
          .text_right()
          .text_sm()
          .text_color(if installed {
            colors.success
          } else {
            colors.muted_foreground
          })
          .child(if installed { "Installed" } else { "Not Found" }),
      )
  }

  fn render_install_step(
    step: usize,
    title: &'static str,
    command: &'static str,
    description: &'static str,
    cx: &Context<'_, Self>,
  ) -> impl IntoElement {
    let colors = &cx.theme().colors;
    let cmd = command.to_string();

    v_flex()
      .w_full()
      .gap(px(8.))
      .child(
        h_flex()
          .gap(px(8.))
          .items_center()
          .child(
            div()
              .w(px(24.))
              .h(px(24.))
              .rounded_full()
              .bg(colors.primary)
              .flex()
              .items_center()
              .justify_center()
              .text_xs()
              .font_weight(gpui::FontWeight::BOLD)
              .text_color(colors.background)
              .child(step.to_string()),
          )
          .child(
            Label::new(title)
              .text_color(colors.foreground)
              .font_weight(gpui::FontWeight::SEMIBOLD),
          ),
      )
      .child(
        div()
          .ml(px(32.))
          .text_sm()
          .text_color(colors.muted_foreground)
          .child(description),
      )
      .child(
        h_flex()
          .ml(px(32.))
          .mt(px(4.))
          .w_full()
          .gap(px(8.))
          .child(
            div()
              .flex_1()
              .px(px(12.))
              .py(px(8.))
              .bg(colors.sidebar)
              .rounded(px(6.))
              .font_family("monospace")
              .text_sm()
              .text_color(colors.foreground)
              .child(command),
          )
          .child(
            Button::new(("copy", step))
              .icon(IconName::Copy)
              .ghost()
              .small()
              .on_click(move |_ev, _window, cx| {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(cmd.clone()));
              }),
          ),
      )
  }

  fn render_k8s_diagnostic(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
    let colors = &cx.theme().colors;
    let diag = &self.k8s_diagnostic;

    // Determine if there are issues
    let has_issues =
      !diag.kubectl_installed || diag.context_mismatch || (diag.expected_context.is_some() && !diag.api_reachable);

    if !has_issues {
      return div().into_any_element();
    }

    v_flex()
      .w_full()
      .gap(px(12.))
      .pt(px(8.))
      .border_t_1()
      .border_color(colors.border)
      .child(
        h_flex()
          .gap(px(8.))
          .items_center()
          .child(Icon::new(IconName::Info).text_color(colors.warning))
          .child(
            Label::new("Kubernetes Issues Detected")
              .text_color(colors.warning)
              .text_xs()
              .font_weight(gpui::FontWeight::SEMIBOLD),
          ),
      )
      // Show action message if any
      .when_some(self.action_message.clone(), |el, msg| {
        el.child(
          div()
            .w_full()
            .p(px(8.))
            .rounded(px(6.))
            .bg(colors.success.opacity(0.1))
            .text_sm()
            .text_color(colors.success)
            .child(msg),
        )
      })
      // Context mismatch
      .when(diag.context_mismatch, |el| {
        let current = diag.current_context.clone().unwrap_or_default();
        let expected = diag.expected_context.clone().unwrap_or_else(|| "colima".to_string());

        el.child(
          v_flex()
            .w_full()
            .gap(px(8.))
            .p(px(12.))
            .rounded(px(8.))
            .bg(colors.warning.opacity(0.1))
            .child(
              h_flex()
                .gap(px(8.))
                .child(
                  Label::new("Wrong kubectl context")
                    .text_color(colors.foreground)
                    .font_weight(gpui::FontWeight::MEDIUM),
                ),
            )
            .child(
              div()
                .text_sm()
                .text_color(colors.muted_foreground)
                .child(format!("Current: '{current}' | Expected: '{expected}'")),
            )
            .child(
              Button::new("fix-context")
                .label(format!("Switch to '{expected}'"))
                .primary()
                .small()
                .on_click({
                  let ctx = expected.clone();
                  move |_ev, _window, _cx| {
                    let _ = switch_kubectl_context(&ctx);
                  }
                }),
            )
            .child(
              Button::new("refresh-context")
                .label("Refresh")
                .ghost()
                .small()
                .on_click(move |_ev, _window, _cx| {
                  // Will be handled by parent
                }),
            ),
        )
      })
      // K8s API not reachable
      .when(diag.expected_context.is_some() && !diag.api_reachable && !diag.context_mismatch, |el| {
        let error = diag.error_message.clone().unwrap_or_else(|| "Unknown error".to_string());

        el.child(
          v_flex()
            .w_full()
            .gap(px(8.))
            .p(px(12.))
            .rounded(px(8.))
            .bg(colors.danger.opacity(0.1))
            .child(
              h_flex()
                .gap(px(8.))
                .child(
                  Label::new("Kubernetes API not responding")
                    .text_color(colors.foreground)
                    .font_weight(gpui::FontWeight::MEDIUM),
                ),
            )
            .child(div().text_sm().text_color(colors.muted_foreground).child(error))
            .child(
              h_flex()
                .gap(px(8.))
                .child(
                  Button::new("reset-k8s")
                    .label("Reset Kubernetes")
                    .primary()
                    .small()
                    .on_click(move |_ev, _window, _cx| {
                      // This will block - in production you'd want to do this async
                      let _ = reset_colima_k8s();
                    }),
                )
                .child(
                  Button::new("refresh-status")
                    .label("Refresh")
                    .ghost()
                    .small()
                    .on_click(move |_ev, _window, _cx| {
                      // Will be handled by parent
                    }),
                ),
            ),
        )
      })
      .into_any_element()
  }
}

impl Focusable for SetupDialog {
  fn focus_handle(&self, _cx: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

impl Render for SetupDialog {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
    let colors = cx.theme().colors;

    v_flex()
            .w_full()
            .max_h(px(500.))
            .overflow_y_scrollbar()
            .gap(px(16.))
            // Status section
            .child(
                v_flex()
                    .w_full()
                    .gap(px(8.))
                    .child(
                        Label::new("Status")
                            .text_color(colors.muted_foreground)
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD),
                    )
                    .child(Self::render_status_item("Homebrew", self.homebrew_installed, cx))
                    .child(Self::render_status_item("Docker CLI", self.docker_installed, cx))
                    .child(Self::render_status_item("Colima", self.colima_installed, cx)),
            )
            // Installation instructions
            .child(
                v_flex()
                    .w_full()
                    .gap(px(16.))
                    .pt(px(8.))
                    .border_t_1()
                    .border_color(colors.border)
                    .child(
                        Label::new("Installation Steps")
                            .text_color(colors.muted_foreground)
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD),
                    )
                    .when(!self.homebrew_installed, |el| {
                        el.child(Self::render_install_step(
                            1,
                            "Install Homebrew",
                            "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"",
                            "Homebrew is required to install Docker and Colima on macOS.",
                            cx,
                        ))
                    })
                    .when(!self.docker_installed, |el| {
                        el.child(Self::render_install_step(
                            if self.homebrew_installed { 1 } else { 2 },
                            "Install Docker CLI",
                            "brew install docker docker-compose",
                            "Install the Docker command-line tools (no Docker Desktop required).",
                            cx,
                        ))
                    })
                    .when(!self.colima_installed, |el| {
                        el.child(Self::render_install_step(
                            if self.homebrew_installed && self.docker_installed { 1 }
                            else if self.homebrew_installed || self.docker_installed { 2 }
                            else { 3 },
                            "Install Colima",
                            "brew install colima",
                            "Colima provides the container runtime for Docker on macOS.",
                            cx,
                        ))
                    })
                    .child(Self::render_install_step(
                        if self.homebrew_installed && self.docker_installed && self.colima_installed { 1 }
                        else if !self.homebrew_installed && !self.docker_installed && !self.colima_installed { 4 }
                        else if !self.homebrew_installed && (!self.docker_installed || !self.colima_installed)
                          || !self.docker_installed && !self.colima_installed { 3 }
                        else { 2 },
                        "Start Colima",
                        "colima start",
                        "Start the Colima VM to run containers. Use 'colima start --kubernetes' for Kubernetes support.",
                        cx,
                    )),
            )
            // K8s diagnostics section
            .child(self.render_k8s_diagnostic(cx))
            // Help text
            .child(
                div()
                    .w_full()
                    .mt(px(8.))
                    .p(px(12.))
                    .bg(colors.sidebar)
                    .rounded(px(8.))
                    .child(
                        v_flex()
                            .gap(px(4.))
                            .child(
                                h_flex()
                                    .gap(px(8.))
                                    .items_center()
                                    .child(Icon::new(IconName::Info).text_color(colors.primary))
                                    .child(
                                        Label::new("Need help?")
                                            .text_color(colors.foreground)
                                            .font_weight(gpui::FontWeight::MEDIUM),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.muted_foreground)
                                    .child("Visit github.com/abiosoft/colima for documentation and troubleshooting."),
                            ),
                    ),
            )
  }
}
