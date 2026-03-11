import { useEffect, useState } from "react";
import type { ProfileData, CharacterSummary } from "../types";
import { fetchAllCharacters } from "../api";

interface Props {
  profile: ProfileData;
}

const CLASS_COLORS: Record<string, string> = {
  "Warrior": "#C79C6E",
  "Paladin": "#F58CBA",
  "Hunter": "#ABD473",
  "Rogue": "#FFF569",
  "Priest": "#FFFFFF",
  "Death Knight": "#C41F3B",
  "Shaman": "#0070DE",
  "Mage": "#69CCF0",
  "Warlock": "#9482C9",
  "Monk": "#00FF96",
  "Druid": "#FF7D0A",
  "Demon Hunter": "#A330C9",
  "Evoker": "#33937F",
};

export function Overview({ profile }: Props) {
  const [characters, setCharacters] = useState<CharacterSummary[]>([]);
  const [loadingChars, setLoadingChars] = useState(true);

  useEffect(() => {
    fetchAllCharacters()
      .then(setCharacters)
      .catch(() => {})
      .finally(() => setLoadingChars(false));
  }, []);

  const cs = profile.completion_score;

  return (
    <div className="overview-container">
      <div className="header">
        <span className="char-name">{profile.character}</span>
        <span className="char-realm">
          {profile.realm} ({profile.region})
        </span>
      </div>

      {cs && (
        <div className="large-card">
          <div className="card-left">
            <div className="card-category">{cs.category}</div>
            <div className="card-value">{cs.value}</div>
            {cs.percentage && <div className="card-pct">{cs.percentage}</div>}
          </div>
          <div className="card-ranks">
            {[
              { label: "World", value: cs.world_rank },
              { label: cs.region_label || profile.region, value: cs.region_rank },
              { label: cs.realm_label || profile.realm, value: cs.realm_rank },
            ]
              .filter((b) => b.value)
              .map((b) => (
                <div key={b.label} className="rank-row">
                  <span className="rank-label">{b.label}</span>
                  <span className="badge">{b.value}</span>
                </div>
              ))}
          </div>
        </div>
      )}

      <div className="section-label">Characters</div>
      <div className="char-list">
        {loadingChars ? (
          <div className="loading" style={{ height: 80 }}>Loading characters...</div>
        ) : characters.length === 0 ? (
          <div className="loading" style={{ height: 80 }}>No characters found</div>
        ) : (
          characters.map((ch) => (
            <div key={`${ch.region}-${ch.realm}-${ch.name}`} className="char-row">
              {ch.thumbnail && (
                <img className="char-avatar" src={ch.thumbnail} alt="" />
              )}
              <div className="char-details">
                <div className="char-row-top">
                  <span
                    className="char-row-name"
                    style={{ color: CLASS_COLORS[ch.class_name] || "#e0e0e0" }}
                  >
                    {ch.name}
                  </span>
                  <span className="char-row-level">{ch.level}</span>
                  <span className="char-row-ilvl">{ch.item_level} ilvl</span>
                </div>
                <div className="char-row-bottom">
                  <span className="char-row-meta">
                    {ch.race_name} {ch.class_name}
                    {ch.guild && <> &middot; &lt;{ch.guild}&gt;</>}
                  </span>
                </div>
                {ch.professions.length > 0 && (
                  <div className="char-row-profs">
                    {ch.professions.map((p) => (
                      <span key={p} className="prof-tag">{p}</span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
