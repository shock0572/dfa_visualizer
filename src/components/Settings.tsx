import { useEffect, useState } from "react";
import type { AppConfig } from "../types";
import { loadSettings, saveSettings, getAllCategories } from "../api";

interface Props {
  onSaved: () => void;
  onCancel: () => void;
}

const REGIONS = ["EU", "US", "KR", "TW"];

export function Settings({ onSaved, onCancel }: Props) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [allCategories, setAllCategories] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    Promise.all([loadSettings(), getAllCategories()]).then(([cfg, cats]) => {
      setConfig(cfg);
      setAllCategories(cats);
    });
  }, []);

  if (!config) return <div className="loading">Loading settings...</div>;

  const tracked = new Set(config.tracked_rankings);

  const toggleCategory = (cat: string) => {
    const next = new Set(tracked);
    if (next.has(cat)) next.delete(cat);
    else next.add(cat);
    setConfig({ ...config, tracked_rankings: Array.from(next) });
  };

  const handleSave = async () => {
    if (!config.realm.trim() || !config.character.trim()) {
      setError("Please enter both Realm and Character name.");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await saveSettings(config);
      onSaved();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="settings-page">
      <h2>Character Profile</h2>

      <div className="form-group">
        <label>Region</label>
        <select
          value={config.region}
          onChange={(e) => setConfig({ ...config, region: e.target.value })}
        >
          {REGIONS.map((r) => (
            <option key={r} value={r}>{r}</option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label>Realm</label>
        <input
          type="text"
          value={config.realm}
          onChange={(e) => setConfig({ ...config, realm: e.target.value })}
          placeholder="e.g. Silvermoon"
        />
      </div>

      <div className="form-group">
        <label>Character</label>
        <input
          type="text"
          value={config.character}
          onChange={(e) => setConfig({ ...config, character: e.target.value })}
          placeholder="e.g. Earthpug"
        />
      </div>

      <hr className="divider" />
      <h2>Tracked Rankings</h2>

      <div className="checkbox-grid">
        {allCategories.map((cat) => (
          <label key={cat} className="checkbox-item">
            <input
              type="checkbox"
              checked={tracked.has(cat)}
              onChange={() => toggleCategory(cat)}
            />
            {cat}
          </label>
        ))}
      </div>

      <hr className="divider" />

      <div className="form-group">
        <label>Refresh interval (minutes)</label>
        <input
          type="number"
          min={1}
          value={config.refresh_interval_minutes}
          onChange={(e) =>
            setConfig({
              ...config,
              refresh_interval_minutes: Math.max(1, parseInt(e.target.value) || 30),
            })
          }
        />
      </div>

      {error && <div className="error-banner">{error}</div>}

      <div className="btn-row">
        <button className="btn btn-secondary" onClick={onCancel}>
          Cancel
        </button>
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save"}
        </button>
      </div>
    </div>
  );
}
