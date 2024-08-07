#![feature(async_closure)]
use std::{env, str::FromStr, vec};
use once_cell::sync::Lazy;
use regex::Regex;
use roboat::ClientBuilder;
use ::serenity::all::{GuildId, Member, RoleId, User};
use serenity::{all::{ActivityData, OnlineStatus, Ready}, async_trait};
use serenity::{prelude::*, UserId};
use poise::serenity_prelude as serenity;
use reqwest::Client;

mod main_modules;
use main_modules::{helper::{self, video_convert, video_format_changer, image_to_png_converter, video_to_gif_converter, png_to_gif_converter}, timer::TimerSystem};
mod commands;
use commands::{
    video_module::{
        convert_video,
        convert_gif
    },
    log_module::{
        discord_log, 
        false_infraction, 
        probation_log, 
        roblox_log, 
        role_log
    },
    info_module::{
        discord_info,
        get_info
    },
    time_module::timed_role, 
    update
};

static_toml::static_toml! {
    static CONFIG = include_toml!("config.toml");
}
static RBX_CLIENT: Lazy<roboat::Client> = Lazy::new(|| ClientBuilder::new().build());
static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());
static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::from_str(r"[^\d\s]").expect("err"));

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

static mut TIMER_SYSTEM: Lazy<TimerSystem> = Lazy::new(|| TimerSystem::new("probation_role").unwrap());

struct Handler;

static DODGED_FILE_FORMATS: Lazy<Vec<String>> = Lazy::new(|| vec!["video/mp4".to_string(), "video/webm".to_string(), "video/quicktime".to_string()]);
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    
        unsafe { 
            TIMER_SYSTEM.set_event_handler(move |user_id: String, role_id: String| {
            let ctx = ctx.clone();
            Box::pin(async move {
                let user_id = UserId::from_str(user_id.as_str()).expect("Invalid user ID");
                let role_id = RoleId::from_str(role_id.as_str()).expect("Invalid role ID");

                // Fetch the guilds the bot is in
                let guilds = ctx.cache.guilds();

                // Find the guild and role
                for guild_id in guilds {
                    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
                        if let Ok(member) = guild.member(&ctx.http, user_id).await {
                            match member.remove_role(&ctx.http, role_id).await {
                                Ok(()) => (),
                                Err(err) => println!("Couldnt remove role from user in {}, {}", guild_id, err)
                            };
                        }
                    }
                }
            })
        }).await;
    
        TIMER_SYSTEM.start_timer_thread();
    }
    }

    async fn message(&self, ctx: serenity::prelude::Context, new_message: serenity::all::Message) {
        if new_message.attachments.is_empty() {
            return;
        }

        for attachment in &new_message.attachments {
            let Some(content_type) = &attachment.content_type else { continue };
            if !content_type.contains("video/") || DODGED_FILE_FORMATS.contains(content_type) {
                continue;
            }

            let new_message = new_message.clone();
            let attachment = attachment.clone();
            let ctx = ctx.clone();
            tokio::spawn(async move {
                video_convert(new_message, ctx, attachment).await;
            });
        }
    }

    async fn guild_member_addition(&self, ctx: serenity::prelude::Context, new_member: Member) {
        unsafe {
            match TIMER_SYSTEM.resume_timer(new_member.user.id.to_string().as_str()).await {
                Ok(role_id) => {
                    new_member.add_role(&ctx.http, RoleId::new(role_id.parse::<u64>().unwrap())).await.unwrap();
                    ()
                },
                Err(_) => {
                    ()
                }
            };}
        ()
    }
    async fn guild_member_removal(&self, _ctx: serenity::prelude::Context, _guild_id: GuildId, user: User, _: Option<Member>) {
        unsafe {match TIMER_SYSTEM.pause_timer(user.id.to_string().as_str()).await {
            Ok(()) => {()},
            Err(_) => {()}
        };}
        ()
    }
}

#[tokio::main]
async fn main() {
    std::fs::create_dir_all("./tmp").unwrap();
    let discord_api_key = &CONFIG.main.discord_api_key;
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
            commands: vec![
                discord_log::discordlog(), 
                roblox_log::robloxlog(), 
                probation_log::probationlog(), 
                role_log::rolelog(), 
                get_info::getinfo(), 
                update::update(), 
                discord_info::discordinfo(), 
                timed_role::timed_role(), 
                false_infraction::false_infraction(),
                convert_video::convert_video(),
                convert_gif::convert_gif()
            ],
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
