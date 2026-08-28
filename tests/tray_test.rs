use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use devtray::gui::bridge::SlintAppController;
use devtray::gui::tray::{format_tray_tooltip, load_tray_icon, DevTraySysTray};
use devtray::MainWindow;
use ksni::menu::MenuItem;
use ksni::Tray;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_tray_tooltip_formatting() {
    assert_eq!(format_tray_tooltip(0), "DevTray");
    assert_eq!(format_tray_tooltip(1), "DevTray (1 active)");
    assert_eq!(format_tray_tooltip(4), "DevTray (4 active)");
}

#[test]
fn test_tray_icon_loading() {
    let icon = load_tray_icon().expect("Should load icon from embedded assets");
    assert!(icon.width > 0);
    assert!(icon.height > 0);
    assert_eq!(icon.data.len(), (icon.width * icon.height * 4) as usize);
}

#[test]
fn test_tray_metadata_and_id() {
    let dir = tempdir().unwrap();
    let config = ConfigManager::with_path(dir.path().join("config.json"));
    let logs = LogBroadcaster::new(dir.path().join("logs"), 100);
    let process = ProcessManager::new(logs.clone());
    let controller = Arc::new(SlintAppController::new(config, process, logs));
    let ui_handle = slint::Weak::<MainWindow>::default();

    let tray = DevTraySysTray::new(controller, ui_handle);
    assert_eq!(tray.id(), "devtray");
    assert_eq!(tray.title(), "DevTray");

    let tooltip = tray.tool_tip();
    assert_eq!(tooltip.title, "DevTray");
    assert!(!tooltip.description.is_empty());

    let icons = tray.icon_pixmap();
    assert_eq!(icons.len(), 1);
    assert!(icons[0].width > 0);
}

#[test]
fn test_tray_menu_structure_with_groups_and_uncategorized() {
    let dir = tempdir().unwrap();
    let config = ConfigManager::with_path(dir.path().join("config.json"));
    let logs = LogBroadcaster::new(dir.path().join("logs"), 100);
    let process = ProcessManager::new(logs.clone());
    let controller = Arc::new(SlintAppController::new(config, process, logs));
    let ui_handle = slint::Weak::<MainWindow>::default();

    // Add tasks
    controller.add_task("Backend", "echo backend", ".", Some("Web")).unwrap();
    controller.add_task("Frontend", "echo frontend", ".", Some("Web")).unwrap();
    controller.add_task("Postgres", "echo postgres", ".", Some("Database")).unwrap();
    controller.add_task("Standalone Worker", "echo worker", ".", None).unwrap();

    let tray = DevTraySysTray::new(controller, ui_handle);
    let menu = tray.menu();

    // Menu layout:
    // 0: Standard("Open DevTray")
    // 1: Separator
    // 2: SubMenu("Database")
    // 3: SubMenu("Web")
    // 4: Checkmark("Standalone Worker")
    // 5: Separator
    // 6: Standard("Quit DevTray")
    assert_eq!(menu.len(), 7);

    // 0: Open DevTray
    match &menu[0] {
        MenuItem::Standard(item) => assert_eq!(item.label, "Open DevTray"),
        _ => panic!("Expected StandardItem for Open DevTray"),
    }

    // 1: Separator
    match &menu[1] {
        MenuItem::Separator => (),
        _ => panic!("Expected Separator"),
    }

    // 2: Database SubMenu
    match &menu[2] {
        MenuItem::SubMenu(sub) => {
            assert_eq!(sub.label, "Database");
            assert_eq!(sub.submenu.len(), 4);
            match &sub.submenu[0] {
                MenuItem::Standard(item) => assert_eq!(item.label, "▶ Start All"),
                _ => panic!("Expected Start All"),
            }
            match &sub.submenu[1] {
                MenuItem::Standard(item) => assert_eq!(item.label, "🛑 Stop All"),
                _ => panic!("Expected Stop All"),
            }
            match &sub.submenu[2] {
                MenuItem::Separator => (),
                _ => panic!("Expected Separator in SubMenu"),
            }
            match &sub.submenu[3] {
                MenuItem::Checkmark(item) => {
                    assert_eq!(item.label, "Postgres");
                    assert!(!item.checked);
                }
                _ => panic!("Expected Checkmark for Postgres"),
            }
        }
        _ => panic!("Expected SubMenu for Database"),
    }

    // 3: Web SubMenu
    match &menu[3] {
        MenuItem::SubMenu(sub) => {
            assert_eq!(sub.label, "Web");
            assert_eq!(sub.submenu.len(), 5);
            match &sub.submenu[0] {
                MenuItem::Standard(item) => assert_eq!(item.label, "▶ Start All"),
                _ => panic!("Expected Start All"),
            }
            match &sub.submenu[1] {
                MenuItem::Standard(item) => assert_eq!(item.label, "🛑 Stop All"),
                _ => panic!("Expected Stop All"),
            }
            match &sub.submenu[2] {
                MenuItem::Separator => (),
                _ => panic!("Expected Separator in SubMenu"),
            }
            match &sub.submenu[3] {
                MenuItem::Checkmark(item) => {
                    assert_eq!(item.label, "Backend");
                    assert!(!item.checked);
                }
                _ => panic!("Expected Checkmark for Backend"),
            }
            match &sub.submenu[4] {
                MenuItem::Checkmark(item) => {
                    assert_eq!(item.label, "Frontend");
                    assert!(!item.checked);
                }
                _ => panic!("Expected Checkmark for Frontend"),
            }
        }
        _ => panic!("Expected SubMenu for Web"),
    }

    // 4: Uncategorized task "Standalone Worker"
    match &menu[4] {
        MenuItem::Checkmark(item) => {
            assert_eq!(item.label, "Standalone Worker");
            assert!(!item.checked);
        }
        _ => panic!("Expected Checkmark for Standalone Worker"),
    }

    // 5: Separator
    match &menu[5] {
        MenuItem::Separator => (),
        _ => panic!("Expected Separator before Quit"),
    }

    // 6: Quit DevTray
    match &menu[6] {
        MenuItem::Standard(item) => assert_eq!(item.label, "Quit DevTray"),
        _ => panic!("Expected StandardItem for Quit DevTray"),
    }
}
