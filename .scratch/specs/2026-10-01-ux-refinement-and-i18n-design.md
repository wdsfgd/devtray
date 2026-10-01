# DevTray UX Refinement & i18n (Mandarin) Design Specification

**Date:** 2026-10-01  
**Status:** Approved  
**Topic:** Hover Reset Bug Fix, Action Layout Refinement, and Bilingual i18n (English & Mandarin)  

---

## 1. Context & Motivation

Following the initial UI styling overhaul, testing revealed:
1. **Hover Reset Bug:** Mouse hover states on `TaskCard` were resetting after 1 second. The 1-second background sync timer in `src/main.rs` called `controller.refresh_tasks(&ui)` which recreated `VecModel` unconditionally, forcing Slint to reconstruct all card components and destroy transient hover state.
2. **Action Layout Usability:** Hiding the Start/Stop toggle button behind hover degraded quick-action ergonomics. Users need the Start/Stop button to remain permanently visible, while secondary text buttons (`Logs`, `Edit`, `Del`) reveal smoothly on hover.
3. **Typography & Plain Text Preference:** Cryptic/minimalist glyphs (`>_`, `✎`, `✕`) are replaced with clear plain text buttons (`Logs`, `Edit`, `Del`).
4. **Internationalization (i18n):** Support for English (`en`) and Mandarin Chinese (`zh` / 简体中文), switchable via a sleek header button and persisted in config.
5. **No Emojis Rule:** Strict project rule — zero emojis across the entire application (use clean text such as `ZH` / `EN` or `中文` / `EN`, not `🌐`).

---

## 2. Architecture & Components

### 2.1 Hover Reset Fix (`src/gui/bridge.rs` & `src/main.rs`)
* **State Change Detection:**
  * `SlintAppController` tracks the last rendered task list fingerprint / snapshot (`Vec<TaskItemSnapshot>`).
  * On the 1-second timer tick, compare existing state against current state (task IDs, order, names, groups, commands, running status).
  * If no task property or running state has changed, skip calling `ui.set_tasks(...)`.
  * If running count or task states have changed, update model.
  * Result: While idling or hovering, no component reconstruction occurs. Mouse hover is persistent and rock-solid.

### 2.2 TaskCard Action Layout (`ui/task_card.slint`)
* **Permanent Start/Stop Button (Rightmost):**
  * Width: 28px, Height: 26px, Radius: 5px.
  * Always visible regardless of hover state.
  * Running: `⏹` icon, `Theme.danger` (`#f87171`), `Theme.danger-surface` background, `Theme.danger-border` border.
  * Stopped: `▶` icon, `Theme.running` (`#10b981`), `#121815` background, `Theme.running-border` border.
* **Secondary Text Action Group (Left of Start/Stop):**
  * Resting: `opacity: 0.0`.
  * Hover (`root.is_hovered`): `opacity: 1.0` with 150ms transition.
  * Clean text buttons (height 26px, radius 5px, 1px `Theme.border`, font-size 11px):
    * `Logs` / `日志`: `Theme.text-secondary` -> `Theme.text-primary` on hover, `Theme.accent` background.
    * `Edit` / `编辑`: `Theme.text-secondary` -> `Theme.text-primary` on hover, `Theme.accent` background.
    * `Del` / `删除`: `Theme.text-secondary` -> `Theme.danger` on hover, `Theme.danger-hover-bg` background, `Theme.danger-border` border.

### 2.3 i18n Architecture (`src/core/i18n.rs` & `ui/main_window.slint`)
* **Language Enum:**
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "lowercase")]
  pub enum Language {
      En,
      Zh,
  }
  ```
* **Persistent Configuration:**
  * Extended `AppConfig` or `ConfigManager` to support optional `language: "en" | "zh"` in `~/.config/devtray/config.json`.
* **Slint Bridge Integration:**
  * Struct `I18nStrings` defined in `ui/main_window.slint` and populated by `SlintAppController`.
  * Header Language Switcher:
    * Compact pill button next to `+ New Task`.
    * When in English, button label: `ZH` or `中文`.
    * When in Mandarin, button label: `EN`.
    * Clicking triggers `controller.toggle_language(&ui)`, immediately re-binding all localized strings and saving config.
* **Localization Dictionary:**
  * Zero emojis in any text string.
  * Complete coverage across Main Window, Task Cards, Dialogs, and Log Viewer.

---

## 3. Verification & Testing Strategy

1. **Unit & Bridge Tests:**
   * Test `Language` serialization, deserialization, and toggle.
   * Test `has_changed` snapshot detection to verify `set_tasks` is not called when processes remain unchanged.
2. **Hover Stability Test:**
   * Verify mouse hover on TaskCard persists across 5+ seconds without interruption.
3. **i18n Verification:**
   * Verify clicking language toggle swaps all text labels instantly between English and Mandarin.
   * Verify restarting application preserves selected language.
4. **Emoji Check:**
   * Automated scan to verify zero emoji characters in UI templates and Rust source.
