use super::{Context, Error, helper, UserId, Mentionable, serenity, FromStr, NUMBER_REGEX, TIMER_SYSTEM};

#[poise::command(slash_command, prefix_command)]
/// Assigns a role, temporarily, based on the parameters inputted.
pub async fn timed_role(
    ctx: Context<'_>,
    #[description = "Users for the command, only accepts Discord ids."] users: String,
    #[description = "Type of infraction."] role: serenity::model::guild::Role,
    #[description = "Duration of the probation (e.g., '1h', '2d', '1w'). Use 'delete' to remove the timer."] duration: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let purified_users = NUMBER_REGEX.replace_all(users.as_str(), "");
    if purified_users.is_empty() {
        ctx.say("Command failed; no users inputted, or users improperly inputted.").await?;
        return Ok(());
    }

    let users: Vec<UserId> = purified_users
        .split_whitespace()
        .filter_map(|id| UserId::from_str(id).ok())
        .collect();

    let guild_id = ctx.guild_id().ok_or("Command must be used in a guild")?;

    if duration.to_lowercase() == "delete" {
        // Delete the timer
        for user_id in users {
            unsafe {
                match TIMER_SYSTEM.delete_timer(&user_id.to_string()).await {
                    Ok(()) => {
                        // Remove the role from the user
                        if let Err(e) = ctx.http().remove_member_role(guild_id, user_id, role.id, None).await {
                            ctx.say(format!("Failed to remove role from user, but removed timer from user {}: {}", user_id, e)).await?;
                        } else {
                            ctx.say(format!("Timer deleted and role removed for user {}", user_id.mention())).await?;
                        }
                    },
                    Err(e) => {
                        ctx.say(format!("Failed to delete timer for user {}: {}", user_id, e)).await?;
                    }
                }
            }
        }
    } else {
        // Add new timer
        let (current_time, unix_timestamp, timestamp_string) = match helper::duration_conversion(duration).await {
            Ok(result) => result,
            Err(err) => {
                ctx.say(format!("Error processing duration: {}", err)).await?;
                return Ok(());
            }
        };

        let duration_secs = unix_timestamp - current_time;

        for user_id in users {
            unsafe {
                if let Err(e) = TIMER_SYSTEM.add_timer(user_id.to_string(), role.id.to_string(), duration_secs, false, None) {
                    ctx.say(format!("Failed to add timer for user {}: {}", user_id, e)).await?;
                    continue;
                }

                if let Err(e) = ctx.http().add_member_role(guild_id, user_id, role.id, None).await {
                    ctx.say(format!("Failed to add role to user {}, Timer added but paused: {}", user_id, e)).await?;
                    TIMER_SYSTEM.pause_timer(&user_id.to_string()).await.unwrap_or_else(|e| {
                        println!("Failed to pause timer for user {}: {}", user_id, e);
                    });
                    continue;
                }
                ctx.say(format!("Role timer {} added for user {} for {}", role.id, user_id.mention(), timestamp_string)).await?;
            }
        }
    }

    Ok(())
}