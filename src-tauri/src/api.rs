use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ORIGIN, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.dataforazeroth.com";
const SITE_BASE: &str = "https://www.dataforazeroth.com";

const RANKING_KEYS: &[(&str, &str)] = &[
    ("completion-score", "Completion Score"),
    ("gameplay-score", "Gameplay Score"),
    ("completion-count", "Completion Count"),
    ("achievement-points", "Achievement Points"),
    ("account-mounts", "Mounts"),
    ("pets-score", "Pet Score"),
    ("account-titles", "Titles"),
    ("account-reputations", "Reputations"),
    ("account-recipes", "Recipes"),
    ("account-quests", "Quests"),
    ("account-toys", "Toys"),
    ("account-appearance-sources", "Appearance Sources"),
    ("heirlooms-score", "Heirloom Score"),
    ("account-decor", "Decor"),
    ("achievements", "Achievements"),
    ("feats", "Feats of Strength"),
    ("legacy", "Legacy Achievements"),
    ("pets", "Pets"),
    ("account-appearances", "Appearances"),
    ("heirlooms", "Heirlooms"),
    ("alts", "Alts"),
    ("alts-score", "Alt Score"),
    ("honorlevel", "Honor Level"),
    ("account-kills", "Honorable Kills"),
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankInfo {
    pub category: String,
    pub value: String,
    pub percentage: String,
    pub world_rank: String,
    pub region_rank: String,
    pub region_label: String,
    pub realm_rank: String,
    pub realm_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileData {
    pub character: String,
    pub realm: String,
    pub region: String,
    pub completion_score: Option<RankInfo>,
    pub rankings: Vec<RankInfo>,
    pub error: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterSummary {
    pub name: String,
    pub realm: String,
    pub region: String,
    pub class_name: String,
    pub race_name: String,
    pub level: u32,
    pub item_level: u32,
    pub guild: String,
    pub professions: Vec<String>,
    pub thumbnail: String,
}

fn class_name(id: u64) -> &'static str {
    match id {
        1 => "Warrior", 2 => "Paladin", 3 => "Hunter", 4 => "Rogue",
        5 => "Priest", 6 => "Death Knight", 7 => "Shaman", 8 => "Mage",
        9 => "Warlock", 10 => "Monk", 11 => "Druid", 12 => "Demon Hunter",
        13 => "Evoker", _ => "Unknown",
    }
}

fn race_name(id: u64) -> &'static str {
    match id {
        1 => "Human", 2 => "Orc", 3 => "Dwarf", 4 => "Night Elf",
        5 => "Undead", 6 => "Tauren", 7 => "Gnome", 8 => "Troll",
        9 => "Goblin", 10 => "Blood Elf", 11 => "Draenei",
        22 => "Worgen", 24 | 25 | 26 => "Pandaren",
        27 => "Nightborne", 28 => "Highmountain Tauren",
        29 => "Void Elf", 30 => "Lightforged Draenei",
        31 => "Zandalari Troll", 32 => "Kul Tiran",
        34 => "Dark Iron Dwarf", 35 => "Vulpera",
        36 => "Mag'har Orc", 37 => "Mechagnome",
        52 => "Dracthyr", 70 => "Dracthyr",
        84 => "Earthen", 85 => "Earthen",
        _ => "Unknown",
    }
}


#[derive(Deserialize)]
struct VersionResponse {
    max: Option<String>,
}

#[derive(Deserialize)]
struct CharacterResponse {
    character: Option<CharacterInfo>,
    scores: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize)]
struct CharacterInfo {
    name: Option<String>,
    realm: Option<String>,
    updated: Option<u64>,
}

#[derive(Deserialize)]
struct RankEntry {
    world: Option<i64>,
    region: Option<i64>,
    realm: Option<i64>,
}

fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/json; charset=iso-8859-1"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.dataforazeroth.com/"),
    );
    headers.insert(
        ORIGIN,
        HeaderValue::from_static("https://www.dataforazeroth.com"),
    );
    headers
}

