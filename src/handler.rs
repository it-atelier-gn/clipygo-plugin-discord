use crate::api;
use crate::config::{load_config, save_config};
use crate::protocol::{InfoResponse, Request, SendResponse, Target, TargetsResponse};

// 1×1 pixel PNG, Discord blurple (#5865F2)
const ICON: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPj/HwADBwIAMCbHYQAAAABJRU5ErkJggg==";

// Discord channel types
const GUILD_TEXT: u8 = 0;
const GUILD_ANNOUNCEMENT: u8 = 5;

pub fn handle(request: Request) -> serde_json::Value {
    match request {
        Request::GetInfo => serde_json::to_value(InfoResponse {
            name: "Discord",
            version: env!("CARGO_PKG_VERSION"),
            description: "Send clipboard content to Discord channels",
            author: "clipygo",
            link: Some("https://github.com/it-atelier-gn/clipygo-plugin-discord"),
        })
        .unwrap(),

        Request::GetTargets => {
            let config = load_config();

            if config.bot_token.is_empty() {
                eprintln!("[discord] Bot token not configured");
                return serde_json::to_value(TargetsResponse { targets: vec![] }).unwrap();
            }

            let guilds = match api::fetch_guilds(&config.bot_token) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("[discord] Failed to fetch guilds: {e}");
                    return serde_json::to_value(TargetsResponse { targets: vec![] }).unwrap();
                }
            };

            let mut targets = Vec::new();

            for guild in &guilds {
                match api::fetch_channels(&config.bot_token, &guild.id) {
                    Ok(channels) => {
                        let mut text_channels: Vec<_> = channels
                            .into_iter()
                            .filter(|c| {
                                c.channel_type == GUILD_TEXT || c.channel_type == GUILD_ANNOUNCEMENT
                            })
                            .collect();
                        text_channels.sort_by_key(|c| c.position.unwrap_or(i32::MAX));

                        for channel in text_channels {
                            let name = channel.name.as_deref().unwrap_or("unknown");
                            targets.push(Target {
                                id: format!("channel:{}", channel.id),
                                provider: "Discord".to_string(),
                                formats: vec!["text".to_string(), "image".to_string()],
                                title: format!("#{name}"),
                                description: guild.name.clone(),
                                image: ICON.to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("[discord] Failed to fetch channels for {}: {e}", guild.name);
                    }
                }
            }

            serde_json::to_value(TargetsResponse { targets }).unwrap()
        }

        Request::GetConfigSchema => {
            let config = load_config();
            serde_json::json!({
                "instructions": "1. Go to https://discord.com/developers/applications\n\
                    2. Create a New Application\n\
                    3. Go to Bot → Reset Token → copy the token\n\
                    4. Under Privileged Gateway Intents, enable Message Content Intent\n\
                    5. Go to OAuth2 → URL Generator:\n\
                       - Scopes: bot\n\
                       - Bot Permissions: Send Messages, Attach Files\n\
                    6. Use the generated URL to invite the bot to your server(s)",
                "schema": {
                    "type": "object",
                    "title": "Discord",
                    "properties": {
                        "bot_token": {
                            "type": "string",
                            "title": "Bot Token",
                            "description": "Bot token from Discord Developer Portal",
                            "format": "password"
                        }
                    },
                    "required": ["bot_token"]
                },
                "values": {
                    "bot_token": config.bot_token
                }
            })
        }

        Request::SetConfig { values } => {
            let mut config = load_config();

            if let Some(v) = values.get("bot_token").and_then(|v| v.as_str()) {
                config.bot_token = v.to_string();
            }

            save_config(&config);

            serde_json::to_value(SendResponse {
                success: true,
                error: None,
            })
            .unwrap()
        }

        Request::Send {
            target_id,
            content,
            format,
        } => {
            let config = load_config();

            if config.bot_token.is_empty() {
                return serde_json::to_value(SendResponse {
                    success: false,
                    error: Some("Bot token not configured".to_string()),
                })
                .unwrap();
            }

            let channel_id = target_id.strip_prefix("channel:").unwrap_or(&target_id);

            let result = match format.as_str() {
                "text" => api::send_text(&config.bot_token, channel_id, &content),
                "image" => api::send_image(&config.bot_token, channel_id, &content),
                _ => Err(format!("Unsupported format: {format}")),
            };

            match result {
                Ok(()) => serde_json::to_value(SendResponse {
                    success: true,
                    error: None,
                })
                .unwrap(),
                Err(e) => serde_json::to_value(SendResponse {
                    success: false,
                    error: Some(e),
                })
                .unwrap(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_info_fields() {
        let resp = handle(Request::GetInfo);
        assert_eq!(resp["name"], "Discord");
        assert!(resp["version"].is_string());
        assert!(resp["description"].is_string());
        assert_eq!(resp["author"], "clipygo");
    }

    #[test]
    fn get_info_includes_link() {
        let resp = handle(Request::GetInfo);
        assert!(resp["link"].as_str().unwrap().starts_with("https://"));
    }

    #[test]
    fn get_targets_empty_when_no_token() {
        let resp = handle(Request::GetTargets);
        let targets = resp["targets"].as_array().unwrap();
        assert!(targets.is_empty());
    }

    #[test]
    fn get_config_schema_has_required_fields() {
        let resp = handle(Request::GetConfigSchema);
        assert!(resp.get("instructions").is_some());
        assert!(resp.get("schema").is_some());
        assert!(resp.get("values").is_some());
        let props = &resp["schema"]["properties"];
        assert!(props.get("bot_token").is_some());
    }

    #[test]
    fn get_config_schema_bot_token_is_password() {
        let resp = handle(Request::GetConfigSchema);
        let format = resp["schema"]["properties"]["bot_token"]["format"]
            .as_str()
            .unwrap();
        assert_eq!(format, "password");
    }

    #[test]
    fn set_config_returns_success() {
        let resp = handle(Request::SetConfig {
            values: serde_json::json!({ "bot_token": "test-token" }),
        });
        assert_eq!(resp["success"], true);
    }

    #[test]
    fn send_fails_without_token() {
        save_config(&crate::config::Config::default());
        let resp = handle(Request::Send {
            target_id: "channel:123".to_string(),
            content: "hello".to_string(),
            format: "text".to_string(),
        });
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("token"));
    }

    #[test]
    fn send_rejects_unsupported_format() {
        save_config(&crate::config::Config {
            bot_token: "fake-token".to_string(),
        });
        let resp = handle(Request::Send {
            target_id: "channel:123".to_string(),
            content: "data".to_string(),
            format: "video".to_string(),
        });
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("video"));
    }

    #[test]
    fn invalid_json_rejected() {
        assert!(serde_json::from_str::<Request>("not json").is_err());
    }

    #[test]
    fn unknown_command_rejected() {
        assert!(serde_json::from_str::<Request>(r#"{"command":"unknown"}"#).is_err());
    }

    #[test]
    fn config_roundtrip() {
        let config = crate::config::Config {
            bot_token: "test-token".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: crate::config::Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.bot_token, "test-token");
    }
}
