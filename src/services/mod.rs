mod dispatcher;
mod task_manager;

pub use dispatcher::*;
pub use task_manager::*;

use gpui::App;

use crate::state::init_docker_state;

/// Initialize all global services
pub fn init_services(cx: &mut App) {
    // Initialize state first
    init_docker_state(cx);

    // Initialize services
    init_task_manager(cx);
    init_dispatcher(cx);
}
