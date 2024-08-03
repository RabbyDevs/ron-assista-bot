use std::env;
use once_cell::sync::Lazy;
use regex::Regex;
use roboat::ClientBuilder;
use ::serenity::{all::{ActivityData, OnlineStatus, Ready}, async_trait};
use serenity::{prelude::*, UserId};
use poise::serenity_prelude as serenity;
use std::str::FromStr;

mod helper;
mod commands;
use commands::{discord_info, discord_log, get_info, probation_log, roblox_log, role_log, update};

static_toml::static_toml! {
    static CONFIG = include_toml!("config.toml");
}
static RBX_CLIENT: Lazy<roboat::Client> = Lazy::new(|| ClientBuilder::new().build());
static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| reqwest::Client::new());
static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::from_str(r"[^\d\s]").expect("err"));

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let discord_api_key = CONFIG.main.discord_api_key;
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
            commands: vec![discord_log::discordlog(), roblox_log::robloxlog(), probation_log::probationlog(), role_log::rolelog(), get_info::getinfo(), update::update(), discord_info::discordinfo()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            let activity = ActivityData::custom(format!("Running on v{}!", env!("CARGO_PKG_VERSION")));
            let status = OnlineStatus::Online;

            ctx.set_presence(Some(activity), status);
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