//! Utility functions for the application

use std::path::PathBuf;
use std::process::Command;

/// Common Homebrew paths to search for binaries
const BREW_PATHS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

/// Find a binary in common locations (prioritize known paths over PATH)
pub fn find_binary(name: &str) -> Option<PathBuf> {
  // Check common Homebrew/system locations FIRST
  // This is more reliable than PATH when launched from GUI launchers
  for base in BREW_PATHS {
    let path = std::path::Path::new(base).join(name);
    if path.exists() {
      return Some(path);
    }
  }

  // Fallback to PATH lookup
  if let Ok(path) = which::which(name) {
    return Some(path);
  }

  None
}

/// Get the user's home directory reliably
fn get_home_dir() -> Option<PathBuf> {
  // Try HOME env var first
  if let Ok(home) = std::env::var("HOME")
    && !home.is_empty()
  {
    return Some(PathBuf::from(home));
  }

  // Fallback: try to get from /Users/<username> on macOS
  #[cfg(target_os = "macos")]
  {
    if let Ok(user) = std::env::var("USER") {
      let home = PathBuf::from(format!("/Users/{user}"));
      if home.exists() {
        return Some(home);
      }
    }

    // Last resort: check if we're running as the console user
    if let Ok(output) = Command::new("id").arg("-un").output()
      && output.status.success()
    {
      let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
      let home = PathBuf::from(format!("/Users/{user}"));
      if home.exists() {
        return Some(home);
      }
    }
  }

  None
}

/// Create a Command with proper environment for GUI-launched apps
fn create_cmd(path: PathBuf) -> Command {
  let mut cmd = Command::new(path);

  // Ensure HOME is set (critical for colima to find ~/.colima)
  if std::env::var("HOME").is_err()
    && let Some(home) = get_home_dir()
  {
    cmd.env("HOME", home);
  }

  // Ensure PATH includes common binary locations
  let current_path = std::env::var("PATH").unwrap_or_default();
  if !current_path.contains("/opt/homebrew/bin") {
    let new_path = format!("/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:{current_path}");
    cmd.env("PATH", new_path);
  }

  cmd
}

/// Create a Command for colima, finding the binary in common paths
pub fn colima_cmd() -> Command {
  let path = find_binary("colima").unwrap_or_else(|| PathBuf::from("colima"));
  create_cmd(path)
}

/// Create a Command for docker, finding the binary in common paths
pub fn docker_cmd() -> Command {
  let path = find_binary("docker").unwrap_or_else(|| PathBuf::from("docker"));
  create_cmd(path)
}

/// Create a Command for kubectl, finding the binary in common paths
pub fn kubectl_cmd() -> Command {
  let path = find_binary("kubectl").unwrap_or_else(|| PathBuf::from("kubectl"));
  create_cmd(path)
}
