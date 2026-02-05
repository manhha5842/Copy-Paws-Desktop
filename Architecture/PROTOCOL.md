# CopyPaws WebSocket Protocol Reference

## Overview

CopyPaws sử dụng WebSocket để giao tiếp giữa Desktop Hub và Mobile clients. Tất cả payload được mã hóa AES-256-GCM.

**Server Details:**
- Protocol: WebSocket (`ws://`)
- Default Port: `8765`
- Encryption: AES-256-GCM
- Message Format: JSON

---

## Connection Flow

### Initial Pairing (First Time)

```
Mobile App                          Desktop Hub
    |                                     |
    |------ WebSocket Connect ----------->|
    |                                     |
    |------ PAIRING_REQUEST ------------->|
    |                                     |
    |<----- PAIRING_RESPONSE -------------|
    |     (includes encryption_key)       |
    |                                     |
```

### Reconnection (Subsequent)

```
Mobile App                          Desktop Hub
    |                                     |
    |------ WebSocket Connect ----------->|
    |                                     |
    |------ HANDSHAKE ------------------->|
    |     (with stored device_id)         |
    |                                     |
    |<----- HANDSHAKE_RESPONSE -----------|
    |                                     |
```

### Heartbeat

```
Desktop Hub                         Mobile App
    |                                     |
    |------------- PING ----------------->| (every 30s)
    |                                     |
    |<------------ PONG ------------------| (respond immediately)
    |                                     |
```

- Server sends `PING` every **30 seconds**
- Client must respond with `PONG` within **90 seconds**
- Failure causes automatic disconnection

---

## Message Types

### Client → Server

#### PAIRING_REQUEST
```json
{
  "type": "PAIRING_REQUEST",
  "device_id": "mobile-uuid-v4",
  "device_name": "John's iPhone",
  "platform": "iOS",
  "pairing_token": "token-from-qr-code"
}
```

#### HANDSHAKE
```json
{
  "type": "HANDSHAKE",
  "device_id": "mobile-uuid-v4"
}
```

#### CLIP_PUSH
```json
{
  "type": "CLIP_PUSH",
  "payload_encrypted": "base64-encrypted-content",
  "iv": "base64-12-byte-iv",
  "device_info": {
    "name": "John's iPhone",
    "battery": "75%"
  }
}
```

#### GET_LATEST
```json
{
  "type": "GET_LATEST"
}
```

#### PONG
```json
{
  "type": "PONG"
}
```

---

### Server → Client

#### PAIRING_RESPONSE
```json
{
  "type": "PAIRING_RESPONSE",
  "success": true,
  "message": "Pairing successful",
  "encryption_key": "base64-encoded-32-byte-key"
}
```

#### HANDSHAKE_RESPONSE
```json
{
  "type": "HANDSHAKE_RESPONSE",
  "success": true
}
```

#### ENCRYPTED (Wrapper)
```json
{
  "type": "ENCRYPTED",
  "payload": "base64-encrypted-json",
  "iv": "base64-12-byte-iv"
}
```

After decryption, inner message:
```json
{
  "type": "CLIP_BROADCAST",
  "clip_id": "uuid-of-clip",
  "payload_encrypted": "plaintext-content",
  "source_app": "Chrome",
  "timestamp": 1234567890
}
```

#### PING
```json
{
  "type": "PING"
}
```

---

## QR Code Format

```
copypaws://pair?ip=192.168.1.100&port=8765&token=uuid-token&secret=base64-key&name=Desktop%20Hub&id=server-uuid
```

| Parameter | Description |
|-----------|-------------|
| `ip` | Server IP address |
| `port` | WebSocket port (8765) |
| `token` | Pairing token (5 min expiry) |
| `secret` | Base64-encoded encryption key |
| `name` | Server name |
| `id` | Server UUID |

---

## Error Codes

| Error | Meaning |
|-------|---------|
| `DEVICE_NOT_FOUND` | Device không tồn tại trong database |
| `DEVICE_BLOCKED` | Device đã bị block |
| `DEVICE_REVOKED` | Device đã bị revoke |
| `INVALID_TOKEN` | Token không hợp lệ |
| `TOKEN_EXPIRED` | Token đã hết hạn |
