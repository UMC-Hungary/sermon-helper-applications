use serde::{Deserialize, Serialize};

// V2 API types (nyiregyhazimetodista.hu)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct V2Verse {
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct V2ParsedRef {
    pub book: String,
    pub book_id: i32,
    pub chapter_from: i32,
    pub chapter_to: Option<i32>,
    pub verse_from: Option<i32>,
    pub verse_to: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct V2SuggestResponse {
    pub label: String,
    pub link: String,
    pub hungarian_label: String,
    pub parsed_refs: Vec<V2ParsedRef>,
    pub verses: Vec<V2Verse>,
    pub verses_as_text: Vec<String>,
}

// Legacy API types (szentiras.eu)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyLocation {
    pub gepi: String,
    pub szep: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyNote {
    pub position: Option<i32>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyVerse {
    pub szoveg: String,
    #[serde(default)]
    pub jegyzetek: Vec<LegacyNote>,
    pub hely: LegacyLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyTranslation {
    pub nev: String,
    pub rov: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyAnswer {
    pub versek: Vec<LegacyVerse>,
    pub forditas: LegacyTranslation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegacySearchQuery {
    pub feladat: String,
    pub hivatkozas: String,
    pub forma: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegacySearchResponse {
    pub keres: LegacySearchQuery,
    pub valasz: LegacyAnswer,
}

// Suggestion from szentiras.eu
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacySuggestion {
    pub cat: String,
    pub label: String,
    pub link: String,
}

fn map_suggestion_label(label: &str) -> String {
    let books = [("Ter", "1Móz"), ("Kiv", "2Móz"), ("Lev", "3Móz"), ("Szám", "4Móz"), ("MTörv", "5Móz")];
    let mut result = label.to_string();
    for (from, to) in books.iter() {
        result = result.replace(from, to);
    }
    result
}

// Remove HTML heading tags from verse text
fn remove_headings(html: &str) -> String {
    let re = regex::Regex::new(r"<h[1-6][^>]*>[\s\S]*?</h[1-6]>").unwrap_or_else(|_| regex::Regex::new("").unwrap());
    re.replace_all(html, "").to_string()
}

// Remove <br> tags from verse text
fn remove_breaks(html: &str) -> String {
    let re = regex::Regex::new(r"<br\s*/?>(\s*)?").unwrap_or_else(|_| regex::Regex::new("").unwrap());
    re.replace_all(html, "").to_string()
}

// Clean HTML from verse text
fn clean_verse_text(text: &str) -> String {
    let cleaned = remove_headings(text);
    let cleaned = remove_breaks(&cleaned);
    // Remove any remaining HTML tags
    let re = regex::Regex::new(r"<[^>]*>").unwrap_or_else(|_| regex::Regex::new("").unwrap());
    re.replace_all(&cleaned, "").to_string()
}

// V2 API: Fetch verses directly (immediate results)
#[tauri::command]
pub async fn fetch_bible_v2(
    reference: String,
    translation: String,
    api_url: String,
) -> Result<V2SuggestResponse, String> {
    let url = format!("{}/suggest/{}/{}", api_url, urlencoding::encode(&reference), translation);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {} - {}", response.status(), url));
    }

    let mut data: V2SuggestResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    // Clean up verse text
    for verse in &mut data.verses {
        verse.text = clean_verse_text(&verse.text);
    }

    Ok(data)
}

// Legacy API: Get suggestions for autocomplete
#[tauri::command]
pub async fn fetch_bible_suggestions(
    term: String,
    api_url: String,
) -> Result<Vec<LegacySuggestion>, String> {
    let url = format!("{}/kereses/suggest?term={}", api_url, urlencoding::encode(&term));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let suggestions: Vec<LegacySuggestion> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    // Filter by cat === 'ref' and map book names
    let filtered: Vec<LegacySuggestion> = suggestions
        .into_iter()
        .filter(|s| s.cat == "ref")
        .map(|s| LegacySuggestion {
            cat: s.cat,
            label: map_suggestion_label(&s.label),
            link: map_suggestion_label(&s.link),
        })
        .collect();

    Ok(filtered)
}

// Encode only spaces in path segments (preserve commas, slashes, etc.)
fn encode_path_segment(s: &str) -> String {
    s.replace(" ", "%20")
}

// Legacy API: Fetch verses by reference
#[tauri::command]
pub async fn fetch_bible_legacy(
    reference: String,
    translation: String,
    api_url: String,
) -> Result<LegacySearchResponse, String> {
    // Strip leading slash if present and encode only spaces
    let clean_ref = reference.trim_start_matches('/');
    let url = format!("{}/api/idezet/{}/{}", api_url, encode_path_segment(clean_ref), translation);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let mut data: LegacySearchResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    // Clean up verse text
    for verse in &mut data.valasz.versek {
        verse.szoveg = clean_verse_text(&verse.szoveg);
    }

    Ok(data)
}
