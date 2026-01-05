# CopyPaws Desktop

**Seamless clipboard synchronization across your devices**

CopyPaws Desktop is the hub application that syncs clipboard content between your desktop (Windows/macOS/Linux) and mobile devices over your local network.

## Features

- **Real-time Sync**: Automatically sync clipboard when you copy text
- **Secure**: End-to-end encryption with AES-256-GCM
- **Local Network**: Works entirely on your LAN - no cloud required
- **Device Management**: Pair, rename, and revoke device access
- **Clipboard History**: Keep track of your recent copies
- **System Tray**: Runs quietly in the background
- **Cross-Platform**: Windows, macOS, and Linux support

## Screenshots

*Coming soon*

## Installation

### Pre-built Binaries

Download the latest release for your platform:
- **Windows**: `CopyPaws_x64.msi` or `CopyPaws_x64.exe`
- **macOS**: `CopyPaws.dmg`
- **Linux**: `CopyPaws.AppImage` or `CopyPaws.deb`

### Build from Source

**Prerequisites:**
- Node.js 18+
- Rust 1.70+
- System dependencies for Tauri (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/))

```bash
# Clone the repository
git clone https://github.com/yourusername/copypaws-desktop.git
cd copypaws-desktop/clipboard-hub

# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

## Quick Start

1. **Launch CopyPaws** on your desktop
2. **Generate QR Code** - Click "Connect New Device" in the Devices tab
3. **Scan with Mobile** - Use CopyPaws mobile app to scan the QR code
4. **Start Syncing** - Copy text on any device, it appears everywhere!

## How It Works

```
┌─────────────────┐     WebSocket     ┌─────────────────┐
│  CopyPaws       │◄────────────────►│  CopyPaws       │
│  Desktop        │   (Encrypted)     │  Mobile         │
│  (Hub Server)   │                   │  (Client)       │
└─────────────────┘                   └─────────────────┘
        │
        ▼
   ┌─────────┐
   │ SQLite  │  Local clipboard history
   └─────────┘
```

1. **Desktop monitors clipboard** - Detects when you copy text
2. **Encrypts and broadcasts** - Sends encrypted content to connected devices
3. **Mobile receives** - Decrypts and sets clipboard
4. **Anti-loop protection** - Prevents infinite sync cycles

## Tech Stack

| Component | Technology |
|-----------|------------|
| Framework | [Tauri](https://tauri.app/) (Rust + React) |
| Frontend | React + TypeScript |
| Backend | Rust |
| Database | SQLite |
| Network | WebSocket |
| Encryption | AES-256-GCM |
| Discovery | mDNS (optional) |

## Project Structure

```
clipboard-hub/
├── src/                    # React frontend
│   ├── App.tsx            # Main application
│   ├── App.css            # Styles
│   └── main.tsx           # Entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Main entry, Tauri commands
│   │   ├── database/      # SQLite operations
│   │   ├── crypto.rs      # AES encryption
│   │   ├── websocket.rs   # WebSocket server
│   │   ├── clipboard.rs   # Clipboard monitoring
│   │   ├── pairing.rs     # QR code generation
│   │   ├── sync_manager.rs# Sync orchestration
│   │   └── mdns.rs        # Service discovery
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # App configuration
└── package.json           # Node dependencies
```

## Configuration

Settings are stored in:
- **Windows**: `%APPDATA%\CopyPaws\`
- **macOS**: `~/Library/Application Support/CopyPaws/`
- **Linux**: `~/.local/share/copypaws/`

### Available Settings

| Setting | Default | Description |
|---------|---------|-------------|
| Server Port | 8765 | WebSocket server port |
| Sync Mode | Auto | Auto, HotkeyOnly, ReceiveOnly, Paused |
| Max History | 1000 | Maximum clips to store |
| Retention Days | 30 | Days to keep clips |

## Security

- **Local Network Only**: No data leaves your network
- **End-to-End Encryption**: AES-256-GCM with unique per-device keys
- **Secure Pairing**: Time-limited tokens (5 minutes)
- **Device Revocation**: Instantly revoke any device access

## Development

```bash
# Run development server
npm run tauri dev

# Run Rust tests
cd src-tauri && cargo test

# Build release
npm run tauri build
```

## Roadmap

- [x] Phase 1: Project Foundation
- [x] Phase 2: Core Infrastructure
- [x] Phase 3: Security & Pairing
- [ ] Phase 4: Protocol Implementation
- [ ] Phase 5: mDNS Service Discovery
- [ ] Phase 6: System Tray Integration
- [ ] Phase 7: Complete Frontend UI
- [ ] Phase 8: Testing
- [ ] Phase 9: Build & Distribution
- [ ] Phase 10: Documentation

## Related Projects

- [CopyPaws Mobile (iOS)](../Mobile-iOS/) - *Coming soon*
- [CopyPaws Mobile (Android)](../Mobile-Android/) - *Coming soon*
- [Test Client](../test-client/) - WebSocket testing tool

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/copypaws-desktop/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/copypaws-desktop/discussions)

---

**CopyPaws** - Copy once, paste everywhere.
