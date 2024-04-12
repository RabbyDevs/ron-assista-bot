use std::{env, time::Duration};
use once_cell::sync::Lazy;
use roboat::ClientBuilder;
use ::serenity::{all::{Message, Ready}, async_trait};
use tokio::time::sleep as tokio_sleep;
use serenity::{prelude::*, UserId};
use poise::serenity_prelude as serenity;
use std::str::FromStr;
use serenity::builder::CreateMessage;

mod helper;
mod commands;
use commands::{discord_log, probation_log, roblox_log, role_log, get_info};

static_toml::static_toml! {
    static CONFIG = include_toml!("config.toml");
}
static RBX_CLIENT: Lazy<roboat::Client> = Lazy::new(|| ClientBuilder::new().build());
static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| reqwest::Client::new());

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn message(&self, ctx: serenity::prelude::Context, msg: Message) {
        if msg.content.len() == 0 && msg.attachments.len() == 0 && msg.sticker_items.len() == 0 && msg.member(&ctx).await.expect("member err").permissions(&ctx).expect("err").moderate_members() == false {
            if msg.author.bot == false {
                msg.delete(&ctx).await.expect("err deleting msg");
                let test_msg = msg.channel_id.send_message(&ctx, CreateMessage::new().content(format!("{} sending polls is not allowed!", msg.author.mention()))).await.expect("err sending msg");
                tokio_sleep(Duration::from_secs(5)).await;
                test_msg.delete(ctx).await.expect("err");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let discord_api_key = CONFIG.discord_api_key;
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_TYPING
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![discord_log::discordlog(), roblox_log::robloxlog(), probation_log::probationlog(), role_log::rolelog(), get_info::getinfo()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(discord_api_key, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("client start err");
    client.start().await.unwrap();
}


