# GTA Discord Bot

A Discord bot written in Rust that provides information about GTA Online, including weekly updates, server status, and searchable GTA Wiki information.

Sadly this isnt hosted anywhere (yet) i just have this working on my local term but i used this to learn async and api calls using rust and i always wanted to build a discrd bot (mostly for the dev tag hehe )

## Features

- **Weekly GTA Online Updates**: Get the latest bonuses, discounts, podium vehicles, and prize rides
- **Server Status**: Check if GTA Online servers are operational
- **Vehicle Information**: Look up details about specific GTA vehicles
- **GTA Wiki Search**: Search the GTA Wiki for any GTA-related topic

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `!info` | Shows general information about the bot | `!info` |
| `!weekly` | Shows the latest GTA Online weekly update | `!weekly` |
| `!status` | Checks GTA Online server status | `!status` |
| `!vehicle [name]` | Looks up information about a specific vehicle | `!vehicle Oppressor` |
| `!search [term]` | Searches the GTA Wiki for any term | `!search Los Santos` |
| `!help_gta` | Shows GTA-specific help | `!help_gta` |
| `!help` | Shows all available commands | `!help` |

## Technical Overview

### Project Structure

```
gta_discord_bot/
├── src/
│   ├── main.rs                 # Main bot code with Discord commands
│   └── api/                    # API modules
│       ├── mod.rs              # API module exports
│       ├── gta_wiki.rs         # GTA Wiki API functionality
│       └── weekly_updates.rs   # Weekly updates and server status
├── Cargo.toml                  # Dependencies
└── .env                        # Environment variables (Discord token)
```

### Dependencies

- **serenity**: Discord API integration
- **tokio**: Async runtime
- **reqwest**: HTTP client for API calls
- **scraper**: HTML parsing for web scraping
- **serde/serde_json**: JSON serialization/deserialization
- **dotenv**: Environment variable loading

## How It Works

### Discord Integration

The bot uses the Serenity crate to interact with Discord. It sets up:

1. A connection to Discord using your bot token
2. Command handling with the command framework
3. Event handling for Discord events like ready and messages

```rust
#[tokio::main]
async fn main() {
    // Load bot token from .env file
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Missing Discord token");
    
    // Set up command framework
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .help(&MY_HELP)
        .group(&GTA_GROUP);
    
    // Configure intents (permissions)
    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;
    
    // Create and start the client
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
        
    client.start().await.expect("Error starting client");
}
```

### GTA Wiki Integration (`gta_wiki.rs`)

This module interfaces with the GTA Wiki's MediaWiki API to:

1. Search for GTA-related articles
2. Extract information like titles, descriptions, and images
3. Return structured data for Discord messages

```rust
pub async fn search_gta_wiki(search_term: &str) -> Result<Vec<WikiArticle>, Box<dyn Error + Send + Sync>> {
    // Make an API request to the GTA Wiki search endpoint
    // Process the response and extract article data
    // Return structured WikiArticle objects
}

pub async fn get_vehicle_info(vehicle_name: &str) -> Result<Option<WikiArticle>, Box<dyn Error + Send + Sync>> {
    // Search for a specific vehicle
    // Return the first matching result
}
```

### Weekly Updates (`weekly_updates.rs`)

This module scrapes Rockstar's Newswire to find the latest GTA Online weekly update:

1. Fetches the Newswire page filtered for GTA Online
2. Extracts the latest article information
3. Fetches the full article content
4. Parses the content to find bonuses, discounts, etc.

Due to thread safety concerns with HTML parsing, it uses Tokio's `spawn_blocking` to perform the parsing in separate threads:

```rust
pub async fn get_latest_weekly_update() -> Result<WeeklyUpdate, Box<dyn Error + Send + Sync>> {
    // Fetch the Newswire page
    let html_content = reqwest::get(url).await?.text().await?;
    
    // Parse the HTML in a blocking task
    let article_data = tokio::task::spawn_blocking(move || {
        extract_article_info(&html_content)
    }).await??;
    
    // Fetch and parse the full article content
    let article_html = reqwest::get(&article_data.url).await?.text().await?;
    let content_data = tokio::task::spawn_blocking(move || {
        extract_article_content(&article_html)
    }).await??;
    
    // Return structured weekly update data
}
```

### Server Status

The bot can check GTA Online server status by querying Rockstar's status API:

```rust
pub async fn check_server_status() -> Result<String, Box<dyn Error + Send + Sync>> {
    // Make a request to Rockstar's status API
    // Parse the JSON response
    // Extract and return the GTA Online status
}
```

## Command Handling

Commands are defined using Serenity's command framework. For example, the vehicle command:

```rust
#[command]
async fn vehicle(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "Please provide a vehicle name").await?;
        return Ok(());
    }
    
    let vehicle_name = args.rest();
    let mut processing_msg = msg.channel_id.say(&ctx.http, 
        format!("Looking up info for '{}'...", vehicle_name)).await?;
    
    match get_vehicle_info(vehicle_name).await {
        Ok(Some(vehicle)) => {
            // Create and send an embed with vehicle info
        },
        Ok(None) => {
            // Send a message that the vehicle wasn't found
        },
        Err(e) => {
            // Handle and report the error
        }
    }
    
    Ok(())
}
```

## Error Handling and Thread Safety

The bot includes comprehensive error handling to ensure a good user experience:

1. **Error Propagation**: Uses Rust's `?` operator to propagate errors up the call stack
2. **User-Friendly Messages**: Converts errors into user-friendly Discord messages
3. **Thread Safety**: Uses `spawn_blocking` to handle non-`Send` types in HTML parsing

## Setup Guide

1. Create a Discord Application and Bot at the [Discord Developer Portal](https://discord.com/developers/applications)
2. Get your bot token from the Bot tab
3. Create a `.env` file with `DISCORD_TOKEN=your_token_here`
4. Install Rust if not already installed
5. Build and run the bot:
   ```bash
   cargo run
   ```
6. Invite the bot to your server using the OAuth2 URL Generator in the Discord Developer Portal

## Deployment Considerations

For production deployment:

1. Set up proper logging instead of println statements
2. Consider adding a database for caching and storing data
3. Implement rate limiting to prevent API abuse
4. Add monitoring and restart mechanisms

## Extending the Bot

To add new commands:

1. Create a new command function with the `#[command]` attribute
2. Add the command to the command group in `main.rs`
3. Update the help messages to include the new command

```rust
#[command]
async fn my_new_command(ctx: &Context, msg: &Message) -> CommandResult {
    // Command implementation
    Ok(())
}
```

Then add it to the group:

```rust
#[group]
#[commands(info, weekly, status, vehicle, search, help_gta, my_new_command)]
struct GTA;
```