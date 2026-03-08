export interface RankInfo {
  category: string;
  value: string;
  percentage: string;
  world_rank: string;
  region_rank: string;
  region_label: string;
  realm_rank: string;
  realm_label: string;
}

export interface ProfileData {
  character: string;
  realm: string;
  region: string;
  completion_score: RankInfo | null;
  rankings: RankInfo[];
  error: string;
  updated_at: number;
}

export interface AppConfig {
  region: string;
  realm: string;
  character: string;
  tracked_rankings: string[];
  refresh_interval_minutes: number;
}