fn format_rank(raw: i64) -> String {
    let val = raw.unsigned_abs();
    let formatted = format_number(val);
    if raw < 0 {
        format!("{formatted}+")
    } else {
        formatted
    }
}

fn format_number(val: u64) -> String {
    let s = val.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

fn format_value(val: f64) -> String {
    if val == val.floor() {
        format_number(val as u64)
    } else {
        format!("{val:.1}")
    }
}

pub async fn fetch_profile(region: &str, realm: &str, character: &str) -> ProfileData {
    let mut profile = ProfileData {
        character: character.to_string(),
        realm: realm.to_string(),
        region: region.to_uppercase(),
        ..Default::default()
    };

    let client = match reqwest::Client::builder()
        .default_headers(build_headers())
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            profile.error = format!("Failed to build HTTP client: {e}");
            return profile;
        }
    };

    // 1. Fetch max values for percentage calculation
    let max_values: HashMap<String, f64> = match fetch_max_values(&client).await {
        Ok(m) => m,
        Err(_) => HashMap::new(),
    };

    // 2. Fetch character data
    let char_url = format!("{API_BASE}/characters/{region}/{realm}/{character}");
    let char_resp = match client.get(&char_url).send().await {
        Ok(r) => r,
        Err(e) => {
            profile.error = format!("Failed to fetch character: {e}");
            return profile;
        }
    };

    if !char_resp.status().is_success() {
        profile.error = format!("Character API returned {}", char_resp.status());
        return profile;
    }

    let char_data: CharacterResponse = match char_resp.json().await {
        Ok(d) => d,
        Err(e) => {
            profile.error = format!("Failed to parse character data: {e}");
            return profile;
        }
    };

    if let Some(info) = &char_data.character {
        if let Some(name) = &info.name {
            profile.character = name.clone();
        }
        if let Some(r) = &info.realm {
            profile.realm = r.clone();
        }
        if let Some(ts) = info.updated {
            profile.updated_at = ts;
        }
    }

    let scores = match char_data.scores {
        Some(s) => s,
        None => {
            profile.error = "No scores data in response".into();
            return profile;
        }
    };

    // 3. Build ranking query params
    let mut ranking_params: Vec<(String, String)> = Vec::new();
    for (api_key, _) in RANKING_KEYS {
        if let Some(val) = scores.get(*api_key) {
            if let Some(n) = val.as_f64() {
                ranking_params.push((api_key.to_string(), (n as i64).to_string()));
            }
        }
    }

    // 4. Fetch rankings
    let ranking_url = format!("{API_BASE}/rankings/{region}/{realm}");
    let rank_resp = match client.get(&ranking_url).query(&ranking_params).send().await {
        Ok(r) => r,
        Err(e) => {
            profile.error = format!("Failed to fetch rankings: {e}");
            return profile;
        }
    };

    let rank_data: HashMap<String, RankEntry> = match rank_resp.json().await {
        Ok(d) => d,
        Err(e) => {
            profile.error = format!("Failed to parse rankings: {e}");
            return profile;
        }
    };

    // 5. Build RankInfo objects
    for (api_key, display_name) in RANKING_KEYS {
        let score_val = match scores.get(*api_key).and_then(|v| v.as_f64()) {
            Some(v) => v,
            None => continue,
        };

        let rank_entry = rank_data.get(*api_key);

        let mut percentage = String::new();
        if let Some(&mv) = max_values.get(*api_key) {
            if mv > 0.0 {
                let pct = (score_val / mv) * 100.0;
                if pct > 0.0 {
                    percentage = format!("{:.1}%", pct.min(100.0));
                }
            }
        }

        let info = RankInfo {
            category: display_name.to_string(),
            value: format_value(score_val),
            percentage,
            world_rank: rank_entry
                .and_then(|r| r.world.map(format_rank))
                .unwrap_or_default(),
            region_rank: rank_entry
                .and_then(|r| r.region.map(format_rank))
                .unwrap_or_default(),
            region_label: region.to_uppercase(),
            realm_rank: rank_entry
                .and_then(|r| r.realm.map(format_rank))
                .unwrap_or_default(),
            realm_label: profile.realm.clone(),
        };

        if *api_key == "completion-score" {
            profile.completion_score = Some(info);
        } else {
            profile.rankings.push(info);
        }
    }

    profile
}

