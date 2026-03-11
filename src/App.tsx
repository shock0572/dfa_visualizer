import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ProfileData, AppConfig } from "./types";
import { open } from "@tauri-apps/plugin-shell";
import { getProfile, loadSettings, fetchProfile, openDfaUpdate } from "./api";
import { Overview } from "./components/Overview";
import { RankingsGrid } from "./components/RankingsGrid";
import { Settings } from "./components/Settings";
import "./styles/app.css";

type View = "overview" | "rankings" | "settings";

export default function App() {
  const [view, setView] = useState<View>("overview");
  const [profile, setProfile] = useState<ProfileData | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<string | null>(null);
  const [watching, setWatching] = useState(false);
  const blurTimeout = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [p, c] = await Promise.all([getProfile(), loadSettings()]);
      if (p) setProfile(p);
      setConfig(c);
    } catch {
      /* initial load before first fetch */
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();

    const unlistenProfile = listen<ProfileData>("profile-updated", (event) => {
      setProfile(event.payload);
      setLoading(false);
    });

    const unlistenNav = listen<string>("navigate", (event) => {
      if (event.payload === "settings") setView("settings");
    });

    const unlistenStatus = listen<string>("update-status", (event) => {
      if (event.payload === "done") {
        setWatching(false);
        setUpdateStatus("Updated!");
        setTimeout(() => setUpdateStatus(null), 4000);
      }
    });


    // Hide window on blur with a delay so dragging doesn't dismiss it
    const appWindow = getCurrentWindow();
    const unlistenFocus = appWindow.onFocusChanged(({ payload: focused }) => {
      if (blurTimeout.current) {
        clearTimeout(blurTimeout.current);
        blurTimeout.current = null;
      }
      if (!focused) {
        blurTimeout.current = window.setTimeout(() => {
          appWindow.hide();
        }, 300);
      }
    });

    return () => {
      unlistenProfile.then((f) => f());
      unlistenNav.then((f) => f());
      unlistenStatus.then((f) => f());
      unlistenFocus.then((f) => f());
    };
  }, [refresh]);

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      const p = await fetchProfile();
      setProfile(p);
    } catch {
      /* ignore */
    } finally {
      setRefreshing(false);
    }
  };

  const handleUpdateChars = () => {
    setWatching(true);
    open("https://www.dataforazeroth.com/mycharacters");
    openDfaUpdate().catch(() => {});
  };

  if (view === "settings") {
    return (
      <div className="app">
        <Settings
          onSaved={() => {
            setLoading(true);
            setView("overview");
            refresh();
          }}
          onCancel={() => setView("overview")}
        />
      </div>
    );
  }

  const tracked = new Set(config?.tracked_rankings ?? []);
  const isConfigured = config && config.realm && config.character;

  return (
    <div className="app">
      <div className="tabs" data-tauri-drag-region>
        <button
          className={`tab ${view === "overview" ? "active" : ""}`}
          onClick={() => setView("overview")}
        >
          Overview
        </button>
        <button
          className={`tab ${view === "rankings" ? "active" : ""}`}
          onClick={() => setView("rankings")}
        >
          Account Wide
        </button>
        <button className="tab" onClick={() => setView("settings")}>
          Settings
        </button>
      </div>

      {!isConfigured ? (
        <div className="no-data">
          <p>No character configured yet.</p>
          <button className="btn btn-primary" onClick={() => setView("settings")}>
            Open Settings
          </button>
        </div>
      ) : loading && !profile ? (
        <div className="loading">Loading profile data...</div>
      ) : profile?.error ? (
        <>
          <div className="error-banner">{profile.error}</div>
          {view === "overview" && profile.completion_score && (
            <Overview profile={profile} />
          )}
        </>
      ) : profile ? (
        view === "overview" ? (
          <Overview profile={profile} />
        ) : (
          <RankingsGrid profile={profile} tracked={tracked} />
        )
      ) : (
        <div className="loading">Waiting for data...</div>
      )}

      {isConfigured && (
        <div className="bottom-bar">
          <button
            className="bottom-btn"
            onClick={handleUpdateChars}
            disabled={watching}
            title="Open DFA to update via Battle.net, then auto-refresh"
          >
            {watching ? "Watching for update..." : "Update Characters"}
          </button>
          <button
            className="bottom-btn"
            onClick={handleRefresh}
            disabled={refreshing}
          >
            {refreshing ? "Refreshing..." : "Refresh"}
          </button>
          {updateStatus && <span className="update-status">{updateStatus}</span>}
        </div>
      )}
    </div>
  );
}
