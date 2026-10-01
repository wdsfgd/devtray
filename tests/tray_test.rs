use devtray::core::config::ConfigManager;
use devtray::core::i18n::Language;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use devtray::gui::bridge::SlintAppController;
use devtray::gui::tray::{format_tray_tooltip, load_tray_icon, DevTraySysTray};
use devtray::MainWindow;
use ksni::menu::MenuItem;
use ksni::Tray;
use slint::ComponentHandle;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_tray_tooltip_formatting() {
    assert_eq!(format_tray_tooltip(0), "DevTray");
    assert_eq!(format_tray_tooltip(1), "DevTray (1 active)");
    assert_eq!(format_tray_tooltip(4), "DevTray (4 active)");
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_menu_on_activate_constant() {
    assert!(<DevTraySysTray as Tray>::MENU_ON_ACTIVATE);
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
    controller
        .add_task("Backend", "echo backend", ".", Some("Web"))
        .unwrap();
    controller
        .add_task("Frontend", "echo frontend", ".", Some("Web"))
        .unwrap();
    controller
        .add_task("Postgres", "echo postgres", ".", Some("Database"))
        .unwrap();
    controller
        .add_task("Standalone Worker", "echo worker", ".", None)
        .unwrap();

    let tray = DevTraySysTray::new(controller.clone(), ui_handle);
    let menu = tray.menu();

    // Menu layout:
    // 0: Standard("Open Window")
    // 1: Separator
    // 2: SubMenu("Database")
    // 3: SubMenu("Web")
    // 4: Checkmark("Standalone Worker")
    // 5: Separator
    // 6: Standard("Quit")
    assert_eq!(menu.len(), 7);

    // 0: Open Window
    match &menu[0] {
        MenuItem::Standard(item) => assert_eq!(item.label, "Open Window"),
        _ => panic!("Expected StandardItem for Open Window"),
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
                MenuItem::Standard(item) => assert_eq!(item.label, "⏹ Stop All"),
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
                MenuItem::Standard(item) => assert_eq!(item.label, "⏹ Stop All"),
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

    // 6: Quit
    match &menu[6] {
        MenuItem::Standard(item) => assert_eq!(item.label, "Quit"),
        _ => panic!("Expected StandardItem for Quit"),
    }

    // Dynamic Localization Verification: Mandarin
    controller.set_language(Language::Zh);
    let zh_menu = tray.menu();
    assert_eq!(zh_menu.len(), 7);

    // 0: Open Window (Mandarin)
    match &zh_menu[0] {
        MenuItem::Standard(item) => assert_eq!(item.label, "打开窗口"),
        _ => panic!("Expected StandardItem for 打开窗口"),
    }

    // 2: Database SubMenu (Mandarin Start All & Stop All)
    match &zh_menu[2] {
        MenuItem::SubMenu(sub) => {
            assert_eq!(sub.label, "Database");
            match &sub.submenu[0] {
                MenuItem::Standard(item) => assert_eq!(item.label, "▶ 全部启动"),
                _ => panic!("Expected ▶ 全部启动"),
            }
            match &sub.submenu[1] {
                MenuItem::Standard(item) => assert_eq!(item.label, "⏹ 全部停止"),
                _ => panic!("Expected ⏹ 全部停止"),
            }
        }
        _ => panic!("Expected SubMenu for Database"),
    }

    // 3: Web SubMenu (Mandarin Start All & Stop All)
    match &zh_menu[3] {
        MenuItem::SubMenu(sub) => {
            assert_eq!(sub.label, "Web");
            match &sub.submenu[0] {
                MenuItem::Standard(item) => assert_eq!(item.label, "▶ 全部启动"),
                _ => panic!("Expected ▶ 全部启动"),
            }
            match &sub.submenu[1] {
                MenuItem::Standard(item) => assert_eq!(item.label, "⏹ 全部停止"),
                _ => panic!("Expected ⏹ 全部停止"),
            }
        }
        _ => panic!("Expected SubMenu for Web"),
    }

    // 6: Quit (Mandarin)
    match &zh_menu[6] {
        MenuItem::Standard(item) => assert_eq!(item.label, "退出"),
        _ => panic!("Expected StandardItem for 退出"),
    }
}

#[test]
fn test_show_and_activate_helper() {
    let window = MainWindow::new().unwrap();
    devtray::gui::tray::show_and_activate(&window);
    assert!(!window.window().is_minimized());
}
