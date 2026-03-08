import type { ProfileData } from "../types";
import { RankCard } from "./RankCard";

interface Props {
  profile: ProfileData;
  tracked: Set<string>;
}

export function RankingsGrid({ profile, tracked }: Props) {
  const filtered = profile.rankings.filter((r) => tracked.has(r.category));

  if (filtered.length === 0) {
    return (
      <div className="no-data">
        <p>No rankings to display. Check your tracked categories in Settings.</p>
      </div>
    );
  }

  return (
    <div className="rankings-container">
      <div className="section-label">Account Wide Leaderboards</div>
      <div className="rankings-grid">
        {filtered.map((rank) => (
          <RankCard key={rank.category} rank={rank} />
        ))}
      </div>
    </div>
  );
}
