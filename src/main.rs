use std::{env, io::Write, process::Command, str::FromStr, vec};
use once_cell::sync::Lazy;
use regex::Regex;
use roboat::ClientBuilder;
use ::serenity::all::EditMessage;
use serenity::{all::{ActivityData, OnlineStatus, Ready}, async_trait};
use serenity::{prelude::*, UserId};
use poise::serenity_prelude as serenity;
use reqwest::Client;

mod helper;
mod commands;
use commands::{discord_info, discord_log, get_info, probation_log, roblox_log, role_log, update};

static_toml::static_toml! {
    static CONFIG = include_toml!("config.toml");
}
static RBX_CLIENT: Lazy<roboat::Client> = Lazy::new(|| ClientBuilder::new().build());
static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());
static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::from_str(r"[^\d\s]").expect("err"));

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;
use uuid::Uuid;

static DODGED_FILE_FORMATS: Lazy<Vec<String>> = Lazy::new(|| vec!["video/mp4".to_string(), "video/webm".to_string(), "video/quicktime".to_string()]);
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
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
                let mut msg = new_message.reply_ping(&ctx.http, format!("Converting {} to MP4!", attachment.filename)).await.unwrap();
                let input_filename = format!("./tmp/input_{}.tmp", Uuid::new_v4());
                let output_filename = format!("./tmp/output_{}.mp4", Uuid::new_v4());

                // Download the file
                let response = REQWEST_CLIENT.get(&attachment.url).send().await.unwrap();
                let bytes = response.bytes().await.unwrap();
                let mut file = std::fs::File::create(&input_filename).expect("Failed to create input file");
                file.write_all(&bytes).expect("Failed to write input file");

                // Convert the video using FFmpeg
                let output = Command::new("ffmpeg")
                    .args(&[
                        "-i", &input_filename,
                        "-c:v", "libx264",
                        "-preset", "medium",
                        "-crf", "23",
                        "-c:a", "aac",
                        "-b:a", "128k",
                        &output_filename
                    ])
                    .output()
                    .expect("Failed to execute FFmpeg command.");

                if output.status.success() {
                    let file = serenity::all::CreateAttachment::path(&output_filename).await.unwrap();
                    let build = EditMessage::new().new_attachment(file).content("Done!");
                    msg.edit(&ctx.http, build).await.unwrap();
                } else {
                    println!("FFmpeg conversion failed: {:?}", String::from_utf8_lossy(&output.stderr));
                    let _ = new_message.channel_id.say(&ctx.http, "Failed to convert the video.").await;
                }

                let _ = std::fs::remove_file(&input_filename);
                let _ = std::fs::remove_file(&output_filename);
            });
        }
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
