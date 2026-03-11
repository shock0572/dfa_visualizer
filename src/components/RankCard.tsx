import type { RankInfo } from "../types";

interface Props {
  rank: RankInfo;
}

export function RankCard({ rank }: Props) {
  const badges = [
    { label: "W", value: rank.world_rank },
    { label: rank.region_label || "R", value: rank.region_rank },
    { label: rank.realm_label || "Rlm", value: rank.realm_rank },
  ].filter((b) => b.value);

  return (
    <div className="rank-card">
      <div className="card-category">{rank.category}</div>
      <div className="card-badges-row">
        {badges.map((b) => (
          <span key={b.label} className="mini-badge">
            <span className="mini-label">{b.label}</span>
            <span className="mini-value">{b.value}</span>
          </span>
        ))}
      </div>
      <div className="card-value">{rank.value || "—"}</div>
      {rank.percentage && <div className="card-pct">{rank.percentage}</div>}
    </div>
  );
}
