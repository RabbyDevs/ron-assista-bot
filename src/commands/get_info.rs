use chrono::{DateTime, Local};
use regex::Regex;

use super::{Context, Error, helper, FromStr, RBX_CLIENT, CONFIG, NUMBER_REGEX};

#[poise::command(slash_command, prefix_command)]
pub async fn getinfo(
    interaction: Context<'_>,
    #[description = "Roblox Usernames for the command, seperate with spaces."] roblox_users: Option<String>,
    #[description = "Roblox IDs for the command, seperate with spaces."] roblox_ids: Option<String>,
    #[description = "Discord IDs for the command, seperate with spaces."] discord_ids: Option<String>,
    #[description = "How many badge pages should the command get?"] badge_max_iterations: Option<i64>,
) -> Result<(), Error> {
    interaction.reply("Finding user info, standby!").await?;
    let new_line_regex = Regex::from_str("/(?:\r?\n){4,}/gm").expect("regex err");
    let default_iterations: i64 = CONFIG.default_badge_iterations;

    let roblox_users: Vec<String> = roblox_users.unwrap_or_default().split(' ').map(str::to_string).collect::<Vec<String>>();
    let purified_users = NUMBER_REGEX.replace_all(discord_ids.unwrap_or_default().as_str(), "").to_string();
    let discord_ids = purified_users.split(' ').map(str::to_string).collect::<Vec<String>>();
    let purified_roblox_ids = NUMBER_REGEX.replace_all(roblox_ids.unwrap_or_default().as_str(), "").to_string();
    let mut roblox_ids = purified_roblox_ids.split(' ').map(str::to_string).collect::<Vec<String>>();
    let bad_ones_to_remove = roblox_ids.iter().position(|x| *x == "").unwrap();
    roblox_ids.remove(bad_ones_to_remove);
    let roblox_conversion_errors;
    (roblox_ids, roblox_conversion_errors) = helper::merge_types(roblox_users, discord_ids, roblox_ids).await;
    if roblox_ids.is_empty() {
        interaction.channel_id().say(interaction, "Command failed; every user was converted and command had no valid users, meaning you might have inputted the users incorrectly...").await?;
        return Ok(());
    }

    for error in roblox_conversion_errors {
        interaction.channel_id().say(interaction, error).await?;
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
                    (0, 0, 0, String::new())
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
<t:{}:D>
\- Friend Count -
{}
\- Group Count -
{}"#, user_details.id, sanitized_description, user_details.display_name, created_at_timestamp, friend_count, group_count);
        if badge_iterations > default_iterations {response = format!("{}\nGetting badge info with more than {} (default, recommended) iterations, *this might take longer than usual.*", response, default_iterations);}
        channel.say(interaction, response).await?;
        
        let (badge_count, win_rate, welcome_badge_count, awarders_string) = badge_data.await?;
        channel.say(interaction, format!(r#"\- Badge Info -
- Badge Count: {}
- Average Win Rate: {}%
- Welcome Badge Count: {}
- Top Badge Givers for User: {}"#, badge_count, win_rate, welcome_badge_count, awarders_string)).await?;
    }
    Ok(())
}