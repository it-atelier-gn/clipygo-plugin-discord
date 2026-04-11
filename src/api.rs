use base64::Engine;
use reqwest::blocking::Client;
use serde::Deserialize;

const API_BASE: &str = "https://discord.com/api/v10";

fn client(token: &str) -> Client {
    Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bot {token}").parse().unwrap(),
            );
            headers
        })
        .build()
        .expect("Failed to build HTTP client")
}

#[derive(Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: u8,
    pub position: Option<i32>,
}

pub fn fetch_guilds(token: &str) -> Result<Vec<Guild>, String> {
    let resp = client(token)
        .get(format!("{API_BASE}/users/@me/guilds"))
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("Discord API error ({status}): {body}"));
    }

    resp.json().map_err(|e| format!("Bad response: {e}"))
}

pub fn fetch_channels(token: &str, guild_id: &str) -> Result<Vec<Channel>, String> {
    let resp = client(token)
        .get(format!("{API_BASE}/guilds/{guild_id}/channels"))
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("Discord API error ({status}): {body}"));
    }

    resp.json().map_err(|e| format!("Bad response: {e}"))
}

pub fn send_text(token: &str, channel_id: &str, text: &str) -> Result<(), String> {
    let resp = client(token)
        .post(format!("{API_BASE}/channels/{channel_id}/messages"))
        .json(&serde_json::json!({ "content": text }))
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        Err(format!("Discord API error ({status}): {body}"))
    }
}

pub fn send_image(token: &str, channel_id: &str, base64_data: &str) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Invalid base64: {e}"))?;

    let part = reqwest::blocking::multipart::Part::bytes(bytes)
        .file_name("clipboard.png")
        .mime_str("image/png")
        .map_err(|e| format!("MIME error: {e}"))?;

    let form = reqwest::blocking::multipart::Form::new().part("files[0]", part);

    let resp = client(token)
        .post(format!("{API_BASE}/channels/{channel_id}/messages"))
        .multipart(form)
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        Err(format!("Discord API error ({status}): {body}"))
    }
}
