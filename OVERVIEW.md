# CopyPaws Desktop - Overview

**Phiên bản:** 1.1  
**Trạng thái:** Production Ready  
**Mô hình:** Local-first, WebSocket P2P (LAN)

---

## Mục tiêu

Desktop App đóng vai trò **Hub trung tâm** trong hệ sinh thái CopyPaws:

- **Core:** Theo dõi clipboard hệ thống (Windows/macOS/Linux) và đồng bộ nội dung **Text** tức thì
- **Connectivity:** Kết nối các thiết bị mobile qua mạng nội bộ (LAN) sử dụng **WebSocket**
- **Management:** Giao diện quản lý thiết bị, lịch sử copy, và cấu hình bảo mật
- **Performance:** Hoạt động nhẹ, tiết kiệm tài nguyên, chạy nền (System Tray)

---

## Tính năng chính

### Hub WebSocket Server
- Tự động khởi chạy server nội bộ (port 8765)
- Multi-client support
- Pause/Resume sync

### Pairing & Security
- Pairing qua **QR Code** (Token + IP + Shared Secret)
- Mã hoá **AES-256-GCM** cho mọi payload
- Quản lý thiết bị: Rename, Revoke, Block

### Sync Logic
- **Desktop → Mobile:** Broadcast khi copy
- **Mobile → Desktop:** Nhận push từ mobile
- **Anti-loop:** Chống lặp vô hạn
- **Modes:** Auto, Hotkey-only, Receive-only, Paused

### Clipboard History
- Lưu SQLite locally
- Pin, Delete, Clear All
- Retention policy (max items, TTL)

### Device Discovery
- mDNS (Bonjour/Avahi) với service `_copypaws._tcp`
- TXT records: `server_id`, `version`

### System Tray
- Icon trong system tray
- Menu: Show, Pause Sync, Quit
- Minimize to tray khi đóng window

### Global Shortcuts
| Shortcut | Chức năng |
|----------|-----------|
| `Ctrl+Alt+C` | Push clipboard |
| `Ctrl+Alt+V` | Paste latest |
| `Super+Alt+V` | Toggle history |

---

## Tech Stack

- **Framework:** Tauri (Rust + React)
- **Database:** SQLite
- **Communication:** WebSocket (Tungstenite)
- **Discovery:** mDNS (mdns-sd)

---

## Giới hạn v1

- **Data Type:** Chỉ Plain Text / UTF-8
- **Payload Cap:** 2MB max
- **Mobile Background:** Best-effort (phụ thuộc OS restrictions)

---

## Roadmap v2+

- Image/File clipboard support
- Cloud relay (sync khác WiFi)
- Auto-pair improvements
