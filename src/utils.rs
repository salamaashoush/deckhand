//! Utility functions for the application

use std::path::PathBuf;
use std::process::Command;

/// Common Homebrew paths to search for binaries
const BREW_PATHS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

/// Find a binary in PATH or common locations
pub fn find_binary(name: &str) -> Option<PathBuf> {
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

/// Create a Command for colima, finding the binary in common paths
pub fn colima_cmd() -> Command {
  let path = find_binary("colima").unwrap_or_else(|| PathBuf::from("colima"));
  Command::new(path)
}

/// Create a Command for docker, finding the binary in common paths
pub fn docker_cmd() -> Command {
  let path = find_binary("docker").unwrap_or_else(|| PathBuf::from("docker"));
  Command::new(path)
}

/// Create a Command for kubectl, finding the binary in common paths
pub fn kubectl_cmd() -> Command {
  let path = find_binary("kubectl").unwrap_or_else(|| PathBuf::from("kubectl"));
  Command::new(path)
}
