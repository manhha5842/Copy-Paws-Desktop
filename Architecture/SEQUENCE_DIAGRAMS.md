# CopyPaws Sequence Diagrams

## 1. Initial Pairing Flow

```
┌─────────┐          ┌─────────────┐          ┌──────────┐
│  User   │          │   Mobile    │          │  Desktop │
└────┬────┘          └──────┬──────┘          └────┬─────┘
     │                      │                      │
     │  Click "Pair Device" │                      │
     │─────────────────────>│                      │
     │                      │                      │
     │                      │   Generate QR Code   │
     │                      │<─────────────────────│
     │                      │                      │
     │    Scan QR Code      │                      │
     │─────────────────────>│                      │
     │                      │                      │
     │                      │   PAIRING_REQUEST    │
     │                      │─────────────────────>│
     │                      │                      │
     │                      │  PAIRING_RESPONSE    │
     │                      │<─────────────────────│
     │                      │   (encryption_key)   │
     │                      │                      │
     │    Store Credentials │                      │
     │<─────────────────────│                      │
     │                      │                      │
     │                      │     HANDSHAKE        │
     │                      │─────────────────────>│
     │                      │                      │
     │                      │ HANDSHAKE_RESPONSE   │
     │                      │<─────────────────────│
     │                      │                      │
     │   ✓ Connected        │                      │
     │<─────────────────────│                      │
```

---

## 2. Auto-Connect Flow

```
┌──────────┐          ┌─────────────┐          ┌──────────┐
│  Mobile  │          │   mDNS      │          │  Desktop │
└────┬─────┘          └──────┬──────┘          └────┬─────┘
     │                       │                      │
     │   App Start/Resume    │                      │
     │                       │                      │
     │  Start Discovery      │                      │
     │──────────────────────>│                      │
     │                       │                      │
     │                       │  Advertise Service   │
     │                       │<─────────────────────│
     │                       │  _copypaws._tcp      │
     │                       │  TXT: server_id      │
     │                       │                      │
     │  Service Found        │                      │
     │<──────────────────────│                      │
     │                       │                      │
     │  Match server_id?     │                      │
     │  ✓ Yes                │                      │
     │                       │                      │
     │         WebSocket Connect                    │
     │─────────────────────────────────────────────>│
     │                                              │
     │              HANDSHAKE                       │
     │─────────────────────────────────────────────>│
     │                                              │
     │          HANDSHAKE_RESPONSE                  │
     │<─────────────────────────────────────────────│
     │                                              │
     │           ✓ Connected                        │
```

---

## 3. Clipboard Sync Flow

```
┌──────────┐                              ┌──────────┐
│  Desktop │                              │  Mobile  │
└────┬─────┘                              └────┬─────┘
     │                                         │
     │  User copies text                       │
     │                                         │
     │  Clipboard Monitor triggers             │
     │                                         │
     │  Check anti-loop (hash)                 │
     │  ✓ New content                          │
     │                                         │
     │  Save to database                       │
     │                                         │
     │  Encrypt with device key                │
     │                                         │
     │         ENCRYPTED                       │
     │  ┌──────────────────────────────────┐   │
     │  │ payload: CLIP_BROADCAST (enc)    │   │
     │  │ iv: random_12_bytes              │──>│
     │  └──────────────────────────────────┘   │
     │                                         │
     │                            Decrypt      │
     │                                         │
     │                     Display in app      │
     │                                         │
     │                     User taps "Copy"    │
     │                                         │
     │                     Set clipboard       │
```

---

## 4. Heartbeat Flow

```
┌──────────┐                              ┌──────────┐
│  Desktop │                              │  Mobile  │
└────┬─────┘                              └────┬─────┘
     │                                         │
     │  Every 30 seconds                       │
     │                                         │
     │              PING                       │
     │────────────────────────────────────────>│
     │                                         │
     │                           Receive PING  │
     │                                         │
     │              PONG                       │
     │<────────────────────────────────────────│
     │                                         │
     │  Update last_pong_time                  │
     │                                         │
     │                                         │
     │  ... 90 seconds without PONG ...        │
     │                                         │
     │  Close connection                       │
     │                                         │
     │  Remove from active clients             │
```

---

## 5. System Tray Behavior

```
┌─────────┐          ┌─────────────┐          ┌───────────┐
│  User   │          │    App      │          │   Tray    │
└────┬────┘          └──────┬──────┘          └─────┬─────┘
     │                      │                       │
     │  Click X (close)     │                       │
     │─────────────────────>│                       │
     │                      │                       │
     │                      │  Hide window          │
     │                      │  (not exit)           │
     │                      │                       │
     │                      │  App still running    │
     │                      │──────────────────────>│
     │                                              │
     │  Right-click tray                            │
     │─────────────────────────────────────────────>│
     │                                              │
     │                      ┌───────────────┐       │
     │                      │ Show CopyPaws │       │
     │                      │ Pause Sync    │       │
     │                      │ Quit          │       │
     │                      └───────────────┘       │
     │                                              │
     │  Click "Quit"                                │
     │─────────────────────────────────────────────>│
     │                      │                       │
     │                      │  app.exit(0)          │
     │<─────────────────────│                       │
```
