# CopyPaws Desktop

**Hub server** chính trong hệ sinh thái CopyPaws - đồng bộ clipboard giữa desktop và mobile devices.

## 🚀 Quick Start

```bash
# Cài dependencies
npm install

# Chạy development mode
npm run tauri dev

# Build production
npm run tauri build
```

## 📁 Cấu trúc

```
Desktop/
├── src/                    # React Frontend (TypeScript)
│   ├── App.tsx            # Main application
│   ├── App.css            # Styling
│   ├── HistoryWindow.tsx  # Clipboard history popup
│   └── main.tsx           # Entry point
│
├── src-tauri/             # Rust Backend
│   └── src/
│       ├── lib.rs         # Tauri commands & app initialization
│       ├── websocket.rs   # WebSocket server (port 8765)
│       ├── clipboard.rs   # Clipboard monitoring
│       ├── crypto.rs      # AES-256-GCM encryption
│       ├── pairing.rs     # QR code pairing
│       ├── sync_manager.rs # Sync orchestration
│       ├── mdns.rs        # mDNS service discovery
│       ├── shortcuts.rs   # Global hotkeys
│       └── database/      # SQLite storage
│
└── public/                # Static assets
```

## ✅ Tính năng hoàn thành

### Core Features
- ✅ **WebSocket Server** - Multi-client, port 8765
- ✅ **Clipboard Monitoring** - Real-time với anti-loop
- ✅ **AES-256-GCM Encryption** - End-to-end encryption
- ✅ **QR Code Pairing** - Token expires sau 5 phút
- ✅ **SQLite Database** - Lưu clips, devices, settings
- ✅ **mDNS Discovery** - Auto-discover service
- ✅ **Sync Manager** - Auto/Hotkey/ReceiveOnly/Paused modes

### Global Shortcuts
- ✅ `Ctrl+Alt+C` - Push clipboard to devices
- ✅ `Ctrl+Alt+V` - Paste latest (simulates Ctrl+V)
- ✅ `Super+Alt+V` - Toggle history window

### Dashboard UI
- ✅ Server status display
- ✅ Clipboard history list
- ✅ Connected devices list
- ✅ Real-time device count
- ✅ QR code generation
- ✅ Pin/Delete clips
- ✅ Block/Revoke devices
- ✅ Settings page
- ✅ Autostart toggle | Sync mode control
- ✅ History popup window

## 🔧 Sync Modes

| Mode | Mô tả |
|------|-------|
| **Auto** | Tự động sync mỗi khi copy |
| **HotkeyOnly** | Chỉ sync khi nhấn hotkey |
| **ReceiveOnly** | Chỉ nhận, không gửi |
| **Paused** | Tạm dừng hoàn toàn |

## 🧪 Testing

Sử dụng `test-client` để test WebSocket:

1. Chạy Desktop app: `npm run tauri dev`
2. Mở `../test-client/index.html` trong browser
3. Kết nối đến `localhost:8765`
4. Test pairing và clipboard sync

## 📊 Tiến độ

Xem chi tiết tại [PROGRESS.md](./PROGRESS.md)

| Module | Status |
|--------|--------|
| WebSocket Server | ✅ 100% |
| Clipboard Monitoring | ✅ 100% |
| Encryption | ✅ 100% |
| Pairing | ✅ 100% |
| Database | ✅ 100% |
| Global Shortcuts | ✅ 100% |
| Frontend Dashboard | ✅ 100% |

## 📋 TODO

- [ ] System tray integration
- [ ] Multiple clipboard formats (images, files)
- [ ] Cloud relay (sync qua internet)
- [ ] Auto-pair (không cần QR mỗi lần)

## 📚 API Reference

Xem [MOBILE_API_REFERENCE.md](./MOBILE_API_REFERENCE.md) để hiểu protocol WebSocket.

---

Part of the **CopyPaws** ecosystem.
