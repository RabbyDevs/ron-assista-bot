use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local};
use regex::Regex;
use serenity::all::EditMessage;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use serenity::model::colour::Colour;
use serenity::builder::CreateMessage;
use super::{Context, Error, helper, FromStr, RBX_CLIENT, CONFIG};

#[poise::command(slash_command, prefix_command)]
/// Gets the ROBLOX info of the users inputted. Do not input Discord IDs as a test, please.
pub async fn getinfo(
    interaction: Context<'_>,
    #[description = "Users for the command, accepts Discord ids, ROBLOX users and ROBLOX ids."] users: String,
    #[description = "How many badge pages should the command get?"] badge_max_iterations: Option<i64>,
) -> Result<(), Error> {
    interaction.reply("Getting user info, please standby!").await?;
    let new_line_regex = Regex::new(r"(?:\r?\n){4,}").expect("Invalid regex");
    let badge_iterations = badge_max_iterations.unwrap_or(CONFIG.main.default_badge_iterations);

    let users: Vec<String> = users.split_whitespace().map(str::to_string).collect();
    let (roblox_ids, roblox_conversion_errors) = helper::merge_types(users).await;

    if !roblox_conversion_errors.is_empty() {
        interaction.channel_id().say(&interaction.http(), &roblox_conversion_errors.join("\n")).await?;
    }

    if roblox_ids.is_empty() {
        interaction.say("Command failed; no valid users were found. You might have inputted the users incorrectly.").await?;
        return Ok(());
    }

    for id in roblox_ids {
        let badge_data_future = helper::badge_data(id.clone(), badge_iterations);
        let friend_count_future = helper::roblox_friend_count(&id);
        let group_count_future = helper::roblox_group_count(&id);
        if id.is_empty() { continue; }

        let user_details = RBX_CLIENT.user_details(id.parse::<u64>().expect("Invalid user ID")).await?;

        interaction.channel_id().say(&interaction.http(), "### Username").await?;
        interaction.channel_id().say(&interaction.http(), format!("{}", user_details.username)).await?;
        interaction.channel_id().say(&interaction.http(), "### User ID").await?;
        interaction.channel_id().say(&interaction.http(), format!("{}", user_details.id)).await?;

        let avatar_image = helper::get_roblox_avatar_bust(user_details.id.to_string()).await;
        // Prepare initial embed with basic info
        let footer = CreateEmbedFooter::new("Made by RabbyDevs, with 🦀 and ❤️.")
        .icon_url("https://cdn.discordapp.com/icons/1094323433032130613/6f89f0913a624b2cdb6d663f351ac06c.webp");
        let mut embed = CreateEmbed::default()
            .title("Extra ROBLOX Information")
            .color(Colour::from_rgb(98,32,7))
            .footer(footer)
            .thumbnail(format!("{}", avatar_image.as_str()))
            .field("User Link", format!("https://roblox.com/users/{}", user_details.id), false);

        let sanitized_description = new_line_regex.replace(&user_details.description, "").into_owned();
        let created_at: DateTime<Local> = DateTime::from_str(&user_details.created_at).expect("Invalid date");
        let created_at_timestamp = created_at.timestamp();

        let account_age = SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(created_at_timestamp as u64);
        let new_account_message = if account_age < Duration::from_secs(60 * 24 * 60 * 60) {
            ", **Account is new, below 60 days old.**"
        } else {
            ""
        };

        embed = embed
            .field("Display Name", user_details.display_name, false)
            .field("Description", sanitized_description.is_empty().then(|| "No description found.").unwrap_or(&sanitized_description), false)
            .field("Account Creation", format!("<t:{}:D>{}", created_at_timestamp, new_account_message), false);

        let mut init_message = interaction.channel_id().send_message(&interaction.http(), CreateMessage::new().add_embed(embed.clone())).await?;
        
        // Execute friend count first and update the embed
        if let Ok(friend_count) = friend_count_future.await {
            embed = embed.field("Friend Count", friend_count.to_string(), false);
            init_message.edit(&interaction.http(), EditMessage::new().add_embed(embed.clone())).await?;
        }
        
        // Execute group count next and update the embed
        if let Ok(group_count) = group_count_future.await {
            embed = embed.field("Group Count", group_count.to_string(), false);
            init_message.edit(&interaction.http(), EditMessage::new().add_embed(embed.clone())).await?;
        }
        
        // Execute badge data last
        if let Ok((badge_count, win_rate, awarders_string)) = badge_data_future.await {
            embed = embed
                .field("Badge Count", badge_count.to_string(), false)
                .field("Average Win Rate", format!("{:.3}%", win_rate), false)
                .field("Top Badge Givers", awarders_string, false);
            init_message.edit(&interaction.http(), EditMessage::new().add_embed(embed.clone())).await?;
        }

        init_message.edit(&interaction.http(), EditMessage::new().add_embed(embed.clone())).await?;
    }

    Ok(())
}