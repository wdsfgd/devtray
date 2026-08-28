use crate::gui::bridge::SlintAppController;
use crate::MainWindow;
use ksni::blocking::TrayMethods;
use slint::ComponentHandle;
use std::sync::Arc;

/// Formats the tray tooltip string dynamically based on the number of active/running tasks.
pub fn format_tray_tooltip(active_count: usize) -> String {
    if active_count == 0 {
        "DevTray".to_string()
    } else {
        format!("DevTray ({} active)", active_count)
    }
}

include!(concat!(env!("OUT_DIR"), "/tray_icon_meta.rs"));
static ICON_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tray_icon.bin"));

/// Loads the pre-decoded PNG icon into a `ksni::Icon` ARGB32 format.
pub fn load_tray_icon() -> Option<ksni::Icon> {
    Some(ksni::Icon {
        width: ICON_WIDTH,
        height: ICON_HEIGHT,
        data: ICON_DATA.to_vec(),
    })
}

/// Linux StatusNotifierItem system tray integration via `ksni`.
pub struct DevTraySysTray {
    pub controller: Arc<SlintAppController>,
    pub ui_handle: slint::Weak<MainWindow>,
}

impl DevTraySysTray {
    /// Creates a new `DevTraySysTray` instance.
    pub fn new(controller: Arc<SlintAppController>, ui_handle: slint::Weak<MainWindow>) -> Self {
        Self {
            controller,
            ui_handle,
        }
    }

    /// Spawns the system tray service in the background.
    pub fn spawn(
        controller: Arc<SlintAppController>,
        ui_handle: slint::Weak<MainWindow>,
    ) -> Option<ksni::blocking::Handle<DevTraySysTray>> {
        let tray = Self::new(controller, ui_handle);
        match tray.assume_sni_available(true).spawn() {
            Ok(handle) => Some(handle),
            Err(e) => {
                eprintln!("[Tray] Failed to spawn system tray: {e}");
                None
            }
        }
    }
}

impl ksni::Tray for DevTraySysTray {
    fn id(&self) -> String {
        "devtray".to_string()
    }

    fn title(&self) -> String {
        format_tray_tooltip(self.controller.running_count())
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let running = self.controller.running_count();
        ksni::ToolTip {
            title: format_tray_tooltip(running),
            description: "DevTray Service Manager".to_string(),
            ..Default::default()
        }
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        load_tray_icon().into_iter().collect()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let ui_weak = self.ui_handle.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.show().ok();
            }
        })
        .ok();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::new();

        // 1. Open DevTray
        let ui_weak = self.ui_handle.clone();
        items.push(
            ksni::menu::StandardItem {
                label: "Open DevTray".to_string(),
                activate: Box::new(move |_| {
                    let ui_weak = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.show().ok();
                        }
                    })
                    .ok();
                }),
                ..Default::default()
            }
            .into(),
        );

        // Separator
        items.push(ksni::MenuItem::Separator);

        let tasks = self.controller.tasks();
        let groups = self.controller.get_groups();

        // 2. Group submenus
        for group in groups {
            let group_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| t.group.as_deref().map(|g| g.trim()) == Some(group.as_str()))
                .cloned()
                .collect();

            let mut sub_items: Vec<ksni::MenuItem<Self>> = Vec::new();

            // ▶ Start All
            let c = self.controller.clone();
            let g = group.clone();
            sub_items.push(
                ksni::menu::StandardItem {
                    label: "▶ Start All".to_string(),
                    activate: Box::new(move |_| {
                        let _ = c.start_group(&g);
                    }),
                    ..Default::default()
                }
                .into(),
            );

            // 🛑 Stop All
            let c = self.controller.clone();
            let g = group.clone();
            sub_items.push(
                ksni::menu::StandardItem {
                    label: "🛑 Stop All".to_string(),
                    activate: Box::new(move |_| {
                        let _ = c.stop_group(&g);
                    }),
                    ..Default::default()
                }
                .into(),
            );

            if !group_tasks.is_empty() {
                sub_items.push(ksni::MenuItem::Separator);
            }

            for task in group_tasks {
                let c = self.controller.clone();
                let task_id = task.id.clone();
                let is_running = self.controller.is_task_running(&task.id);
                sub_items.push(
                    ksni::menu::CheckmarkItem {
                        label: task.name.clone(),
                        checked: is_running,
                        activate: Box::new(move |_| {
                            if c.is_task_running(&task_id) {
                                let _ = c.stop_task(&task_id);
                            } else {
                                let _ = c.start_task(&task_id);
                            }
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }

            items.push(
                ksni::menu::SubMenu {
                    label: group,
                    submenu: sub_items,
                    ..Default::default()
                }
                .into(),
            );
        }

        // 3. Uncategorized tasks
        let uncat_tasks: Vec<_> = tasks
            .iter()
            .filter(|t| match &t.group {
                None => true,
                Some(g) => g.trim().is_empty(),
            })
            .cloned()
            .collect();

        for task in uncat_tasks {
            let c = self.controller.clone();
            let task_id = task.id.clone();
            let is_running = self.controller.is_task_running(&task.id);
            items.push(
                ksni::menu::CheckmarkItem {
                    label: task.name.clone(),
                    checked: is_running,
                    activate: Box::new(move |_| {
                        if c.is_task_running(&task_id) {
                            let _ = c.stop_task(&task_id);
                        } else {
                            let _ = c.start_task(&task_id);
                        }
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        // Separator
        items.push(ksni::MenuItem::Separator);

        // 4. Quit DevTray
        let c = self.controller.clone();
        items.push(
            ksni::menu::StandardItem {
                label: "Quit DevTray".to_string(),
                activate: Box::new(move |_| {
                    c.stop_all();
                    slint::quit_event_loop().ok();
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}
