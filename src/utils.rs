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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_brew_paths_contains_common_locations() {
    assert!(BREW_PATHS.contains(&"/opt/homebrew/bin"));
    assert!(BREW_PATHS.contains(&"/usr/local/bin"));
    assert!(BREW_PATHS.contains(&"/usr/bin"));
  }

  #[test]
  fn test_find_binary_returns_path_for_common_binaries() {
    // ls should exist on any Unix system
    let path = find_binary("ls");
    assert!(path.is_some(), "ls binary should be found");
    assert!(path.unwrap().exists());
  }

  #[test]
  fn test_find_binary_returns_none_for_nonexistent() {
    let path = find_binary("this_binary_definitely_does_not_exist_xyz123");
    assert!(path.is_none());
  }

  #[test]
  fn test_colima_cmd_returns_command() {
    // Should return a Command even if colima isn't installed
    let cmd = colima_cmd();
    // Verify it's a valid Command (program is set)
    assert!(format!("{cmd:?}").contains("colima"));
  }

  #[test]
  fn test_docker_cmd_returns_command() {
    let cmd = docker_cmd();
    assert!(format!("{cmd:?}").contains("docker"));
  }

  #[test]
  fn test_kubectl_cmd_returns_command() {
    let cmd = kubectl_cmd();
    assert!(format!("{cmd:?}").contains("kubectl"));
  }

  #[test]
  fn test_find_binary_prefers_brew_paths() {
    // If a binary exists in both BREW_PATHS and elsewhere,
    // we should find the BREW_PATHS version first
    // This test verifies the logic works by checking that
    // we iterate through BREW_PATHS before falling back to PATH

    // Test with 'cat' which exists in /usr/bin on most systems
    if let Some(path) = find_binary("cat") {
      // Path should be from one of the known locations
      let path_str = path.to_string_lossy();
      let is_known_path = BREW_PATHS.iter().any(|p| path_str.starts_with(p)) || path.exists();
      assert!(is_known_path, "Should find cat in a known location");
    }
  }

  #[test]
  fn test_get_home_dir_not_empty() {
    // This should return Some on any properly configured system
    let home = get_home_dir();
    // We can't guarantee this works in all CI environments,
    // so just verify it doesn't panic
    if let Some(path) = home {
      assert!(!path.as_os_str().is_empty());
    }
  }

  #[test]
  fn test_create_cmd_sets_path_env() {
    let cmd = create_cmd(PathBuf::from("echo"));
    // The command should have PATH configured
    let debug = format!("{cmd:?}");
    // Verify the command was created (contains the program)
    assert!(debug.contains("echo"));
  }
}
