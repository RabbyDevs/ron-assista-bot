use std::{env, time::{Duration, SystemTime, UNIX_EPOCH}};
use once_cell::sync::Lazy;
use regex::Regex;
use roboat::ClientBuilder;
use ::serenity::{all::{Message, MessageId, MessageType, Ready}, async_trait};
use serenity::{prelude::*, UserId};
use poise::serenity_prelude as serenity;
use std::str::FromStr;

mod helper;
mod commands;
use commands::{discord_log, probation_log, roblox_log, role_log, get_info};

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
    async fn message(&self, ctx: serenity::prelude::Context, msg: Message) {
        let regular = MessageType::Regular;
        let member = match msg.member(&ctx).await {
            Ok(member) => {
                member
            },
            Err(_) => {return;},   
        };
        // if msg.channel_id == 825755301981978685 {
        //     let start = SystemTime::now();
        //     let since_the_epoch = start
        //         .duration_since(UNIX_EPOCH)
        //         .expect("Time went backwards");
        //     let epoch_in_s = since_the_epoch.as_secs();
        // }
    }

    // async fn message_delete(&self, ctx: serenity::prelude::Context, channelid: serenity::all::ChannelId, msg_id: MessageId, _: Option<serenity::all::GuildId>) {

    // }
    // async fn reaction_add(&self, ctx: serenity::prelude::Context, add_reaction: serenity::Reaction) {
    //     if add_reaction.member.unwrap().permissions(&ctx).unwrap().administrator() {

    //     }
    // }
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


