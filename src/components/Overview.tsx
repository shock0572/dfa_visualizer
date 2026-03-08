import type { ProfileData } from "../types";

interface Props {
  profile: ProfileData;
}

export function Overview({ profile }: Props) {
  const cs = profile.completion_score;

  if (!cs) {
    return <div className="no-data"><p>No completion score data available.</p></div>;
  }

  const badges = [
    { label: "World", value: cs.world_rank },
    { label: cs.region_label || profile.region, value: cs.region_rank },
    { label: cs.realm_label || profile.realm, value: cs.realm_rank },
  ].filter((b) => b.value);

  return (
    <div className="overview-container">
      <div className="header">
        <span className="char-name">{profile.character}</span>
        <span className="char-realm">
          {profile.realm} ({profile.region})
        </span>
      </div>

      <div className="large-card">
        <div className="card-left">
          <div className="card-category">{cs.category}</div>
          <div className="card-value">{cs.value}</div>
          {cs.percentage && <div className="card-pct">{cs.percentage}</div>}
        </div>
        <div className="card-ranks">
          {badges.map((b) => (
            <div key={b.label} className="rank-row">
              <span className="rank-label">{b.label}</span>
              <span className="badge">{b.value}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
