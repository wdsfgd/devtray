# Window Focus from Tray & Group Editable ComboBox Design

- **Status:** approved
- **Date:** 2026-10-01
- **Branch:** `feat/tray-groups-stop-logs`

## Executive Summary

This design addresses two key user experience refinements in DevTray:
1. **Window Focus & Raise from System Tray:** When DevTray's main window is already opened and either hidden behind other application windows or minimized, selecting "Open Window" from the system tray or left-clicking the tray icon now properly brings DevTray to the foreground and gives it input focus via Winit and EWMH (`_NET_ACTIVE_WINDOW`).
2. **Group Editable ComboBox (HTML Datalist Style):** Replaces the cluttered row of quick group chips in `TaskDialog` with a sleek, compact HTML-like editable combobox. Users can freely type a new group name or click a dropdown toggle `▼` to open a floating selection menu containing existing groups and an option to clear/uncategorize.

---

## 1. Window Focus & Raise from System Tray

### Problem Statement
Currently, `DevTraySysTray::activate()` (left-click) and the `open_window` tray menu item only call `ui.show().ok()`. On Linux desktop environments (GNOME Shell, KDE Plasma, Xfce) under X11 and Wayland, if a window is already mapped (visible on another layer or covered by another window), calling `show()` does not un-minimize or raise it above other windows due to Window Manager Focus Stealing Prevention (FSP). The window remains obscured in the background.

### Technical Architecture & Solution
1. **Slint Feature Enablement:**
   Enable the `unstable-winit-030` feature on the `slint` dependency in `Cargo.toml`. This exposes the `WinitWindowAccessor` trait, allowing direct access to the underlying `winit::window::Window`.
2. **Centralized Helper Function:**
   Implement `pub fn show_and_activate(ui: &MainWindow)` in `src/gui/tray.rs`:
   ```rust
   pub fn show_and_activate(ui: &MainWindow) {
       ui.show().ok();
       ui.window().set_minimized(false);
       use slint::winit_030::{winit, WinitWindowAccessor};
       ui.window().with_winit_window(|winit_window: &winit::window::Window| {
           winit_window.set_minimized(false);
           winit_window.focus_window();
           winit_window.request_user_attention(Some(winit::window::UserAttentionType::Critical));
       });
   }
   ```
   - `ui.show().ok()` ensures the window is visible if it was previously hidden via close-to-tray.
   - `ui.window().set_minimized(false)` and `winit_window.set_minimized(false)` restore minimized windows.
   - `winit_window.focus_window()` dispatches `_NET_ACTIVE_WINDOW` client messages on X11 / xdg-activation on Wayland to raise and focus the window.
   - `request_user_attention(Critical)` signals the window manager as a fallback.
3. **Integration Points:**
   - `src/gui/tray.rs`: `activate()` (left-click on tray icon).
   - `src/gui/tray.rs`: `open_window` menu item.
   - `src/main.rs`: single-instance listener callback (when a second instance launches and signals the primary).

---

## 2. Group Editable ComboBox (HTML Datalist Style) in `TaskDialog`

### Problem Statement
The previous quick group chips displayed as a horizontal row of rounded badges beneath the Group text input. While functional, this added unnecessary vertical clutter and broke the clean aesthetic of the Raycast/Linear dark UI.

### Design & Behavior
The Group input field is transformed into an **Editable ComboBox (Datalist style)**:
1. **Main Input Container:**
   - 32px height container (`background: Theme.input-bg`, `border-color: Theme.border`, `border-radius: 6px`).
   - Divided horizontally into:
     - **Left (Text Input):** `groupInput := LineEdit { text <=> root.group_text; placeholder-text: root.tr.placeholder_group; }`. The user can freely type a new group name or edit the existing group string.
     - **Right (Dropdown Button):** A 28px × 100% button with a `▼` indicator (font-size 10px, centered, hover highlight).
2. **Floating Dropdown Popup (`PopupWindow`):**
   - Attached directly beneath the group input container (`y: 34px`, `width: parent.width`).
   - Uses `close-policy: close-on-click-outside`.
   - Styling: `background: Theme.card-bg`, `border-color: Theme.border`, `border-radius: 6px`, drop shadow for depth.
   - **Menu Items:**
     - If `group_text` is non-empty: a `— None (Uncategorized) —` item at the top. Clicking it clears `group_text` (`root.group_text = ""`) and closes the popup.
     - A scrollable list of all strings from `available_groups`.
     - Each item is 26px high with hover feedback (`Theme.accent-hover`).
     - If the item matches current `group_text`, it is highlighted with `Theme.accent`.
     - Clicking an item sets `group_text = item` and closes the popup.
3. **Elimination of Chips:**
   - The entire `HorizontalLayout` for chips under the input field is removed.

---

## 3. Strict Project Constraints

1. **Zero Emojis:** Strictly no emoji characters in Slint UI code or Rust source. Use text glyphs (`▼`, `✕`) or standard labels.
2. **Backward Compatibility:** All existing `config.json` configurations and Slint bridge APIs (`available_groups`, `group_text`) remain 100% compatible.
3. **Bilingual i18n:** Labels and placeholders continue to use `root.tr` for English (`en`) and Mandarin (`zh`).
4. **Non-blocking Execution:** Window activation and combobox interactions run seamlessly on the main UI event loop without blocking background processes.
