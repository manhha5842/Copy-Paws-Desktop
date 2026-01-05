# CopyPaws Desktop

This folder contains all desktop-related components of the CopyPaws ecosystem.

## Components

| Folder | Description | Status |
|--------|-------------|--------|
| [clipboard-hub](./clipboard-hub/) | Main CopyPaws Desktop application (Tauri) | Active Development |
| [test-client](./test-client/) | Web-based WebSocket testing tool | Ready for Testing |

## Overview

CopyPaws Desktop is the **hub server** in the CopyPaws ecosystem. It:

- Monitors your desktop clipboard
- Runs a WebSocket server for mobile connections
- Manages device pairing and encryption
- Stores clipboard history locally

## Quick Start

```bash
# Navigate to the main app
cd clipboard-hub

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## Architecture

```
Desktop Folder
├── clipboard-hub/      # Main Tauri application
│   ├── src/           # React frontend
│   └── src-tauri/     # Rust backend
│
└── test-client/       # Testing tool
    ├── index.html     # Web interface
    ├── styles.css     # Styling
    └── script.js      # WebSocket client
```

## Development Status

### Completed
- Project initialization with Tauri
- Database layer (SQLite)
- Encryption module (AES-256-GCM)
- WebSocket server structure
- Clipboard monitoring
- QR code pairing
- Sync manager
- mDNS service discovery
- React dashboard UI

### In Progress
- WebSocket server activation on startup
- End-to-end clipboard sync
- System tray integration

### Planned
- Auto-start on boot
- Keyboard shortcuts
- Multiple clipboard format support
- Cloud relay (v2)

## Testing

Use the `test-client` folder to test WebSocket communication:

1. Run CopyPaws Desktop in dev mode
2. Open `test-client/index.html` in a browser
3. Connect to `localhost:8765`
4. Test clipboard sync functionality

## Related

- **Mobile Apps**: See `../Mobile-iOS/` and `../Mobile-Android/` (coming soon)
- **Documentation**: See `Desktop.md` for full specifications

---

Part of the **CopyPaws** project.
