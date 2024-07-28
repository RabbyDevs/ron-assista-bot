use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local};
use regex::Regex;

use super::{Context, Error, helper, FromStr, RBX_CLIENT, CONFIG};

#[poise::command(slash_command, prefix_command)]
/// Gets the ROBLOX info of the users inputted. Do not input Discord IDs as a test, please.
pub async fn getinfo(
    interaction: Context<'_>,
    #[description = "Users for the command, accepts Discord ids, ROBLOX users and ROBLOX ids."] users: String,
    #[description = "How many badge pages should the command get?"] badge_max_iterations: Option<i64>,
) -> Result<(), Error> {
    interaction.reply("Finding user info, standby!").await?;
    let new_line_regex = Regex::from_str("/(?:\r?\n){4,}/gm").expect("regex err");
    let default_iterations: i64 = CONFIG.default_badge_iterations;

    let users: Vec<String> = users.split(' ').map(str::to_string).collect::<Vec<String>>();
    let roblox_ids: Vec<String>;
    let roblox_conversion_errors: Vec<String>;
    (roblox_ids, roblox_conversion_errors) = helper::merge_types(users).await;
    for error in roblox_conversion_errors {
        interaction.channel_id().say(interaction, error).await?;
    }
    if roblox_ids.is_empty() {
        interaction.channel_id().say(interaction, "Command failed; every user was converted and no valid users were found, meaning you might have inputted the users incorrectly...").await?;
        return Ok(());
    }
    let iterations_exists = badge_max_iterations.is_some();
    let channel = interaction.channel_id();
    let badge_iterations: i64 = if iterations_exists {badge_max_iterations.unwrap()} else {default_iterations};

    for id in roblox_ids {
        if id.is_empty() {continue}
        let mut badge_errors: Vec<String> = Vec::new();
        let id_for_badges = id.clone();
        let badge_data = tokio::spawn(async move {
            match helper::badge_data(id_for_badges.clone(), badge_iterations).await {
                Ok(data) => data,
                Err(_) => {
                    badge_errors.push(format!("Something went wrong when getting badges for user {}", id_for_badges));
                    (0, 0.0, String::new())
                }
            }
        });
        let id_for_friends = id.clone();
        let friend_count = tokio::spawn(async move {
            helper::roblox_friend_count(id_for_friends).await
        });
        let id_for_groups = id.clone();
        let group_count = tokio::spawn(async move {
            helper::roblox_group_count(id_for_groups).await
        });

        let user_details = RBX_CLIENT.user_details(id.parse::<u64>().expect("u64 err")).await?;
        let description = user_details.description;
        let mut sanitized_description = new_line_regex.replace(description.as_str(), "").to_string();
        let created_at: DateTime<Local> = DateTime::from_str(user_details.created_at.as_str()).expect("err");
        let created_at_timestamp = created_at.timestamp();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Error getting current time: {}", e))?;

        let difference = now - Duration::from_secs(created_at_timestamp as u64);
        
        let new_account = difference < Duration::from_secs(60 * 24 * 60 * 60);

        let mut new_account_message = "";
        if new_account {
            new_account_message = ", **Account is new, below 60 days old.**"
        }

        channel.say(interaction, "\\- Username -").await?;
        channel.say(interaction, format!("{}", user_details.username)).await?;
        channel.say(interaction, "\\- User ID -").await?;
        channel.say(interaction, format!("{}", user_details.id)).await?;
        let friend_count = friend_count.await?;
        let group_count = group_count.await?;
        sanitized_description = if sanitized_description.is_empty() {"No description found.".to_string()} else {sanitized_description};
        let mut response = format!(r#"\- Profile Link -
https://roblox.com/users/{}
\- Description -
{}
\- Display Name -
{}
\- Account Creation Date -
<t:{}:D>{}
\- Friend Count -
{}
\- Group Count -
{}"#, user_details.id, sanitized_description, user_details.display_name, created_at_timestamp, new_account_message, friend_count, group_count);
        if badge_iterations > default_iterations {response = format!("{}\nGetting badge info with more than {} (default, recommended) iterations, *this might take longer than usual.*", response, default_iterations);}
        channel.say(interaction, response).await?;
        
        let (badge_count, win_rate, awarders_string) = badge_data.await?;
        channel.say(interaction, format!(r#"\- Badge Info -
- Badge Count: {}
- Average Win Rate: {:.3}%
- Top Badge Givers for User: {}"#, badge_count, win_rate, awarders_string)).await?;
    }
    Ok(())
}