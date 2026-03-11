import { invoke } from "@tauri-apps/api/core";
import type { ProfileData, AppConfig, CharacterSummary } from "./types";

export async function fetchProfile(): Promise<ProfileData> {
  return invoke<ProfileData>("fetch_profile");
}

export async function getProfile(): Promise<ProfileData | null> {
  return invoke<ProfileData | null>("get_profile");
}

export async function loadSettings(): Promise<AppConfig> {
  return invoke<AppConfig>("load_settings");
}

export async function saveSettings(config: AppConfig): Promise<void> {
  return invoke<void>("save_settings", { config });
}

export async function getAllCategories(): Promise<string[]> {
  return invoke<string[]>("get_all_categories");
}

export async function fetchAllCharacters(): Promise<CharacterSummary[]> {
  return invoke<CharacterSummary[]>("fetch_all_characters");
}

export async function openDfaUpdate(): Promise<void> {
  return invoke<void>("open_dfa_update");
}

export async function startUpdateWatch(): Promise<void> {
  return invoke<void>("start_update_watch");
}
