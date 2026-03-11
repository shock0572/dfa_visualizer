import { useEffect, useState, useRef } from "react";
import type { AppConfig, CharacterEntry } from "../types";
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
  const [newChar, setNewChar] = useState<CharacterEntry>({ region: "EU", realm: "", name: "" });
  const [showBulk, setShowBulk] = useState(false);
  const [bulkText, setBulkText] = useState("");
  const [bulkRegion, setBulkRegion] = useState("EU");
  const [bulkRealm, setBulkRealm] = useState("");

  useEffect(() => {
    Promise.all([loadSettings(), getAllCategories()]).then(([cfg, cats]) => {
      if (!cfg.extra_characters) cfg.extra_characters = [];
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

  const addCharacter = () => {
    if (!newChar.realm.trim() || !newChar.name.trim()) return;
    setConfig({
      ...config,
      extra_characters: [...config.extra_characters, { ...newChar }],
    });
    setNewChar({ ...newChar, realm: "", name: "" });
  };

  const bulkImportParsed = () => {
    const lines = bulkText.split(/\n+/).map((l) => l.trim()).filter((l) => l.includes("/"));
    const newEntries: CharacterEntry[] = lines
      .map((line) => {
        const parts = line.split("/");
        if (parts.length >= 3) {
          return { region: parts[0], realm: parts[1], name: parts.slice(2).join("/") };
        }
        return null;
      })
      .filter(Boolean) as CharacterEntry[];

    const existing = new Set(
      config.extra_characters.map((c) => `${c.region}/${c.realm}/${c.name}`.toLowerCase())
    );
    const mainKey = `${config.region}/${config.realm}/${config.character}`.toLowerCase();
    const filtered = newEntries.filter(
      (e) => {
        const key = `${e.region}/${e.realm}/${e.name}`.toLowerCase();
        return !existing.has(key) && key !== mainKey;
      }
    );
    setConfig({
      ...config,
      extra_characters: [...config.extra_characters, ...filtered],
    });
    setBulkText("");
    setShowBulk(false);
  };

  const bulkImport = () => {
    if (!bulkRealm.trim() || !bulkText.trim()) return;
    const names = bulkText
      .split(/[\n,]+/)
      .map((n) => n.trim())
      .filter((n) => n.length > 0);
    const newEntries: CharacterEntry[] = names.map((name) => ({
      region: bulkRegion,
      realm: bulkRealm.trim(),
      name,
    }));
    const existing = new Set(
      config.extra_characters.map((c) => `${c.region}/${c.realm}/${c.name}`.toLowerCase())
    );
    const filtered = newEntries.filter(
      (e) => !existing.has(`${e.region}/${e.realm}/${e.name}`.toLowerCase())
    );
    setConfig({
      ...config,
      extra_characters: [...config.extra_characters, ...filtered],
    });
    setBulkText("");
    setShowBulk(false);
  };

  const removeCharacter = (i: number) => {
    const next = [...config.extra_characters];
    next.splice(i, 1);
    setConfig({ ...config, extra_characters: next });
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
      <h2>Main Character</h2>

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
          placeholder="e.g. Quorra"
        />
      </div>

      <hr className="divider" />
      <h2>Alt Characters</h2>

      {config.extra_characters.map((ch, i) => (
        <div key={i} className="alt-row">
          <span className="alt-info">{ch.name} - {ch.realm} ({ch.region})</span>
          <button className="alt-remove" onClick={() => removeCharacter(i)}>x</button>
        </div>
      ))}

      <div className="alt-add">
        <select
          value={newChar.region}
          onChange={(e) => setNewChar({ ...newChar, region: e.target.value })}
          style={{ width: 60 }}
        >
          {REGIONS.map((r) => (
            <option key={r} value={r}>{r}</option>
          ))}
        </select>
        <input
          type="text"
          value={newChar.realm}
          onChange={(e) => setNewChar({ ...newChar, realm: e.target.value })}
          placeholder="Realm"
          style={{ flex: 1 }}
        />
        <input
          type="text"
          value={newChar.name}
          onChange={(e) => setNewChar({ ...newChar, name: e.target.value })}
          placeholder="Name"
          style={{ flex: 1 }}
        />
        <button className="btn btn-primary" onClick={addCharacter} style={{ padding: "6px 12px" }}>+</button>
      </div>

      <button
        className="btn btn-secondary"
        onClick={() => setShowBulk(!showBulk)}
        style={{ marginTop: 8, fontSize: 12, padding: "5px 12px" }}
      >
        {showBulk ? "Hide Import" : "Import from DFA..."}
      </button>

      {showBulk && (
        <div className="bulk-import">
          <p className="import-help">
            1. Open <a href="https://www.dataforazeroth.com/mycharacters" target="_blank" rel="noreferrer">DFA My Characters</a> and log in<br/>
            2. Press F12, go to Console, paste this and press Enter:
          </p>
          <code className="import-snippet" onClick={(e) => {
            navigator.clipboard.writeText((e.target as HTMLElement).textContent || "");
            setError("Copied to clipboard!");
            setTimeout(() => setError(""), 2000);
          }}>
            {`copy(Array.from(document.querySelectorAll('a[href*="/characters/"]')).map(a=>{let m=a.href.match(/characters\\/([^\\/]+)\\/([^\\/]+)\\/([^\\/]+)/);return m?m[1].toUpperCase()+'/'+decodeURIComponent(m[2])+'/'+decodeURIComponent(m[3]):null}).filter(Boolean).filter((v,i,a)=>a.indexOf(v)===i).join('\\n'))`}
          </code>
          <p className="import-help">3. Paste the result below:</p>
          <textarea
            className="bulk-textarea"
            value={bulkText}
            onChange={(e) => setBulkText(e.target.value)}
            placeholder={"EU/Silvermoon/Samarcanda\nEU/Silvermoon/Brianda\n..."}
            rows={6}
          />
          <button className="btn btn-primary" onClick={bulkImportParsed} style={{ fontSize: 12, padding: "5px 12px" }}>
            Import {bulkText.split(/\n+/).filter((n) => n.trim().includes("/")).length} characters
          </button>
        </div>
      )}

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
