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

export interface CharacterEntry {
  region: string;
  realm: string;
  name: string;
}

export interface CharacterSummary {
  name: string;
  realm: string;
  region: string;
  class_name: string;
  race_name: string;
  level: number;
  item_level: number;
  guild: string;
  professions: string[];
  thumbnail: string;
}

export interface AppConfig {
  region: string;
  realm: string;
  character: string;
  tracked_rankings: string[];
  refresh_interval_minutes: number;
  extra_characters: CharacterEntry[];
}
