# Window Focus & Group Editable ComboBox Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix DevTray window activation/focus from system tray when already opened, and replace TaskDialog group quick chips with an HTML datalist-style editable ComboBox.

**Architecture:** Enable Slint's `unstable-winit-030` feature to access underlying `winit::window::Window` for EWMH `_NET_ACTIVE_WINDOW` focus and unminimizing in `show_and_activate()`. Replace the chips row in `ui/task_dialog.slint` with an editable text input and a `PopupWindow` dropdown displaying available groups and a clear/uncategorized option.

**Tech Stack:** Rust 2021, Slint 1.17, Winit 0.30, KSNI (StatusNotifierItem), Linux (X11 / Wayland).

## Global Constraints

- Zero emojis across all UI templates, strings, and Rust source files (strict project rule).
- Retain full backward compatibility with existing `config.json` files and public APIs.
- All newly introduced or modified UI strings must support both English (en) and Mandarin (zh) using `root.tr`.
- Do not block the Slint UI main thread during any operations.

---

### Task 1: Window Focus & Raise from Tray via `unstable-winit-030`

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/gui/tray.rs`
- Modify: `src/main.rs`
- Test: `tests/tray_test.rs`

- [ ] **Step 1: Update `Cargo.toml` dependencies**
Add `"unstable-winit-030"` to the `slint` dependency features in `Cargo.toml`:
```toml
slint = { version = "1.17.1", default-features = false, features = ["std", "compat-1-2", "backend-winit", "renderer-software", "unstable-winit-030"] }
```

- [ ] **Step 2: Implement `show_and_activate` in `src/gui/tray.rs`**
Add the public helper function in `src/gui/tray.rs`:
```rust
/// Shows the window, restores it if minimized, and requests active input focus from the window manager.
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

- [ ] **Step 3: Wire `show_and_activate` in `src/gui/tray.rs` and `src/main.rs`**
In `src/gui/tray.rs`:
- Update `activate()` to call `show_and_activate(&ui)` instead of `ui.show().ok()`.
- Update `open_window` standard item to call `show_and_activate(&ui)` instead of `ui.show().ok()`.

In `src/main.rs`:
- In the single instance listener (`guard.start_listener(...)`), call `devtray::gui::tray::show_and_activate(&ui)` instead of `ui.show().ok()`.

- [ ] **Step 4: Add unit test in `tests/tray_test.rs`**
Add a test verifying that `show_and_activate` executes safely on a `MainWindow` instance:
```rust
#[test]
fn test_show_and_activate_helper() {
    let window = MainWindow::new().unwrap();
    devtray::gui::tray::show_and_activate(&window);
    assert!(!window.window().is_minimized());
}
```

- [ ] **Step 5: Run tests to verify**
Run: `cargo test --test tray_test`
Expected: 6/6 tests pass.

- [ ] **Step 6: Commit**
```bash
git add Cargo.toml src/gui/tray.rs src/main.rs tests/tray_test.rs
git commit -m "feat(tray): raise and focus window using winit on tray activate and open window"
```

---

### Task 2: Group Editable ComboBox in `TaskDialog`

**Files:**
- Modify: `ui/task_dialog.slint`
- Test: `tests/slint_bridge_test.rs`

