# CopyPaws Data Models

## Database Schema (SQLite)

### Table: `devices`

| Field | Type | Description |
|-------|------|-------------|
| device_id | STRING (PK) | UUID của thiết bị |
| name | STRING | Tên hiển thị |
| shared_secret | STRING | Key mã hóa riêng |
| platform | STRING | iOS / Android |
| last_seen | DATETIME | Thời điểm online cuối |
| is_blocked | BOOLEAN | Đã bị chặn? |

### Table: `clips`

| Field | Type | Description |
|-------|------|-------------|
| id | STRING (PK) | UUID |
| content | TEXT | Nội dung clipboard |
| content_hash | STRING | SHA256 hash (anti-loop) |
| source_device | STRING | Device ID hoặc 'LOCAL' |
| source_app | STRING | App nguồn (Chrome, VSCode...) |
| created_at | DATETIME | Timestamp |
| is_pinned | BOOLEAN | Đã ghim? |

### Table: `settings`

| Field | Type | Description |
|-------|------|-------------|
| key | STRING (PK) | Setting name |
| value | STRING | Setting value (JSON) |

---

## Message Types (Rust Enum)

```rust
pub enum WsMessage {
    PairingRequest {
        device_id: String,
        device_name: String,
        platform: String,
        pairing_token: String,
    },
    PairingResponse {
        success: bool,
        message: String,
        encryption_key: Option<String>,
    },
    Handshake {
        device_id: String,
    },
    HandshakeResponse {
        success: bool,
        error: Option<String>,
    },
    ClipPush {
        payload_encrypted: String,
        iv: String,
        device_info: DeviceInfo,
    },
    ClipBroadcast {
        clip_id: String,
        payload_encrypted: String,
        iv: String,
        source_app: Option<String>,
        timestamp: u64,
    },
    Encrypted {
        payload: String,
        iv: String,
    },
    GetLatest,
    Ping,
    Pong,
}
```

---

## TypeScript Interfaces

```typescript
interface ServerStatus {
  status: string;
  ip_address: string;
  port: number;
}

interface Clip {
  id: string;
  content: string;
  content_hash: string;
  source_device: string | null;
  source_app: string | null;
  created_at: string;
  is_pinned: boolean;
}

interface Device {
  device_id: string;
  name: string;
  platform: string;
  last_seen: string | null;
  is_blocked: boolean;
}

interface SyncStatus {
  is_active: boolean;
  sync_mode: string;
  connected_devices: number;
  ip: string;
  port: number;
}
```

---

## Flutter Models

```dart
class HubInfo {
  final String id;
  final String name;
  final String endpoint;
  final String? ip;
  final int? port;
  final DateTime? lastConnected;
  final bool isPaired;
}

class ClipItem {
  final String id;
  final String content;
  final String? sourceApp;
  final DateTime timestamp;
  final bool isPinned;
}
```
