import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { formatDateTime } from "./utils/date";

interface Clip {
  id: string;
  content: string;
  created_at: string;
  source_device?: string;
  is_pinned: boolean;
}

function HistoryWindow() {
  const [clips, setClips] = useState<Clip[]>([]);

  useEffect(() => {
    loadClips();
    
    // Focus listener to refresh when window is shown
    const unlisten = getCurrentWindow().listen("tauri://focus", () => {
      loadClips();
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // Close window when losing focus
  useEffect(() => {
      const unlisten = getCurrentWindow().listen("tauri://blur", () => {
          getCurrentWindow().hide();
      });
      return () => {
          unlisten.then(f => f());
      };
  }, []);

  const loadClips = async () => {
    try {
      const result = await invoke<Clip[]>("get_clips", { limit: 20 });
      setClips(result);
    } catch (error) {
      console.error("Failed to load clips:", error);
    }
  };

  const handleSelectClip = async (content: string) => {
    try {
      await invoke("copy_to_clipboard", { content });
      getCurrentWindow().hide(); 
      // Note: Backend might simulate paste if configured, but here we just copy and hide
    } catch (error) {
      console.error("Failed to select clip:", error);
    }
  };

  return (
    <div className="container history-container">
      <div className="history-header">
        <h3>Clipboard History</h3>
      </div>
      <div className="history-list">
        {clips.length === 0 ? (
            <div className="empty-state">No history</div>
        ) : (
            clips.map((clip) => (
            <div
                key={clip.id}
                className="history-item"
                onClick={() => handleSelectClip(clip.content)}
            >
                <div className="history-content">{clip.content}</div>
                <div className="history-meta">
                  <span>{formatDateTime(clip.created_at)}</span>
                  {clip.source_device ? <span>From: {clip.source_device}</span> : null}
                </div>
            </div>
            ))
        )}
      </div>
      <style>{`
        .history-container {
            padding: 0;
            height: 100vh;
            background-color: var(--bg-color);
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }
        .history-header {
            padding: 10px 15px;
            background-color: var(--surface-color);
            border-bottom: 1px solid var(--border-color);
            -webkit-app-region: drag;
        }
        .history-header h3 {
            margin: 0;
            font-size: 1rem;
        }
        .history-list {
            flex: 1;
            overflow-y: auto;
            padding: 8px;
        }
        .history-item {
            padding: 10px;
            border-bottom: 1px solid var(--border-color);
            cursor: pointer;
            transition: background 0.2s;
            border-radius: 6px;
            margin-bottom: 4px;
        }
        .history-item:hover {
            background-color: var(--surface-color);
        }
        .history-content {
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            font-size: 0.9rem;
            margin-bottom: 4px;
        }
        .history-meta {
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
            font-size: 0.75rem;
            color: #888;
        }
        .empty-state {
            padding: 20px;
            text-align: center;
            color: #888;
        }
      `}</style>
    </div>
  );
}

export default HistoryWindow;
