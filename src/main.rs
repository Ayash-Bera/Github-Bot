use std::{collections::HashSet, env};

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandResult, HelpOptions, StandardFramework,
    CommandGroup,  // Added missing import
};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;
use serenity::builder::CreateEmbed;  // Kept only the CreateEmbed import
use serenity::utils::Colour;
use dotenv::dotenv;

// Import our API modules
mod api;
use api::{search_gta_wiki, get_vehicle_info, get_latest_weekly_update, check_server_status};  // Removed WikiArticle import

// Define a group of commands for GTA features
#[group]
#[commands(info, weekly, status, vehicle, search, help_gta)]
struct GTA;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        // Set bot status
        use serenity::model::gateway::Activity;
        ctx.set_activity(Activity::playing("with ur mom")).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Loading environment variables...");
    
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => {
            println!("Token loaded successfully");
            token
        },
        Err(e) => {
            println!("Error loading token: {:?}", e);
            println!("Make sure you have a .env file with DISCORD_TOKEN=your_token");
            return;
        }
    };
    
    // Set up the framework with GTA command group
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // Set the prefix to !
        .help(&MY_HELP) // Register the help command
        .group(&GTA_GROUP); // Register the GTA command group
    
    // Set up the intents
    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;
    
    println!("Creating client with command framework...");
    let mut client = match Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await {
            Ok(client) => {
                println!("Client created successfully");
                client
            },
            Err(why) => {
                println!("Error creating client: {:?}", why);
                return;
            }
        };

    println!("Starting client...");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

// Basic info command for the bot
#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, 
        "GTA Bot is a Discord bot for GTA V enthusiasts! Use `!help` to see all available commands.\n\
        \n\
        **Available Commands:**\n\
        - `!weekly` - Get this week's GTA Online bonuses and discounts\n\
        - `!status` - Check GTA Online server status\n\
        - `!vehicle [name]` - Get info about a vehicle\n\
        - `!search [term]` - Search the GTA Wiki").await?;
    Ok(())
}

// Command to show weekly updates
#[command]
async fn weekly(ctx: &Context, msg: &Message) -> CommandResult {
    // First, let the user know we're processing
    let mut processing_msg = msg.channel_id.say(&ctx.http, "Fetching the latest GTA Online weekly update...").await?;
    
    // Fetch the data
    match get_latest_weekly_update().await {
        Ok(update) => {
            // Create a nice embed message
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&update.title);
                    e.url(&update.url);
                    e.color(Colour::DARK_GREEN);
                    e.description(format!("Published: {}", update.date));
                    
                    // Add bonuses
                    if !update.bonuses.is_empty() {
                        let bonus_text = if update.bonuses.len() > 3 {
                            format!("{}\n{}\n{}\n...", 
                                update.bonuses[0], 
                                update.bonuses[1],
                                update.bonuses[2])
                        } else {
                            update.bonuses.join("\n")
                        };
                        e.field("🎁 Bonuses", bonus_text, false);
                    }
                    
                    // Add discounts
                    if !update.discounts.is_empty() {
                        let discount_text = if update.discounts.len() > 3 {
                            format!("{}\n{}\n{}\n...", 
                                update.discounts[0], 
                                update.discounts[1],
                                update.discounts[2])
                        } else {
                            update.discounts.join("\n")
                        };
                        e.field("💰 Discounts", discount_text, false);
                    }
                    
                    // Add podium vehicle if available
                    if let Some(podium) = &update.podium_vehicle {
                        e.field("🎰 Diamond Casino Podium Vehicle", podium, false);
                    }
                    
                    // Add prize ride if available
                    if let Some(prize) = &update.prize_ride {
                        e.field("🏎️ LS Car Meet Prize Ride", prize, false);
                    }
                    
                    e.footer(|f| {
                        f.text("Visit the Rockstar Newswire for complete details");
                        f
                    });
                    
                    e
                });
                m
            }).await;
            
            // Delete the processing message
            let _ = processing_msg.delete(&ctx.http).await;
        },
        Err(e) => {
            // Update the processing message with the error
            let _ = processing_msg.edit(&ctx.http, |m| {
                m.content(format!("Error fetching weekly update: {}", e))
            }).await;
        }
    }
    
    Ok(())
}

