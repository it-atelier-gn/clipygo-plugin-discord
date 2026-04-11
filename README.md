# clipygo-plugin-discord

Discord target provider for [clipygo](https://github.com/it-atelier-gn/clipygo).

Sends clipboard content (text and images) to Discord text channels via the Bot API.

## Setup

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a New Application → go to Bot → Reset Token → copy the token
3. Under Privileged Gateway Intents, enable **Message Content Intent**
4. Go to OAuth2 → URL Generator:
   - Scopes: `bot`
   - Bot Permissions: `Send Messages`, `Attach Files`
5. Use the generated URL to invite the bot to your server(s)
6. In clipygo Settings → Plugins, add the plugin and paste the bot token

The plugin auto-discovers all text channels from servers the bot has joined.

## Supported formats

- **text** — sent as a regular message
- **image** — sent as a file attachment (base64-encoded PNG)

## Build

```sh
cargo build --release
```

## License

MIT
