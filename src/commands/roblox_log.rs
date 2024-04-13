use poise::ChoiceParameter;

use super::{Context, Error, helper, UserId, FromStr, RBX_CLIENT, NUMBER_REGEX};

#[derive(Debug, poise::ChoiceParameter)]
pub enum RobloxInfTypes {
    #[name = "Game Ban"]
    Ban,
    #[name = "Temporary Game Ban"]
    TempBan,
    #[name = "Game Server Ban"]
    ServerBan,
    Kick,
    Warn
}

// Command for making roblox-side logs
#[poise::command(slash_command, prefix_command)]
pub async fn robloxlog(
    interaction: Context<'_>,
    #[description = "Type of infraction."] #[rename = "type"] infraction_type: RobloxInfTypes,
    #[description = "Reason for infraction, split with |."] reason: String,
    #[description = "Roblox Usernames for the command, seperate with spaces."] roblox_users: Option<String>,
    #[description = "Roblox IDs for the command, seperate with spaces."] roblox_ids: Option<String>,
    #[description = "Discord IDs for the command, seperate with spaces."] discord_ids: Option<String>,
    #[description = "Note for the infraction, split with |."] note: Option<String>,
    #[description = "Multimessage mode allows creation of multiple logs from 1 command."] multimessage: Option<bool>
) -> Result<(), Error> {
    interaction.reply("Making logs...").await?;
    let multimessage = multimessage.unwrap_or_default();
    let mut roblox_users = roblox_users.unwrap_or_default().split(" ").map(str::to_string).collect::<Vec<String>>();
    let purified_users = NUMBER_REGEX.replace_all(discord_ids.unwrap_or_default().as_str(), "").to_string();
    let discord_ids = purified_users.split(" ").map(str::to_string).collect::<Vec<String>>();
    let purified_roblox_ids = NUMBER_REGEX.replace_all(roblox_ids.unwrap_or_default().as_str(), "").to_string();
    let mut roblox_ids = purified_roblox_ids.split(" ").map(str::to_string).collect::<Vec<String>>();
    if roblox_users[0].len() == 0 && discord_ids[0].len() == 0 && roblox_ids[0].len() == 0 {
        interaction.say("Command failed; no users inputted.").await?;
        return Ok(());
    }
    let reasons = reason.split("|").map(str::to_string).collect::<Vec<String>>();
    let notes = note.unwrap_or_default().split("|").map(str::to_string).collect::<Vec<String>>();

    if roblox_users[0].len() == 0 {roblox_users.remove(0);}
    let user_search = RBX_CLIENT.username_user_details(roblox_users, false).await?;
    for user in user_search {
        roblox_ids.push(user.id.to_string())
    }

    for id in discord_ids {
        if id.len() == 0 {continue}
        let discord_id = UserId::from_str(id.as_str()).expect("err");
        let roblox_id_str = match helper::discord_id_to_roblox_id(discord_id).await {Ok(id) => id, Err(err) => {
            interaction.say(err).await?;
            continue
        }};
        roblox_ids.push(roblox_id_str);
    }

    let mut users_string = String::new();
    let mut user_string_vec: Vec<String> = Vec::new();
    for id in roblox_ids {
        if id.len() == 0 {continue}
        let user_details = RBX_CLIENT.user_details(id.parse::<u64>().expect("err")).await?;
        let value = format!("[{}:{}]\n", user_details.username, user_details.id);
        if multimessage == false { users_string.push_str(value.as_str()) } else { user_string_vec.push(value) }
    }

    let type_string = format!("[{}]\n", infraction_type.name());
    if multimessage == false {
        let reason_string = format!("[{}]", reasons[0]);
        let note_string = if notes[0].len() != 0 {format!("\nNote: {}", notes[0])} else {String::new()};
        let response = format!("{}{}{}{}", type_string, users_string, reason_string, note_string);
        interaction.say(response).await?;
    } else {
        let mut reason_number = 0;
        let mut note_number = 0;
        for user_string in user_string_vec {
            let reason_string = format!("[{}]", reasons[reason_number]);
            let note_string = if notes[note_number].len() != 0 {format!("\nNote: {}", notes[note_number])} else {String::new()};
            let response = format!("{}{}{}{}", type_string, user_string, reason_string, note_string);
            interaction.say(response).await?;
            if reasons.get(reason_number + 1) != None { reason_number += 1 }
            if notes.get(note_number + 1 ) != None { note_number += 1 }
        }
    }
    Ok(())
}