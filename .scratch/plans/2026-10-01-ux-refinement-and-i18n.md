# DevTray UX Refinement & i18n (Mandarin) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the 1-second hover reset bug, make the Start/Stop toggle button permanently visible on TaskCard, restore text buttons (`Logs`, `Edit`, `Del`) revealed on hover, and implement bilingual i18n (English & Mandarin Chinese) with zero emojis.

**Architecture:** Add `src/core/i18n.rs` with `Language` enum and complete localized dictionary. Update `SlintAppController` with snapshot-based change detection so `ui.set_tasks` is never called when task states are idle/unchanged (fixing the hover reset bug). Bind localized strings into Slint components, add a header language switcher (`ZH` / `EN`), and refine `TaskCard` action button layout.

**Tech Stack:** Rust, Slint declarative UI, `serde`, `cargo test`.

## Global Constraints
- Zero emojis across the entire application (no 🌐, no 💻, etc. — use clean text `ZH` / `EN`).
- Start/Stop button (`▶` / `⏹`) must be permanently visible on each `TaskCard`.
- `Logs`, `Edit`, `Del` buttons must be text (not cryptic icons) and reveal on card hover.
- Supported languages: English (`en`) and Mandarin Chinese (`zh`).
- Language choice must persist in `~/.config/devtray/config.json`.
- Slint hover state must never reset due to timer polling.

---

### Task 1: Core i18n Module & Language Configuration

**Files:**
- Create: `src/core/i18n.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/core/config.rs`
- Modify: `src/lib.rs`
- Test: `tests/config_test.rs`

**Interfaces:**
- Produces: `Language` enum (`En`, `Zh`), `I18nStrings` struct with all UI translation strings, `AppConfig` with `language` field.

- [ ] **Step 1: Write test for Language and config persistence**

Add test in `tests/config_test.rs`:
```rust
#[test]
fn test_language_config_persistence() {
    use devtray::core::i18n::Language;
    let config = ConfigManager::new();
    // Test default language is English, and saving/loading persists language
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_test test_language_config_persistence`  
Expected: FAIL (module not found).

- [ ] **Step 3: Implement `src/core/i18n.rs` and update `src/core/config.rs`**

Implement `Language` enum with `toggle()` method, `I18nStrings` struct with complete English and Mandarin strings (zero emojis), and update `ConfigManager` to serialize/deserialize `language`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test config_test`  
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/i18n.rs src/core/mod.rs src/core/config.rs src/lib.rs tests/config_test.rs
git commit -m "feat(i18n): add Language enum, translation dictionary, and config persistence"
```

---

### Task 2: Hover Reset Bug Fix & Smart Diffing in Bridge

**Files:**
- Modify: `src/gui/bridge.rs`
- Test: `tests/slint_bridge_test.rs`

**Interfaces:**
- Consumes: `SlintAppController`.
- Produces: `refresh_tasks(&self, ui: &MainWindow)` with smart change detection preventing redundant `ui.set_tasks` calls.

- [ ] **Step 1: Write unit/integration test for change detection**

Add test in `tests/slint_bridge_test.rs` verifying that calling `refresh_tasks` sequentially with unchanged processes does not trigger unnecessary model rebuilds.

- [ ] **Step 2: Implement snapshot tracking in `SlintAppController`**

In `src/gui/bridge.rs`:
Store `last_task_snapshot: Arc<Mutex<Vec<TaskItemSnapshot>>>`.
In `refresh_tasks`:
1. Collect current `items: Vec<TaskItem>`.
2. Compare against `last_task_snapshot`.
3. If identical, update only `running_count` if changed, and SKIP `ui.set_tasks(...)`.
4. If changed, update `last_task_snapshot` and call `ui.set_tasks(...)`.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test --test slint_bridge_test`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/gui/bridge.rs tests/slint_bridge_test.rs
git commit -m "fix(ui): eliminate hover reset bug with smart task change detection in bridge"
```

---

### Task 3: Slint i18n Binding & Header Switcher

**Files:**
- Modify: `ui/main_window.slint`
- Modify: `src/gui/bridge.rs`