// Command to check server status
#[command]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    // First, let the user know we're processing
    let mut processing_msg = msg.channel_id.say(&ctx.http, "Checking GTA Online server status...").await?;
    
    // Fetch the status
    match check_server_status().await {
        Ok(status) => {
            // Create a status message
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("GTA Online Server Status");
                    
                    // Set color based on status
                    if status.contains("Up") {
                        e.color(Colour::DARK_GREEN);
                    } else if status.contains("Limited") {
                        e.color(Colour::GOLD);
                    } else {
                        e.color(Colour::RED);
                    }
                    
                    e.description(status);
                    e.footer(|f| {
                        f.text("Data from Rockstar Games Service Status");
                        f
                    });
                    
                    e
                });
                m
            }).await;
            
            // Delete the processing message
            let _ = processing_msg.delete(&ctx.http).await;
        },
        Err(e) => {
            // Update the processing message with the error
            let _ = processing_msg.edit(&ctx.http, |m| {
                m.content(format!("Error checking server status: {}", e))
            }).await;
        }
    }
    
    Ok(())
}

// Command to get vehicle information
#[command]
async fn vehicle(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Check if a vehicle name was provided
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "Please provide a vehicle name. Example: `!vehicle Oppressor`").await?;
        return Ok(());
    }
    
    let vehicle_name = args.rest();
    let mut processing_msg = msg.channel_id.say(&ctx.http, format!("Looking up info for '{}'...", vehicle_name)).await?;
    
    // Fetch the data
    match get_vehicle_info(vehicle_name).await {
        Ok(Some(vehicle)) => {
            // Create a nice embed message
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(&vehicle.title);
                    e.url(&vehicle.url);
                    e.color(Colour::DARK_GREEN);
                    e.description(&vehicle.description);
                    
                    // Add image if available
                    if let Some(image_url) = &vehicle.image_url {
                        e.image(image_url);
                    }
                    
                    e.footer(|f| {
                        f.text("Data from GTA Wiki");
                        f
                    });
                    
                    e
                });
                m
            }).await;
            
            // Delete the processing message
            let _ = processing_msg.delete(&ctx.http).await;
        },
        Ok(None) => {
            // Update the processing message with the error
            let _ = processing_msg.edit(&ctx.http, |m| {
                m.content(format!("Couldn't find information about '{}'", vehicle_name))
            }).await;
        },
        Err(e) => {
            // Update the processing message with the error
            let _ = processing_msg.edit(&ctx.http, |m| {
                m.content(format!("Error looking up vehicle information: {}", e))
            }).await;
        }
    }
    
    Ok(())
}

// Command to search the GTA Wiki
#[command]
async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Check if a search term was provided
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "Please provide a search term. Example: `!search Los Santos`").await?;
        return Ok(());
    }
    
    let search_term = args.rest();
    let mut processing_msg = msg.channel_id.say(&ctx.http, format!("Searching GTA Wiki for '{}'...", search_term)).await?;
    
    // Fetch the data
    match search_gta_wiki(search_term).await {
        Ok(articles) if !articles.is_empty() => {
            // Create a nice embed message
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(format!("Search Results for '{}'", search_term));
                    e.color(Colour::DARK_GREEN);
                    
                    // Add top 3 results
                    let limit = std::cmp::min(articles.len(), 3);
                    for i in 0..limit {
                        e.field(
                            &articles[i].title, 
                            format!("{}\n[Read more]({})", 
                                &articles[i].description,
                                &articles[i].url
                            ), 
                            false
                        );
                    }
                    
                    // Add image from first result if available
                    if let Some(image_url) = &articles[0].image_url {
                        e.image(image_url);
                    }
                    
                    e.footer(|f| {
                        f.text("Data from GTA Wiki");
                        f
                    });
                    
                    e
                });
                m
            }).await;
            
            // Delete the processing message
            let _ = processing_msg.delete(&ctx.http).await;
        },
        Ok(_) => {
            // Update the processing message with no results
            let _ = processing_msg.edit(&ctx.http, |m| {
                m.content(format!("No results found for '{}'", search_term))
            }).await;
        },
        Err(e) => {
            // Update the processing message with the error
            let _ = processing_msg.edit(&ctx.http, |m| {
                m.content(format!("Error searching GTA Wiki: {}", e))
            }).await;
        }
    }
    
    Ok(())
}

// Special help command just for GTA info
#[command]
#[aliases(gtahelp)]
async fn help_gta(ctx: &Context, msg: &Message) -> CommandResult {
    let response = "**GTA Bot Commands:**\n\
                    - `!info` - General bot information\n\
                    - `!weekly` - Get this week's GTA Online bonuses and discounts\n\
                    - `!status` - Check GTA Online server status\n\
                    - `!vehicle [name]` - Get info about a vehicle\n\
                    - `!search [term]` - Search the GTA Wiki\n\
                    - `!help_gta` - Show this help message";
    
    msg.channel_id.say(&ctx.http, response).await?;
    Ok(())
}

// Custom help command
#[help]
#[command_not_found_text = "Command not found: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}