use futures::future::join_all;
use serenity::all::MessageId;
use super::{Context, Error, video_convert};
use poise::serenity_prelude as serenity;
use std::sync::Arc;

#[poise::command(slash_command, prefix_command)]
/// Command for updating the bot.
pub async fn convert_video(
    ctx: Context<'_>,
    #[description = "Message ids for command."] message_ids: String,
) -> Result<(), Error> {
    ctx.reply("Converting message videos, searching thru each channel, this may take a while!").await?;

    let message_ids: Vec<u64> = message_ids
        .split_whitespace()
        .filter_map(|id| id.parse().ok())
        .collect();

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("Use this command in a guild!").await?;
            return Ok(());
        }
    };

    let http = Arc::new(ctx.http());
    let channels = http.get_channels(guild_id).await?;
    let serenity_context = ctx.serenity_context().clone();

    let futures = message_ids.into_iter().map(|message_id| {
        let channels = channels.clone();
        let http = Arc::clone(&http);
        let serenity_context = serenity_context.clone();
        async move {
            process_message(channels, http, serenity_context, MessageId::new(message_id)).await;
        }
    });

    join_all(futures).await;

    ctx.say("All videos have been processed!").await?;
    Ok(())
}

async fn process_message(
    channels: Vec<serenity::GuildChannel>,
    http: Arc<&serenity::Http>,
    serenity_context: serenity::Context,
    message_id: MessageId,
) {
    let mut message = None;

    for channel in channels {
        if let Ok(found_message) = channel.message(&http, message_id).await {
            message = Some(found_message);
            break;
        }
    }

    let message = match message {
        Some(msg) => msg,
        None => {
            println!("Couldn't find message in guild, sorry!");
            return;
        }
    };

    let futures = message.attachments.iter().map(|attachment| {
        let message = message.clone();
        let serenity_context = serenity_context.clone();
        let attachment = attachment.clone();
        async move {
            video_convert(message, serenity_context, attachment).await;
        }
    });

    join_all(futures).await;
}