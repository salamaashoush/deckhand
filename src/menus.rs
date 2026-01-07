//! Native macOS menu bar
//!
//! Provides the standard macOS menu bar with Dockside, File, and View menus.

use gpui::{Menu, MenuItem, actions};

use crate::keybindings::{
  FocusSearch, GoToActivityMonitor, GoToCompose, GoToContainers, GoToDeployments, GoToImages, GoToMachines,
  GoToNetworks, GoToPods, GoToServices, GoToSettings, GoToVolumes, NewResource, OpenCommandPalette, Refresh,
  ShowKeyboardShortcuts,
};

// Actions for menu items
actions!(dockside_menu, [About, Quit, CloseWindow,]);

/// Build the application menu bar
pub fn app_menus() -> Vec<Menu> {
  vec![
    // Dockside menu (application menu)
    Menu {
      name: "Dockside".into(),
      items: vec![
        MenuItem::action("About Dockside", About),
        MenuItem::separator(),
        MenuItem::action("Settings...", GoToSettings),
        MenuItem::separator(),
        MenuItem::os_submenu("Services", gpui::SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action("Quit Dockside", Quit),
      ],
    },
    // File menu
    Menu {
      name: "File".into(),
      items: vec![
        MenuItem::action("New Resource", NewResource),
        MenuItem::separator(),
        MenuItem::action("Search...", FocusSearch),
        MenuItem::action("Refresh", Refresh),
        MenuItem::separator(),
        MenuItem::action("Close Window", CloseWindow),
      ],
    },
    // View menu
    Menu {
      name: "View".into(),
      items: vec![
        MenuItem::action("Command Palette...", OpenCommandPalette),
        MenuItem::separator(),
        MenuItem::submenu(Menu {
          name: "Docker".into(),
          items: vec![
            MenuItem::action("Containers", GoToContainers),
            MenuItem::action("Compose", GoToCompose),
            MenuItem::action("Images", GoToImages),
            MenuItem::action("Volumes", GoToVolumes),
            MenuItem::action("Networks", GoToNetworks),
          ],
        }),
        MenuItem::submenu(Menu {
          name: "Kubernetes".into(),
          items: vec![
            MenuItem::action("Pods", GoToPods),
            MenuItem::action("Deployments", GoToDeployments),
            MenuItem::action("Services", GoToServices),
          ],
        }),
        MenuItem::separator(),
        MenuItem::action("Machines", GoToMachines),
        MenuItem::action("Activity Monitor", GoToActivityMonitor),
        MenuItem::separator(),
        MenuItem::action("Keyboard Shortcuts", ShowKeyboardShortcuts),
      ],
    },
  ]
}
