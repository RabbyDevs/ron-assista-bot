use poise::ChoiceParameter;
use super::{Context, Error, UserId, Mentionable, FromStr};

#[derive(Debug, poise::ChoiceParameter)]
pub enum FalseInfTypes {
    #[name = "Discord, Temporary Ban"]
    Ban,
    #[name = "Discord, Temporary Ban"]
    TempBan,
    #[name = "Discord, Kick"]
    Kick,
    #[name = "Discord, Mute"]
    Mute,
    #[name = "Discord, Warn"]
    Warn,
    #[name = "Game, Ban"]
    GameBan,
    #[name = "Game, Serverban"]
    GameServerBan,
    #[name = "Game, Kick"]
    GameKick,
    #[name = "Game, Warn"]
    GameWarn
}

#[poise::command(slash_command, prefix_command)]
/// Makes a Discord infraction log based on the Discord IDs inputted.
pub async fn false_infraction(
    ctx: Context<'_>,
    #[description = "Type of infraction."] #[rename = "type"] infraction_type: FalseInfTypes,
    #[description = "Moderator users (space-separated IDs)"] mod_users: String,
    #[description = "Affected users (space-separated IDs)"] affected_users: String,
    #[description = "Reason for invalidation."] reason: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let mod_ids: Vec<UserId> = mod_users
        .split_whitespace()
        .filter_map(|id| UserId::from_str(id).ok())
        .collect();

    let mut affected_ids: Vec<UserId> = affected_users
        .split_whitespace()
        .filter_map(|id| UserId::from_str(id).ok())
        .collect();

    if mod_ids.is_empty() || affected_ids.is_empty() {
        ctx.say("Error: No valid user IDs provided for moderators or affected users.").await?;
        return Ok(());
    }

    let mut affected_iter = 0;

    for (index, mod_id) in mod_ids.iter().enumerate() {
        if index + 1 == mod_ids.len() {
            let mut response = format!("[{}]\n[{}:{}]", infraction_type.name(), mod_id.to_string(), mod_id.to_user(&ctx.http()).await.unwrap().name);
            for affected_id in &affected_ids {
                response.push_str(format!("\n[{}:{}]", affected_id.to_string(), affected_id.mention()).as_str())
            }
            response.push_str(format!("\n[{}]", reason).as_str());
            ctx.say(response).await?;
        } else {
            let mut response = format!("[{}]\n[{}:{}]", infraction_type.name(), mod_id.to_string(), mod_id.to_user(&ctx.http()).await.unwrap().name);
            let affected_id = affected_ids[affected_iter];
            response.push_str(format!("\n[{}:{}]", affected_id.to_string(), affected_id.mention()).as_str());
            response.push_str(format!("\n[{}]", reason).as_str());
            ctx.say(response).await?;
            if affected_ids.get(affected_iter + 1).is_some() {affected_ids.remove(affected_iter); affected_iter += 1};
        }
    }
    Ok(())
}