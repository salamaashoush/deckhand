//! Kubernetes resource operations (pods, services, deployments)

pub mod deployments;
pub mod pods;
pub mod services;

pub use deployments::*;
pub use pods::*;
pub use services::*;