const PRIMARY_PROFESSIONS: &[&str] = &[
    "alchemy", "blacksmithing", "enchanting", "engineering",
    "herbalism", "inscription", "jewelcrafting", "leatherworking",
    "mining", "skinning", "tailoring",
];

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub async fn fetch_characters_batch(
    entries: &[(String, String, String)],
) -> Vec<CharacterSummary> {
    let client = match reqwest::Client::builder()
        .default_headers(build_headers())
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let stubs_body: Vec<serde_json::Value> = entries
        .iter()
        .map(|(region, realm, name)| {
            serde_json::json!({
                "region": region,
                "realm": realm,
                "character": name
            })
        })
        .collect();

    let url = format!("{API_BASE}/stubs");
    let resp = match client.post(&url).json(&stubs_body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Stubs request failed: {e}");
            return Vec::new();
        }
    };

    let stubs: Vec<serde_json::Value> = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Stubs parse failed: {e}");
            return Vec::new();
        }
    };

    let mut results = Vec::new();
    for (i, stub) in stubs.iter().enumerate() {
        let ch = match stub.get("character") {
            Some(c) => c,
            None => continue,
        };

        let region = entries.get(i).map(|e| e.0.as_str()).unwrap_or("EU");
        let region_lower = region.to_lowercase();

        let thumb = ch.get("thumbnail").and_then(|v| v.as_str()).unwrap_or("");
        let thumbnail = if thumb.is_empty() {
            String::new()
        } else {
            format!("https://render.worldofwarcraft.com/{region_lower}/character/{thumb}")
        };

        let mut professions = Vec::new();
        if let Some(profs) = ch.get("professions").and_then(|v| v.as_object()) {
            for &prof_name in PRIMARY_PROFESSIONS {
                if profs.contains_key(prof_name) {
                    professions.push(capitalize(prof_name));
                }
            }
        }

        results.push(CharacterSummary {
            name: ch.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            realm: ch.get("realm").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            region: region.to_uppercase(),
            class_name: class_name(ch.get("class").and_then(|v| v.as_u64()).unwrap_or(0)).to_string(),
            race_name: race_name(ch.get("race").and_then(|v| v.as_u64()).unwrap_or(0)).to_string(),
            level: ch.get("level").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            item_level: ch.get("averageItemLevel").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            guild: ch.get("guildName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            professions,
            thumbnail,
        });
    }

    results
}

pub async fn fetch_updated_timestamp(region: &str, realm: &str, character: &str) -> Result<u64, String> {
    let client = reqwest::Client::builder()
        .default_headers(build_headers())
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{API_BASE}/characters/{region}/{realm}/{character}");
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let data: CharacterResponse = resp.json().await.map_err(|e| e.to_string())?;
    data.character
        .and_then(|c| c.updated)
        .ok_or_else(|| "No updated timestamp".into())
}

async fn fetch_max_values(client: &reqwest::Client) -> Result<HashMap<String, f64>, String> {
    let version_url = format!("{API_BASE}/version");
    let version_resp = client
        .get(&version_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let version_data: VersionResponse = version_resp.json().await.map_err(|e| e.to_string())?;

    let max_path = version_data.max.ok_or("No max URL in version")?;
    let max_url = format!("{SITE_BASE}{max_path}");
    let max_resp = client
        .get(&max_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let max_data: HashMap<String, serde_json::Value> =
        max_resp.json().await.map_err(|e| e.to_string())?;

    let mut result = HashMap::new();
    for (k, v) in max_data {
        if let Some(n) = v.as_f64() {
            result.insert(k, n);
        }
    }
    Ok(result)
}
