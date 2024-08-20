#![feature(async_closure)]
use std::{env, io::Write, str::FromStr, sync::{Arc, Mutex}, vec};
use once_cell::sync::Lazy;
use regex::Regex;
use roboat::ClientBuilder;
use ::serenity::all::{ChannelId, Color, CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage, GuildId, Member, MessageId, Reaction, ReactionType, RoleId, User};
use serenity::{all::{ActivityData, OnlineStatus, Ready}, async_trait};
use serenity::{prelude::*, UserId};
use poise::serenity_prelude as serenity;
use reqwest::Client;

mod main_modules;
use main_modules::{helper, media::{video_convert, video_format_changer, video_to_gif_converter, image_to_png_converter, png_to_gif_converter, QualityPreset, apply_mask}, timer::TimerSystem, deleted_attachments::{self, AttachmentStoreDB, AttachmentStore}};
mod commands;
use commands::{
    media_module::{
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

async fn do_image_logging(ctx: serenity::prelude::Context, deleting_message: serenity::all::MessageId, guild_id: Option<GuildId>) {
    unsafe {
        let db_entry = match ATTACHMENT_DB.lock().unwrap().get(deleting_message.to_string().as_str()) {
            Some(entry) => entry,
            None => {
                return;
            }
        };

        for attachment in db_entry.attachments {
            let ctx = ctx.clone();
            let guild_id = guild_id.clone();
            tokio::spawn(async move {
                if guild_id.is_some() && guild_id.unwrap().to_string() == CONFIG.modules.logging.guild_id.to_string() {
                    let log_channel_id = ChannelId::new(CONFIG.modules.logging.logging_channel_id.parse::<u64>().unwrap());
                    let output_filename = format!("./tmp/{}", attachment.filename);
                    let response = REQWEST_CLIENT.get(&attachment.url).send().await.unwrap();
                    let bytes = response.bytes().await.unwrap();
                    let mut file = std::fs::File::create(&output_filename).expect("Failed to create input file");
                    file.write_all(&bytes).expect("Failed to write input file");
                    drop(file);
                    let attachment = CreateAttachment::file(&tokio::fs::File::open(&output_filename).await.unwrap(), &attachment.filename).await.unwrap();
                    let footer = CreateEmbedFooter::new("Made by RabbyDevs, with 🦀 and ❤️.")
                        .icon_url("https://cdn.discordapp.com/icons/1094323433032130613/6f89f0913a624b2cdb6d663f351ac06c.webp");
                    let embed = CreateEmbed::new().title("Attachment Log")
                        .field("User", format!("<@{}> - {}", db_entry.user_id, db_entry.user_id), false)
                        .field("Sent on", format!("<t:{}>", db_entry.created_at.unix_timestamp()), false)
                        .color(Color::from_rgb(98,32,7))
                        .footer(footer);
                    log_channel_id.send_message(&ctx.http, CreateMessage::new().add_embed(embed).add_file(attachment)).await.unwrap();
                    std::fs::remove_file(output_filename).unwrap();
                };
            });
        }

        ATTACHMENT_DB.lock().unwrap().delete(deleting_message.to_string().as_str()).unwrap();
    }
}

#[derive(Debug)]
pub struct LoggingQueue {
    pub message_id: MessageId
}

impl LoggingQueue {
    pub async fn do_image_logging(
        &self,
        ctx: &serenity::prelude::Context,
        deleting_message: serenity::all::MessageId,
        guild_id: Option<GuildId>,
    ) {
        do_image_logging(ctx.clone(), deleting_message, guild_id).await;
    }
}

static mut TIMER_SYSTEM: Lazy<TimerSystem> = Lazy::new(|| TimerSystem::new("probation_role").unwrap());
static mut ATTACHMENT_DB: Lazy<Arc<Mutex<AttachmentStoreDB>>> = Lazy::new(|| AttachmentStoreDB::get_instance());
static mut QUEUED_LOGGING: Lazy<Vec<LoggingQueue>> = Lazy::new(||vec![]);

struct Handler;

static DODGED_FILE_FORMATS: Lazy<Vec<String>> = Lazy::new(|| vec!["video/mp4".to_string(), "video/webm".to_string(), "video/quicktime".to_string()]);

async fn reaction_logging(
    ctx: serenity::prelude::Context, 
    event_type: &str, 
    user_id: Option<UserId>, 
    channel_id: ChannelId, 
    message_id: MessageId, 
    guild_id: Option<GuildId>, 
    emoji: Option<&ReactionType>
) {
    let log_channel_id = ChannelId::new(CONFIG.modules.logging.logging_channel_id.parse().unwrap());
    let mut embed_builder = CreateEmbed::new();
    
    let emoji_url = match emoji {
        Some(ReactionType::Custom { animated, id, .. }) => {
            let extension = if *animated { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/emojis/{}.{}", id, extension)
        },
        Some(ReactionType::Unicode(_)) => String::new(),
        _ => String::new(),
    };

    let title = match event_type {
        "add" => "Reaction Added",
        "remove" => "Reaction Removed",
        "remove_all" => "All Reactions Removed",
        "remove_emoji" => "Emoji Removed",
        _ => "Reaction Event",
    };

    embed_builder = embed_builder
        .color(Color::from_rgb(98,32,7))
        .title(title)
        .field("Channel", channel_id.mention().to_string(), true)
        .field("Message", format!("[Jump to Message]({})", message_id.link(channel_id, guild_id)), false);

    if let Some(emoji) = emoji {
        embed_builder = embed_builder.field("Emoji", emoji.to_string(), false);
    }

    if let Some(user_id) = user_id {
        embed_builder = embed_builder.field("Original User", user_id.mention().to_string(), true);
    }

    if !emoji_url.is_empty() {
        embed_builder = embed_builder.thumbnail(emoji_url);
    }

    if let Err(why) = log_channel_id.send_message(&ctx.http, CreateMessage::new().add_embed(embed_builder)).await {
        eprintln!("Error sending log message: {:?}", why);
    }
}


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
        if new_message.channel_id.to_string() == CONFIG.modules.logging.cdn_channel_id.to_string() || new_message.channel_id.to_string() == CONFIG.modules.logging.logging_channel_id.to_string() {
            return;
        }
        if new_message.attachments.is_empty() {
            return;
        }

        let message = CreateMessage::new();
        let mut files = vec![];
        for attachment in &new_message.attachments {
            let output_filename = format!("./tmp/{}", attachment.filename);
            let response = REQWEST_CLIENT.get(&attachment.url).send().await.unwrap();
            let bytes = response.bytes().await.unwrap();
            let mut file = std::fs::File::create(&output_filename).expect("Failed to create input file");
            file.write_all(&bytes).expect("Failed to write input file");
            drop(file);
            files.push(CreateAttachment::file(&tokio::fs::File::open(&output_filename).await.unwrap(), &attachment.filename).await.unwrap());
            std::fs::remove_file(&output_filename).unwrap();
        }
        let log_channel_id = ChannelId::new(CONFIG.modules.logging.cdn_channel_id.parse::<u64>().unwrap());
        let final_msg = log_channel_id.send_message(&ctx.http, message.add_files(files)).await.unwrap();
        let user_id = new_message.author.id;
        let attachments = final_msg.attachments;
        let created_at = new_message.id.created_at();
        let message_id = new_message.id;
        let store = AttachmentStore {
            message_id,
            attachments,
            created_at,
            user_id
        };

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

        unsafe { ATTACHMENT_DB.lock().unwrap().save(&store).unwrap(); }

        unsafe {
            let message_id = new_message.id;
            for (i, log) in QUEUED_LOGGING.iter().enumerate() {
                if log.message_id == message_id {
                    log.do_image_logging(&ctx, message_id, new_message.guild_id).await;
                    QUEUED_LOGGING.remove(i);
                }
            }
        }
    }

    async fn message_delete(&self, ctx: serenity::prelude::Context, channel_id: ChannelId, deleting_message: serenity::all::MessageId, guild_id: Option<GuildId>) { 
        if channel_id.to_string() == CONFIG.modules.logging.cdn_channel_id.to_string() {
            return;
        }
        unsafe {
            match ATTACHMENT_DB.lock().unwrap().get(deleting_message.to_string().as_str()) {
                Some(entry) => entry,
                None => {
                    let message_id = deleting_message;
                    QUEUED_LOGGING.push(LoggingQueue {
                        message_id
                    });
                    return;
                }
            };
            do_image_logging(ctx, deleting_message, guild_id).await;
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

    async fn reaction_add(&self, ctx: serenity::prelude::Context, add_reaction: Reaction) {
        reaction_logging(
            ctx, 
            "add", 
            Some(add_reaction.user_id.unwrap()), 
            add_reaction.channel_id, 
            add_reaction.message_id, 
            add_reaction.guild_id, 
            Some(&add_reaction.emoji)
        ).await;
    }

    async fn reaction_remove(&self, ctx: serenity::prelude::Context, remove_reaction: Reaction) {
        reaction_logging(
            ctx, 
            "remove", 
            Some(remove_reaction.user_id.unwrap()), 
            remove_reaction.channel_id, 
            remove_reaction.message_id, 
            remove_reaction.guild_id, 
            Some(&remove_reaction.emoji)
        ).await;
    }

    async fn reaction_remove_all(&self, ctx: serenity::prelude::Context, channel_id: ChannelId, removed_from_message_id: MessageId) {
        reaction_logging(
            ctx, 
            "remove_all", 
            None, 
            channel_id, 
            removed_from_message_id, 
            None, 
            None
        ).await;
    }

    async fn reaction_remove_emoji(&self, ctx: serenity::prelude::Context, removed_reaction: Reaction) {
        reaction_logging(
            ctx, 
            "remove_emoji", 
            None, 
            removed_reaction.channel_id, 
            removed_reaction.message_id, 
            removed_reaction.guild_id, 
            Some(&removed_reaction.emoji)
        ).await;
    }
}

#[tokio::main]
async fn main() {
    deleted_attachments::start_attachment_db();
    std::fs::create_dir_all("./tmp").unwrap();
    let discord_api_key = &CONFIG.main.discord_api_key;
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_TYPING
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

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
                convert_gif::gif()
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