**Interfaces:**
- Consumes: `I18nStrings` from `src/core/i18n.rs`.
- Produces: `in property <I18nData> tr` in `MainWindow`, `callback toggle_language()`, and header pill button (`ZH` / `EN`).

- [ ] **Step 1: Define `I18nData` in `ui/main_window.slint` and add header language switcher**

Add `I18nData` struct in `main_window.slint`. Add language switcher button next to `+ New Task`:
- If current language is EN, button shows `ZH` (hover `Theme.accent-hover`).
- If current language is ZH, button shows `EN`.
- Click callback: `root.toggle_language()`.

- [ ] **Step 2: Wire `toggle_language` callback in `SlintAppController`**

In `src/gui/bridge.rs`:
Implement callback handler: toggle controller's language, save to config, and push updated `tr` struct to UI.

- [ ] **Step 3: Run tests to verify compilation and behavior**

Run: `cargo test`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add ui/main_window.slint src/gui/bridge.rs
git commit -m "feat(ui): add header language switcher button and bridge localization binding"
```

---

### Task 4: TaskCard Layout Refinement (Permanent Start/Stop + Hover Text Buttons)

**Files:**
- Modify: `ui/task_card.slint`
- Modify: `ui/main_window.slint`

**Interfaces:**
- Consumes: `TaskItem`, `I18nData`.
- Produces: `TaskCard` with permanently visible Start/Stop toggle button and hover-revealed text buttons (`Logs`, `Edit`, `Del`).

- [ ] **Step 1: Update `TaskCard` layout in `ui/task_card.slint`**

1. Right section:
   - Left sub-row: Text buttons (`Logs`, `Edit`, `Del`) wrapped in a container with `opacity: root.is_hovered ? 1.0 : 0.0` and 150ms transition.
   - Rightmost sub-item: Toggle button (`▶` / `⏹`) **permanently visible** with 28px × 26px size.
2. Button labels use passed localized text properties: `tr_logs`, `tr_edit`, `tr_del`.

- [ ] **Step 2: Update `MainWindow` to pass translation strings to `TaskCard`**

In `ui/main_window.slint`, pass `root.tr.logs`, `root.tr.edit`, `root.tr.del` into `TaskCard`.

- [ ] **Step 3: Run tests to verify compilation**

Run: `cargo test --test slint_bridge_test`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add ui/task_card.slint ui/main_window.slint
git commit -m "style(ui): make Start/Stop permanently visible and restore hover text action buttons"
```

---

### Task 5: Complete Localization of Modals, Empty State & Log Viewer

**Files:**
- Modify: `ui/main_window.slint`
- Modify: `ui/task_dialog.slint`
- Modify: `ui/confirm_dialog.slint`
- Modify: `ui/log_viewer.slint`

**Interfaces:**
- Consumes: `I18nData`.
- Produces: Full bilingual localization of all modals, empty states, and log viewer buttons (zero emojis).

- [ ] **Step 1: Update `TaskDialog`, `ConfirmDialog`, and `LogViewer` to accept localized strings**

Bind all titles, field labels, empty state hints, confirm messages, and buttons to `tr` properties.

- [ ] **Step 2: Verify all strings contain zero emojis**

Ensure no emoji characters are present in any template.

- [ ] **Step 3: Run tests**

Run: `cargo test`  
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add ui/task_dialog.slint ui/confirm_dialog.slint ui/log_viewer.slint ui/main_window.slint
git commit -m "feat(ui): complete bilingual localization across all dialogs and log viewer"
```

---

### Task 6: Full Verification, Release Build & Emoji Audit

**Files:**
- Verify: all touched files

- [ ] **Step 1: Emoji audit scan**

Scan repository for non-ASCII emoji characters in `src/` and `ui/`. Must return 0 occurrences.

- [ ] **Step 2: Run complete test suite**

Run: `cargo test --all-targets`  
Expected: PASS with 0 failures.

- [ ] **Step 3: Build release binary**

Run: `cargo build --release`  
Expected: Clean release build with 0 warnings.

- [ ] **Step 4: Final commit if any residual adjustments needed**
