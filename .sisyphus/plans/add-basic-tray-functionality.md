# Basic Tray Functionality for MemFlow

## TL;DR

> **Quick Summary**: Implement basic tray functionality so closing the window minimizes to system tray instead of quitting. Tray icon shows recording status. Right-click menu provides quick controls.
>
> **Deliverables**:
> - Window close event handler (hide to tray, don't quit)
> - System tray icon with recording status indicator
> - Tray menu: Show Window, Recording Control, Settings, Quit
> - Click/double-click handlers for showing window
>
> **Estimated Effort**: Short
> **Parallel Execution**: NO
> **Critical Path**: Setup tray → Handle close event → Add menu items → Test behavior

---

## Context

### Original Request
Add tray functionality to MemFlow so users can:
- Minimize app to system tray instead of quitting when closing window
- See recording status from tray icon
- Control recording from tray menu

### Current State
Looking at `src-tauri/src/lib.rs`, I can see:
- ✅ `TrayIconBuilder` is already imported (line 27)
- ✅ `TrayOnly` run mode exists (line 40, 54 - default mode)
- ✅ `show_or_create_main_window()` function exists (line 57)
- ✅ `recorder::is_recording()` for status check (line 154)
- ✅ `format_tray_status()` for tooltip (line 152)
- ✅ Tray icon menu setup may already exist

### Product Philosophy
MemFlow is designed to be:
- **Local-first, privacy-first, high performance**
- Background activity recording without user interruption
- User can control when to record/pause
- Integrates with Cursor + MCP for AI context awareness

The tray feature supports this by allowing the app to run quietly in the background.

---

## Work Objectives

### Core Objective
Implement basic tray functionality so the app minimizes to system tray on window close instead of quitting. Provide visual status indicator and quick controls via tray menu.

### Concrete Deliverables
- Window close event handler (hide to tray instead of quit)
- System tray icon with status indicator
- Right-click menu with: Show Window, Recording Control, Settings, Quit
- Click/double-click to show window
- Tray icon tooltip showing current status

### Definition of Done
- [ ] Closing window hides app to tray (doesn't quit)
- [ ] Clicking tray icon shows/hides main window
- [ ] Double-clicking tray icon shows and focuses main window
- [ ] Tray icon shows recording status (dot indicator: green=recording, gray=idle)
- [ ] Right-click menu works with: Show Window, Start/Stop Recording, Settings, Quit
- [ ] Tray tooltip shows: "MemFlow - Status: [Recording/Idle] | OCR: [Running/Stopped]"

### Must Have
- Handle window close event (prevent default, hide window)
- System tray icon with click/double-click handlers
- Tray menu with at least: Show Window, Recording Control, Settings, Quit
- Recording status indicator in tray icon
- Tooltip showing current status

### Must NOT Have
- DO NOT add notifications (not in basic version)
- DO NOT add global hotkeys (not in basic version)
- DO NOT remove existing window functionality
- DO NOT break existing run modes (headless, tray-only, ui)

---

## Execution Strategy

### Parallel Execution Waves
Single sequential task - no parallelization needed.

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 2 | None |
| 2 | 1 | 3 | None |
| 3 | 1, 2 | None | None (final verification) |

---

## TODOs

- [ ] 1. Setup System Tray Icon

  **What to do**:
  In `src-tauri/src/lib.rs`, around line 276+ (setup function):

  1. Create tray icon builder using `TrayIconBuilder`
  2. Set icon path (use app icon: `icons/icon.ico`)
  3. Add click handler: show/hide main window
  4. Add double-click handler: show and focus main window
  5. Add menu with items:
     - "显示主窗口" (Show Window) → calls `show_or_create_main_window()`
     - Separator
     - "开始录制" (Start Recording) → calls `commands::start_recording`
     - "停止录制" (Stop Recording) → calls `commands::stop_recording`
     - Separator
     - "设置" (Settings) → open settings modal or window
     - "退出" (Quit) → calls `app.exit()`

  Reference existing code:
  - `TrayIconBuilder` is already imported (line 27)
  - `show_or_create_main_window(app)` exists (line 57)
  - `commands::start_recording` / `commands::stop_recording` exist (lines 225-226)
  - `recorder::is_recording()` can be used for status (line 154)

  **Must NOT do**:
  - DO NOT remove existing window functionality
  - DO NOT break run modes (tray-only, headless, ui)

  **Recommended Agent Profile**: quick

  **Parallelization**: NO

  **References**:
  - `src-tauri/src/lib.rs:27` - TrayIconBuilder import
  - `src-tauri/src/lib.rs:57` - show_or_create_main_window function
  - `src-tauri/src/commands.rs` - Recording commands
  - Tauri docs: https://tauri.app/v1/guides/features/tray

  **Acceptance Criteria**:
  - [ ] Tray icon appears in system tray
  - [ ] Click icon shows/hides main window
  - [ ] Double-click shows and focuses window
  - [ ] Right-click menu appears with all items

  **Agent-Executed QA Scenarios (MANDATORY)**:

  ```
  Scenario: Verify tray icon appears and is clickable
    Tool: Bash (tauri dev)
    Preconditions: App started with tray mode
    Steps:
      1. Start app: pnpm tauri:dev
      2. Check system tray for MemFlow icon
      3. Right-click tray icon
      4. Verify menu appears with: 显示主窗口, 开始录制, 停止录制, 设置, 退出
      5. Double-click tray icon
      6. Verify main window appears and is focused
    Expected Result: Tray icon visible, menu has 5 items, double-click shows window
    Evidence: Screenshot of tray icon and menu

  Scenario: Verify Show Window menu item works
    Tool: Bash (tauri dev)
    Preconditions: App started, window hidden
    Steps:
      1. Right-click tray icon
      2. Click "显示主窗口" menu item
      3. Verify main window appears
      4. Click window close button
      5. Verify window hides but app stays running
    Expected Result: Window shown on menu click, hide on close button
    Evidence: Observation of window behavior

  Scenario: Verify recording control from tray menu
    Tool: Bash (tauri dev)
    Preconditions: App started, idle state
    Steps:
      1. Right-click tray icon
      2. Click "开始录制"
      3. Observe tray icon (should show green dot)
      4. Right-click tray icon again
      5. Click "停止录制"
      6. Verify icon returns to gray state
    Expected Result: Recording starts/stops, tray icon reflects status
    Evidence: Tray icon status change observed
  ```

  **Commit**: YES
  - Message: `feat(tray): add basic system tray functionality`
  - Files: `src-tauri/src/lib.rs`
  - Pre-commit: `pnpm tauri dev` starts successfully

---

- [ ] 2. Handle Window Close Event

  **What to do**:
  In `src-tauri/src/lib.rs`, around line 276+ (setup function):

  Add window event listener:
  ```rust
  app.on_window_event(|event| match event {
      tauri::WindowEvent::CloseRequested { .. } => {
          // Prevent window from closing, instead hide to tray
          event.prevent_default();
          if let Some(window) = app.get_webview_window("main") {
              let _ = window.hide();
          }
      }
      _ => {}
  })?;
  ```

  **Must NOT do**:
  - DO NOT allow the app to quit when window is closed
  - DO NOT remove existing window functionality

  **Recommended Agent Profile**: quick

  **Parallelization**: NO

  **References**:
  - `src-tauri/src/lib.rs:276` - setup function location
  - Tauri docs: https://tauri.app/v1/guides/features/system-tray

  **Acceptance Criteria**:
  - [ ] Clicking window close button hides window (not quit app)
  - [ ] App continues running in background
  - [ ] Tray icon remains visible

  **Agent-Executed QA Scenarios (MANDATORY)**:

  ```
  Scenario: Verify window close hides to tray
    Tool: Bash (tauri dev)
    Preconditions: App running, main window visible
    Steps:
      1. Start app: pnpm tauri:dev
      2. Wait for main window to appear
      3. Click window close button (X button)
      4. Verify window disappears
      5. Check that app is still running (task manager or tray icon)
      6. Verify tray icon is still visible
    Expected Result: Window hidden, app still running, tray icon remains
    Evidence: Screenshot showing closed window and visible tray icon

  Scenario: Verify double-clicking tray shows window
    Tool: Bash (tauri dev)
    Preconditions: App running, window hidden
    Steps:
      1. With window hidden, double-click tray icon
      2. Verify main window reappears
      3. Verify window is focused (active)
    Expected Result: Window shows and receives focus
    Evidence: Screenshot of focused window
  ```

  **Commit**: NO (group with Task 3)

---

- [ ] 3. Add Recording Status Indicator to Tray Icon

  **What to do**:
  In `src-tauri/src/lib.rs`, modify the tray icon setup to show recording status:

   1. Create/update tray icon based on `recorder::is_recording()` status
  2. When recording: show icon with green dot/overlay
   3. When idle: show gray icon
   4. Update tooltip to show status

  Options:
  - **Option A**: Use icon overlay/badge (Tauri may support this)
  - **Option B**: Switch between two icon files (icon.ico and icon-recording.ico)
  - **Option C**: Use system tray native status (if available)

  Start with Option B (switch icons) as fallback to C:

  ```rust
  let is_recording = recorder::is_recording();
  let icon_path = if is_recording {
      "icons/icon-recording.ico"  // Or same icon with overlay
  } else {
      "icons/icon.ico"
  };

  TrayIconBuilder::new()
      .icon(icon_path)
      .tooltip(format_tray_status(&app))
      ...
  ```

  **Must NOT do**:
  - DO NOT add complex animations initially
  - DO NOT break tray icon display

  **Recommended Agent Profile**: quick

  **Parallelization**: NO (depends on Tasks 1, 2)

  **References**:
  - `src-tauri/src/lib.rs:152` - format_tray_status function
  - `src-tauri/src/lib.rs:154` - recorder::is_recording function
  - `src-tauri/icons/` - Icon files directory

  **Acceptance Criteria**:
  - [ ] Tray icon shows green indication when recording
  - [ ] Tray icon shows gray when idle
  - [ ] Tooltip shows: "MemFlow - Status: Recording | OCR: Running"
  - [ ] Status updates when recording starts/stops

  **Agent-Executed QA Scenarios (MANDATORY)**:

  ```
  Scenario: Verify tray icon status changes
    Tool: Bash (tauri dev)
    Preconditions: App running in idle state
    Steps:
      1. Right-click tray icon
      2. Click "开始录制"
           3. Observe tray icon (should change to green)
      4. Right-click tray icon
      5. Click "停止录制"
      6. Observe tray icon (should return to gray)
    Expected Result: Tray icon changes status on recording state
    Evidence: Observation of icon color change

  Scenario: Verify tray tooltip shows correct status
    Tool: Bash (tauri dev)
    Preconditions: App running
    Steps:
      1. Hover mouse over tray icon
      2. Read tooltip text
      3. Verify tooltip shows: "MemFlow - Status: [Recording/Idle] | OCR: [Running/Stopped]"
      4. Start/stop recording and verify tooltip updates
    Expected Result: Tooltip reflects current status accurately
    Evidence: Screenshot of tray tooltip
  ```

  **Commit**: YES
  - Message: `feat(tray): add recording status indicator to tray icon`
  - Files: `src-tauri/src/lib.rs`, `src-tauri/icons/icon-recording.ico` (if creating new icon)
  - Pre-commit: `pnpm tauri dev` starts successfully

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 3 | `feat(tray): add basic system tray functionality` | src-tauri/src/lib.rs | pnpm tauri dev starts |

---

## Success Criteria

### Verification Commands
```bash
# Start app in tray mode
cd D:\Demo\memflow
pnpm tauri:dev

# Expected behaviors:
# 1. Window starts (or doesn't, depending on run mode)
# 2. Closing window hides to tray (doesn't quit)
# 3. Tray icon appears in system tray
# 4. Right-click shows menu with items
# 5. Double-click shows window
# 6. Recording status visible in icon/tooltip
```

### Final Checklist
- [ ] Window close hides to tray (not quit)
- [ ] Click/double-click tray shows window
- [ ] Right-click menu has: Show Window, Start/Stop Recording, Settings, Quit
- [ ] Tray icon shows recording status
- [ ] Tooltip shows current status
- [ ] App doesn't quit when window closed
- [ ] Existing run modes still work (tray-only, headless, ui)

### Exclusions (Explicitly Out of Scope)
- Notifications system (not in basic version)
- Global hotkeys (not in basic version)
- Daily statistics summary (not in basic version)
- Privacy mode enhancements (not in basic version)
- Removing existing functionality

---

## Appendix: Icon Resources

### Required Icons

| Icon | Purpose | Notes |
|------|---------|-------|
| `icons/icon.ico` | Default idle icon | Already exists |
| `icons/icon-recording.ico` | Recording state icon | May need to create or use overlay |
| `icons/icon.png` | macOS/Linux icon | Already exists |

### Icon Creation (if needed)

If creating a recording variant:
1. Copy `icon.ico` to `icon-recording.ico`
2. Add a green dot overlay or change icon color
3. Or use Tauri's built-in badge/overlay support

---

## Run Mode Behavior

Based on existing code (line 54), default mode is `TrayOnly`:
- App starts in tray-only mode by default
- No window appears until user clicks tray icon
- This is perfect for our use case

To override and show window on start:
- Change line 54 from `RunMode::TrayOnly` to `RunMode::Ui`
- Or use command line: `--ui` flag
