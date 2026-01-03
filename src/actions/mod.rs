use gpui::actions;

// Machine actions
actions!(
  machines,
  [RefreshMachines, ShowCreateMachineDialog, HideCreateMachineDialog,]
);

// Container actions
actions!(containers, [RefreshContainers,]);

// Image actions
actions!(images, [RefreshImages,]);

// Volume actions
actions!(volumes, [RefreshVolumes,]);

// Network actions
actions!(networks, [RefreshNetworks,]);

// Navigation actions
actions!(
  navigation,
  [
    ShowContainers,
    ShowImages,
    ShowVolumes,
    ShowNetworks,
    ShowMachines,
    ShowPods,
    ShowServices,
    ShowActivityMonitor,
  ]
);