- [ ] **Step 1: Replace group quick chips with HTML-like ComboBox in `ui/task_dialog.slint`**
In `ui/task_dialog.slint`:
- Remove the `if root.available_groups.length > 0 : HorizontalLayout { ... }` chip block.
- Upgrade the Group input container into an editable combobox with an attached `PopupWindow`:
  ```slint
  // Field 4: Group (Editable ComboBox / Datalist style)
  VerticalLayout {
      spacing: 4px;

      Text {
          text: root.tr.field_group;
          color: Theme.text-secondary;
          font-size: 11px;
          font-weight: 600;
      }

      groupContainer := Rectangle {
          height: 32px;
          background: Theme.input-bg;
          border-color: groupInput.has-focus ? Theme.focus-border : Theme.border;
          border-width: 1px;
          border-radius: 6px;

          HorizontalLayout {
              padding-left: 0px;
              padding-right: 0px;

              groupInput := LineEdit {
                  horizontal-stretch: 1;
                  height: 100%;
                  text <=> root.group_text;
                  placeholder-text: root.tr.placeholder_group;
              }

              // Dropdown arrow toggle button
              dropdownBtn := Rectangle {
                  width: 30px;
                  height: 100%;
                  background: dropdownTouch.has-hover ? Theme.accent : transparent;
                  border-top-right-radius: 6px;
                  border-bottom-right-radius: 6px;

                  dropdownTouch := TouchArea {
                      mouse-cursor: pointer;
                      clicked => {
                          if root.available_groups.length > 0 || root.group_text != "" {
                              groupPopup.show();
                          }
                      }
                  }

                  Text {
                      text: "▼";
                      color: dropdownTouch.has-hover ? Theme.text-primary : Theme.text-muted;
                      font-size: 9px;
                      horizontal-alignment: center;
                      vertical-alignment: center;
                  }
              }
          }

          groupPopup := PopupWindow {
              x: 0;
              y: parent.height + 2px;
              width: parent.width;
              close-policy: close-on-click-outside;

              Rectangle {
                  background: Theme.card-bg;
                  border-color: Theme.border;
                  border-width: 1px;
                  border-radius: 6px;
                  clip: true;

                  VerticalLayout {
                      padding: 4px;
                      spacing: 2px;

                      // Clear / Uncategorized option if group is set
                      if root.group_text != "" : Rectangle {
                          height: 26px;
                          border-radius: 4px;
                          background: noneTouch.has-hover ? Theme.accent-hover : transparent;

                          noneTouch := TouchArea {
                              mouse-cursor: pointer;
                              clicked => {
                                  root.group_text = "";
                                  groupPopup.close();
                              }
                          }

                          Text {
                              x: 8px;
                              y: (parent.height - self.height) / 2;
                              text: "— " + root.tr.uncategorized + " —";
                              color: Theme.text-muted;
                              font-size: 11px;
                              font-style: italic;
                          }
                      }

                      for grp in root.available_groups : Rectangle {
                          height: 26px;
                          border-radius: 4px;
                          background: root.group_text == grp ? Theme.accent : (grpTouch.has-hover ? Theme.accent-hover : transparent);

                          grpTouch := TouchArea {
                              mouse-cursor: pointer;
                              clicked => {
                                  root.group_text = grp;
                                  groupPopup.close();
                              }
                          }

                          Text {
                              x: 8px;
                              y: (parent.height - self.height) / 2;
                              text: grp;
                              color: root.group_text == grp ? Theme.text-primary : Theme.text-secondary;
                              font-size: 12px;
                              font-weight: root.group_text == grp ? 600 : 400;
                          }
                      }
                  }
              }
          }
      }
  }
  ```

- [ ] **Step 2: Update/verify bridge tests in `tests/slint_bridge_test.rs`**
Ensure `test_bridge_task_dialog_available_groups_and_stop_command` continues to pass and tests that available groups can be passed to and read from the dialog.

- [ ] **Step 3: Run Slint compiler and bridge tests**
Run: `cargo test --test slint_bridge_test`
Expected: All 12 bridge tests PASS.

- [ ] **Step 4: Commit**
```bash
git add ui/task_dialog.slint tests/slint_bridge_test.rs
git commit -m "feat(ui): replace task dialog group quick chips with HTML-like editable combobox"
```

---

### Task 3: End-to-End Verification & Suite

**Files:**
- Test all modified files: `Cargo.toml`, `src/gui/tray.rs`, `src/main.rs`, `ui/task_dialog.slint`, `tests/tray_test.rs`, `tests/slint_bridge_test.rs`.

- [ ] **Step 1: Run full automated test suite**
Run: `cargo test --all`
Expected: All unit & integration tests PASS.

- [ ] **Step 2: Run release build check**
Run: `cargo build --release`
Expected: Zero warnings, zero errors.

- [ ] **Step 3: Verify zero emojis**
Run: `python3 -c "import re, glob; emojis = [f for f in glob.glob('ui/**/*.slint', recursive=True) + glob.glob('src/**/*.rs', recursive=True) + glob.glob('tests/**/*.rs', recursive=True) if re.search(r'[\U00010000-\U0010ffff]', open(f).read())]; assert not emojis, f'Found emojis in {emojis}'"`
Expected: Clean exit (0 emojis found).

- [ ] **Step 4: Commit verification**
```bash
git add -A
git commit -m "chore: verify test suite and release build for window focus and group combobox"
```
