use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ::serenity::all::{Colour, CreateEmbed, CreateEmbedFooter, CreateMessage, RoleId};
use serenity::User;
use std::collections::HashMap;

use super::{Context, Error, UserId, serenity, FromStr, NUMBER_REGEX};

#[poise::command(slash_command, prefix_command)]
/// Gets all possible information about the discord account.
pub async fn discordinfo(
    interaction: Context<'_>,
    #[description = "Discord user ids for the command."] users: String,
) -> Result<(), Error> {
    interaction.reply("Getting user info, please standby!").await?;
    let purified_users = NUMBER_REGEX.replace_all(users.as_str(), "");
    if purified_users.is_empty() {
        interaction.say("Command failed; no users inputted, or users improperly inputted.").await?;
        return Ok(());
    }
    let users = purified_users.split(' ');
    for snowflake in users {
        let userid: UserId = UserId::from_str(snowflake).expect("something went wrong.");
        let user: User = match userid.to_user(&interaction.http()).await {
            Ok(user) => user,
            Err(_) => {
                interaction.say(format!("An error occurred attempting to process user `{}`. Skipping user's log.", snowflake)).await?;
                continue;
            }
        };

        interaction.channel_id().say(&interaction.http(), "### User Id").await?;
        interaction.channel_id().say(&interaction.http(), format!("{}", user.id)).await?;
        interaction.channel_id().say(&interaction.http(), "### User Mention").await?;
        interaction.channel_id().say(&interaction.http(), format!(r"<\@{}>", user.id)).await?;

        let created_at_timestamp = user.created_at().unix_timestamp();
        let account_age = SystemTime::now().duration_since(UNIX_EPOCH)? - Duration::from_secs(created_at_timestamp as u64);
        let new_account_message = if account_age < Duration::from_secs(60 * 24 * 60 * 60) {
            "**Account is new, below 60 days old.**"
        } else {
            ""
        };

        let avatar_url = match user.avatar_url() {
            Some(url) => url,
            None => {
                "No URL/User has a default avatar.".to_string()
            }
        };
        let banner_url = match user.banner_url() {
            Some(url) => url,
            None => {
                "No banner.".to_string()
            }
        };

        let global_name = match user.global_name {
            Some(global_name) => global_name,
            None => {
                "No nickname set.".to_string()
            }
        };

        let footer = CreateEmbedFooter::new("Powered by RON Assista Bot").icon_url("https://cdn.discordapp.com/icons/1094323433032130613/6f89f0913a624b2cdb6d663f351ac06c.webp");
        let color = Colour::from_rgb(117, 31, 10);
        let mut first_embed = CreateEmbed::default()
            .title("Extra User Information")
            .field("Username", format!("{}",user.name), true)
            .field("Global Name", format!("{}",global_name), true)
            .field("User Creation Date", format!("<t:{}:D>\n{}", created_at_timestamp, new_account_message), true)
            .field("Avatar URL", format!("{}",avatar_url), true)
            .field("Banner URL", format!("{}",banner_url), true)
            .color(color.clone());
        let mut embeds = vec![];

            if let Some(guild_id) = interaction.guild_id() {
                if let Ok(member) = guild_id.member(&interaction.http(), userid).await {
                    let nickname = match member.clone().nick {
                        Some(nickname) => nickname,
                        None => "No nickname set.".to_string()
                    };
            
                    let mut role_permissions: HashMap<RoleId, Vec<&'static str>> = HashMap::new();
                    
                    if let Ok(guild) = guild_id.to_partial_guild(&interaction.http()).await {
                        // Get all roles, including @everyone
                        for (role_id, role) in &guild.roles {
                            let perm_names: Vec<&'static str> = role.permissions
                                .iter_names()
                                .map(|(name, _)| name)
                                .collect();
                            role_permissions.insert(*role_id, perm_names);
                        }
                    }
            
                    let mut perms_string = String::new();
                    // Handle @everyone role first
                    let everyone_role_id = guild_id.everyone_role();
                    let everyone_role_permissions = role_permissions.remove(&everyone_role_id).unwrap();
                    // Handle other roles
                    for role in &member.roles {
                        if let Some(perms) = role_permissions.remove(role) {
                            if !perms.is_empty() {
                                perms_string.push_str(&format!("<@&{}>: {}\n", role, perms.join(", ")));
                            }
                        }
                    }

                    if !everyone_role_permissions.is_empty() {
                        perms_string.push_str(&format!("@everyone: {}\n", everyone_role_permissions.join(", ")));
                    }
            
                    if perms_string.is_empty() {
                        perms_string = "No permissions found.".to_string();
                    }
            
                    let role_string = member.roles
                        .iter()
                        .map(|roleid| format!("<@&{}>", roleid))
                        .collect::<Vec<String>>()
                        .join(" ");

                    let role_embed = CreateEmbed::default()
                        .title("Guild Member Information")
                        .field("User Roles", role_string, false)
                        .field("Member Nickname", nickname, true)
                        .color(color.clone());
                    embeds.push(role_embed);
            
                    let permissions_embed = CreateEmbed::default()
                        .title("Guild Member Information")
                        .field("User Permissions", perms_string, false)
                        .footer(footer.clone())
                        .color(color.clone());
                    embeds.push(permissions_embed);
                }
            } else {
                first_embed = first_embed.field("Note", "This command was used outside of a guild context. Role and permission information is not available.", false).footer(footer.clone());
            }
        embeds.insert(0, first_embed);

        interaction.channel_id().send_message(&interaction.http(), CreateMessage::default().embeds(embeds)).await?;
    }
    Ok(())
}