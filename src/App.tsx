import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Home,
  History,
  Smartphone,
  Settings,
  Clipboard,
  Pin,
  PinOff,
  Trash2,
  Plus,
  X,
  Wifi,
  WifiOff,
  Clock,
  Monitor,
  Copy,
  Pause,
  Play,
} from "lucide-react";
import "./App.css";

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

interface PairingData {
  qr_svg: string;
  ip: string;
  port: number;
  token: string;
  expires_at: number;
}

interface SyncStatus {
  is_active: boolean;
  sync_mode: string;
  connected_devices: number;
  ip: string;
  port: number;
}

function App() {
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);
  const [clips, setClips] = useState<Clip[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [showPairing, setShowPairing] = useState(false);
  const [pairingData, setPairingData] = useState<PairingData | null>(null);
  const [activeTab, setActiveTab] = useState<"dashboard" | "history" | "devices" | "settings">("dashboard");
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  // Check if we're running in Tauri context
  const isTauri = () => {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  };

  async function loadData() {
    if (!isTauri()) {
      console.log("Not running in Tauri context - skipping data load");
      return;
    }
    
    try {
      const status = await invoke<ServerStatus>("get_server_status");
      setServerStatus(status);
      
      const clipsList = await invoke<Clip[]>("get_clips", { limit: 50 });
      setClips(clipsList);
      
      const devicesList = await invoke<Device[]>("get_devices");
      setDevices(devicesList);
    } catch (error) {
      console.error("Failed to load data:", error);
    }
    
    // Load sync status
    try {
      const status = await invoke<SyncStatus>("get_sync_status");
      setSyncStatus(status);
    } catch (error) {
      console.error("Failed to load sync status:", error);
    }
  }

  async function generatePairingQR() {
    try {
      const data = await invoke<PairingData>("get_pairing_data");
      setPairingData(data);
      setShowPairing(true);
    } catch (error) {
      console.error("Failed to generate pairing QR:", error);
    }
  }

  async function deleteClip(clipId: string) {
    try {
      await invoke("delete_clip", { clipId });
      loadData();
    } catch (error) {
      console.error("Failed to delete clip:", error);
    }
  }

  async function pinClip(clipId: string, pinned: boolean) {
    try {
      await invoke("pin_clip", { clipId, pinned });
      loadData();
    } catch (error) {
      console.error("Failed to pin clip:", error);
    }
  }

  async function revokeDevice(deviceId: string) {
    if (!confirm("Are you sure you want to revoke this device? It will need to re-pair to connect again.")) {
      return;
    }
    try {
      await invoke("revoke_device", { deviceId });
      loadData();
    } catch (error) {
      console.error("Failed to revoke device:", error);
    }
  }

  async function blockDevice(deviceId: string, blocked: boolean) {
    try {
      await invoke("block_device", { deviceId, blocked });
      loadData();
    } catch (error) {
      console.error("Failed to block/unblock device:", error);
    }
  }

  async function copyToClipboard(content: string) {
    try {
      await invoke("copy_to_clipboard", { content });
      // Show a brief visual feedback (could add a toast notification)
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
      // Fallback to browser clipboard API
      try {
        await navigator.clipboard.writeText(content);
      } catch (e) {
        console.error("Fallback clipboard failed:", e);
      }
    }
  }

  async function toggleSync() {
    if (!isTauri()) return;
    try {
      const isActive = await invoke<boolean>("toggle_sync");
      // Reload sync status
      const status = await invoke<SyncStatus>("get_sync_status");
      setSyncStatus(status);
      console.log("Sync toggled:", isActive ? "Active" : "Paused");
    } catch (error) {
      console.error("Failed to toggle sync:", error);
    }
  }

  // Test functions for development
  async function addTestClip() {
    if (!isTauri()) return;
    try {
      const testContent = `Test clip at ${new Date().toLocaleTimeString()}`;
      await invoke("add_test_clip", { content: testContent });
      loadData(); // Refresh data
    } catch (error) {
      console.error("Failed to add test clip:", error);
    }
  }

  async function addTestDevice() {
    if (!isTauri()) return;
    try {
      await invoke("add_test_device", { name: "Test Device" });
      loadData(); // Refresh data
    } catch (error) {
      console.error("Failed to add test device:", error);
    }
  }


  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const truncateContent = (content: string, maxLength: number = 100) => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + "...";
  };

  return (
    <div className="app">
      {/* Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="logo">
            <Clipboard size={24} />
            <span>CopyPaws</span>
          </div>
        </div>
        <nav className="nav">
          <button 
            className={`nav-item ${activeTab === "dashboard" ? "active" : ""}`}
            onClick={() => setActiveTab("dashboard")}
          >
            <Home size={20} />
            <span>Dashboard</span>
          </button>
          <button 
            className={`nav-item ${activeTab === "history" ? "active" : ""}`}
            onClick={() => setActiveTab("history")}
          >
            <History size={20} />
            <span>History</span>
          </button>
          <button 
            className={`nav-item ${activeTab === "devices" ? "active" : ""}`}
            onClick={() => setActiveTab("devices")}
          >
            <Smartphone size={20} />
            <span>Devices</span>
          </button>
          <button 
            className={`nav-item ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
          >
            <Settings size={20} />
            <span>Settings</span>
          </button>
        </nav>
        
        <div className="sidebar-footer">
          <div className="connection-status">
            {serverStatus?.status === "Running" ? (
              <>
                <Wifi size={16} className="status-icon online" />
                <span>Connected</span>
              </>
            ) : (
              <>
                <WifiOff size={16} className="status-icon offline" />
                <span>Disconnected</span>
              </>
            )}
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="main-content">
        {activeTab === "dashboard" && (
          <div className="dashboard">
            <div className="page-header">
              <h2>Dashboard</h2>
              <p className="subtitle">Overview of your clipboard hub</p>
            </div>
            
            {/* Status Card */}
            <div className="status-card">
              <div className="status-header">
                <div className="status-indicator">
                  <span className={`status-dot ${serverStatus?.status === "Running" ? "online" : "offline"}`}></span>
                  <span className="status-text">Server {serverStatus?.status || "Unknown"}</span>
                </div>
                <button 
                  className={`sync-toggle-btn ${syncStatus?.is_active ? 'active' : 'paused'}`}
                  onClick={toggleSync}
                  title={syncStatus?.is_active ? "Click to pause sync" : "Click to resume sync"}
                >
                  {syncStatus?.is_active ? (
                    <>
                      <Pause size={16} />
                      <span>Syncing</span>
                    </>
                  ) : (
                    <>
                      <Play size={16} />
                      <span>Paused</span>
                    </>
                  )}
                </button>
              </div>
              <div className="status-info">
                <div className="info-item">
                  <span className="info-label">IP Address</span>
                  <span className="info-value">{serverStatus?.ip_address || "-"}</span>
                </div>
                <div className="info-item">
                  <span className="info-label">Port</span>
                  <span className="info-value">{serverStatus?.port || "-"}</span>
                </div>
                <div className="info-item">
                  <span className="info-label">Connected</span>
                  <span className="info-value">{syncStatus?.connected_devices || 0} device(s)</span>
                </div>
                <div className="info-item">
                  <span className="info-label">Mode</span>
                  <span className="info-value">{syncStatus?.sync_mode || "Auto"}</span>
                </div>
              </div>
            </div>

            {/* Quick Stats */}
            <div className="stats-grid">
              <div className="stat-card">
                <div className="stat-icon-wrapper">
                  <Clipboard size={24} />
                </div>
                <div className="stat-content">
                  <h3>{clips.length}</h3>
                  <p>Clips in History</p>
                </div>
              </div>
              <div className="stat-card">
                <div className="stat-icon-wrapper">
                  <Smartphone size={24} />
                </div>
                <div className="stat-content">
                  <h3>{devices.length}</h3>
                  <p>Connected Devices</p>
                </div>
              </div>
              <div className="stat-card">
                <div className="stat-icon-wrapper">
                  <Pin size={24} />
                </div>
                <div className="stat-content">
                  <h3>{clips.filter(c => c.is_pinned).length}</h3>
                  <p>Pinned Clips</p>
                </div>
              </div>
            </div>

            {/* Recent Clip */}
            {clips.length > 0 && (
              <div className="recent-clip-card">
                <div className="card-header">
                  <Clipboard size={18} />
                  <h3>Most Recent Clip</h3>
                </div>
                <p className="clip-content">{truncateContent(clips[0].content, 200)}</p>
                <div className="clip-meta">
                  <Clock size={14} />
                  <span>{formatDate(clips[0].created_at)}</span>
                  {clips[0].source_app && (
                    <>
                      <Monitor size={14} />
                      <span>{clips[0].source_app}</span>
                    </>
                  )}
                </div>
              </div>
            )}

            {/* Test Buttons for Development */}
            <div className="button-group" style={{ marginBottom: '16px' }}>
              <button className="secondary-button" onClick={addTestClip}>
                Add Test Clip
              </button>
              <button className="secondary-button" onClick={addTestDevice}>
                Add Test Device
              </button>
            </div>

            {/* Connect Device Button */}
            <button className="primary-button" onClick={generatePairingQR}>
              <Plus size={20} />
              Connect New Device
            </button>
          </div>
        )}

        {activeTab === "history" && (
          <div className="history">
            <div className="page-header">
              <h2>Clipboard History</h2>
              <p className="subtitle">{clips.length} clips stored</p>
            </div>
            <div className="clips-list">
              {clips.length === 0 ? (
                <div className="empty-state">
                  <Clipboard size={48} />
                  <p>No clips yet. Copy something to get started!</p>
                </div>
              ) : (
                clips.map((clip) => (
                  <div key={clip.id} className={`clip-item ${clip.is_pinned ? "pinned" : ""}`}>
                    <div className="clip-content-wrapper">
                      <p className="clip-text">{truncateContent(clip.content)}</p>
                      <div className="clip-meta">
                        <Clock size={12} />
                        <span>{formatDate(clip.created_at)}</span>
                        {clip.source_app && (
                          <>
                            <Monitor size={12} />
                            <span>{clip.source_app}</span>
                          </>
                        )}
                      </div>
                    </div>
                    <div className="clip-actions">
                      <button 
                        className="icon-button"
                        onClick={() => copyToClipboard(clip.content)}
                        title="Copy to clipboard"
                      >
                        <Copy size={18} />
                      </button>
                      <button 
                        className="icon-button"
                        onClick={() => pinClip(clip.id, !clip.is_pinned)}
                        title={clip.is_pinned ? "Unpin" : "Pin"}
                      >
                        {clip.is_pinned ? <PinOff size={18} /> : <Pin size={18} />}
                      </button>
                      <button 
                        className="icon-button danger"
                        onClick={() => deleteClip(clip.id)}
                        title="Delete"
                      >
                        <Trash2 size={18} />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {activeTab === "devices" && (
          <div className="devices">
            <div className="page-header">
              <h2>Connected Devices</h2>
              <p className="subtitle">{devices.length} devices paired</p>
            </div>
            <button className="primary-button" onClick={generatePairingQR}>
              <Plus size={20} />
              Connect New Device
            </button>
            <div className="devices-list">
              {devices.length === 0 ? (
                <div className="empty-state">
                  <Smartphone size={48} />
                  <p>No devices connected. Scan QR code from your mobile app!</p>
                </div>
              ) : (
                devices.map((device) => (
                  <div key={device.device_id} className="device-item">
                    <div className="device-info">
                      <div className="device-icon-wrapper">
                        <Smartphone size={24} />
                      </div>
                      <div className="device-details">
                        <h4>{device.name}</h4>
                        <p>
                          {device.platform}
                          {device.last_seen && (
                            <>
                              <span className="separator">•</span>
                              Last seen: {formatDate(device.last_seen)}
                            </>
                          )}
                        </p>
                      </div>
                    </div>
                    <div className="device-actions">
                      <button 
                        className={`block-button ${device.is_blocked ? "blocked" : ""}`}
                        onClick={() => blockDevice(device.device_id, !device.is_blocked)}
                      >
                        {device.is_blocked ? "Unblock" : "Block"}
                      </button>
                      
                      <button 
                        className="danger-button"
                        onClick={() => revokeDevice(device.device_id)}
                      >
                        Revoke Access
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {activeTab === "settings" && (
          <div className="settings">
            <div className="page-header">
              <h2>Settings</h2>
              <p className="subtitle">Configure your clipboard hub</p>
            </div>
            
            <div className="settings-section">
              <div className="section-header">
                <Wifi size={20} />
                <h3>Network Configuration</h3>
              </div>
              
              <div className="network-info-grid">
                <div className="network-info-item">
                  <div className="network-info-label">Type</div>
                  <div className="network-info-value">Local LAN</div>
                </div>
                <div className="network-info-item">
                  <div className="network-info-label">Status</div>
                  <div className="network-info-value" style={{ color: serverStatus?.status === "Running" ? "var(--success)" : "var(--danger)" }}>
                    {serverStatus?.status || "Unknown"}
                  </div>
                </div>
                <div className="network-info-item">
                  <div className="network-info-label">IP Address</div>
                  <div className="network-info-value">{serverStatus?.ip_address || "0.0.0.0"}</div>
                </div>
                <div className="network-info-item">
                  <div className="network-info-label">Port</div>
                  <div className="network-info-value">{serverStatus?.port || 8765}</div>
                </div>
                <div className="network-info-item" style={{ gridColumn: "1 / -1" }}>
                  <div className="network-info-label">Service Discovery</div>
                  <div className="network-info-value" style={{ fontSize: "0.95rem" }}>
                    mDNS / Bonjour Advertising Active
                  </div>
                  <div style={{ fontSize: "0.85rem", color: "var(--text-muted)", marginTop: "4px" }}>
                    Devices can auto-discover this Hub as "CopyPaws Desktop"
                  </div>
                </div>
              </div>
            </div>

            <div className="settings-section">
              <div className="section-header">
                <Clipboard size={20} />
                <h3>Sync Settings</h3>
              </div>
              <div className="setting-item">
                <div className="setting-info">
                  <label>Sync Mode</label>
                  <span className="setting-description">How clipboard is synchronized</span>
                </div>
                <select 
                  value={syncStatus?.sync_mode || "Auto"}
                  onChange={async (e) => {
                    try {
                      await invoke("set_sync_mode", { mode: e.target.value });
                      const status = await invoke<SyncStatus>("get_sync_status");
                      setSyncStatus(status);
                    } catch (error) {
                      console.error("Failed to change sync mode:", error);
                    }
                  }}
                >
                  <option value="Auto">Auto Sync (Recommended)</option>
                  <option value="HotkeyOnly">Hotkey Only</option>
                  <option value="ReceiveOnly">Receive Only</option>
                  <option value="Paused">Paused</option>
                </select>
              </div>
              
              {/* Sync Mode Description */}
              <div className="sync-mode-info">
                {syncStatus?.sync_mode === "Auto" && (
                  <p className="mode-description">Clipboard changes are automatically synced to all connected devices in real-time.</p>
                )}
                {syncStatus?.sync_mode === "HotkeyOnly" && (
                  <p className="mode-description">Clipboard is only synced when you press the designated hotkey.</p>
                )}
                {syncStatus?.sync_mode === "ReceiveOnly" && (
                  <p className="mode-description">This device will receive clips from other devices, but won't send any.</p>
                )}
                {syncStatus?.sync_mode === "Paused" && (
                  <p className="mode-description">Clipboard synchronization is completely paused. No data is sent or received.</p>
                )}
              </div>
            </div>

            <div className="settings-section">
              <div className="section-header">
                <History size={20} />
                <h3>History Settings</h3>
              </div>
              <div className="setting-item">
                <div className="setting-info">
                  <label>Max Items</label>
                  <span className="setting-description">Maximum clips to store</span>
                </div>
                <input type="number" defaultValue={1000} />
              </div>
              <div className="setting-item">
                <div className="setting-info">
                  <label>Retention Period</label>
                  <span className="setting-description">Days to keep clips</span>
                </div>
                <input type="number" defaultValue={30} />
              </div>
            </div>
          </div>
        )}
      </main>

      {/* Pairing Modal */}
      {showPairing && pairingData && (
        <div className="modal-overlay" onClick={() => setShowPairing(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <button className="modal-close" onClick={() => setShowPairing(false)}>
              <X size={24} />
            </button>
            <div className="modal-header">
              <Smartphone size={32} />
              <h2>Connect Mobile Device</h2>
            </div>
            <p className="modal-description">Scan this QR code with your mobile app to pair</p>
            <div 
              className="qr-code"
              dangerouslySetInnerHTML={{ __html: pairingData.qr_svg }}
            />
            <div className="pairing-info">
              <div className="pairing-info-item">
                <span className="label">IP Address</span>
                <span className="value">{pairingData.ip}</span>
              </div>
              <div className="pairing-info-item">
                <span className="label">Port</span>
                <span className="value">{pairingData.port}</span>
              </div>
              <div className="pairing-info-item">
                <span className="label">Token</span>
                <span className="value" style={{userSelect: "all", cursor: "pointer"}} title="Click to select">{pairingData.token}</span>
              </div>
            </div>
            <button className="secondary-button" onClick={() => setShowPairing(false)}>
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
