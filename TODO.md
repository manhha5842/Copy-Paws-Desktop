# CopyPaws Desktop - Detailed TODO

> **Goal**: Implement "Full Flow" optimized for Windows.

## 1. Auto-Connect (High Priority)
- [x] **mDNS Enhancement**:
  - Add `server_id` and `version` to mDNS TXT records.
  - Ensure mDNS service restarts correctly if network changes (optional but good).
- [x] **WebSocket Handshake**:
  - Update `handle_handshake` to check `device_id` in database.
  - If device exists & allows (not blocked), skip Pairing and allow connection via `shared_secret` (or simple ID check for MVP).
  - Return detailed error codes in Handshake response (e.g., `DEVICE_REVOKED`, `DEVICE_NOT_FOUND`).

## 2. System Tray & Window Management (High Priority)
- [x] **Tray Icon**:
  - Add System Tray icon.
  - Context Menu: "Show", "Pause Sync", "Quit".
- [x] **Window Behavior**:
  - "Close" button should minimize to Tray (don't kill app).
  - App should start hidden (minimized) if launched via Autostart.
- [x] **Single Instance**:
  - Ensure only one instance runs. Focus existing window if second instance launched.

## 3. Autostart (Medium Priority)
- [x] Verify `tauri-plugin-autostart` works correctly on Windows.
- [x] Add "Start with Windows" toggle in Settings UI.

## 4. Rich Clipboard Support (Medium Priority)
- [x] **Images**:
  - detect Image bitmap in clipboard.
  - Convert to suitable format (PNG/JPEG) for transfer.
  - Handle receiving images from Mobile.
- [ ] **Files** (Low):
  - Just file paths for now? Or actual file transfer? (Maybe later).

## 5. UI/UX Polish
- [ ] Show clearer "Connected" notification when Mobile reconnects automatically.
- [ ] "Last Sync" timestamp for each device.
