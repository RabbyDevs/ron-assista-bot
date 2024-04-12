// Command for making discord-side logs
use poise::ChoiceParameter;
use serenity::User;

use super::{Context, Error, helper, UserId, Mentionable, serenity, FromStr, RBX_CLIENT};

#[derive(Debug, poise::ChoiceParameter)]
pub enum RoleEnums {
    #[name = "Dedicated Player"]
    DedicatedPlayer,
    Collector,
    Grandmaster,
    Gamebanned,
    #[name = "VIP Blacklist"]
    VIPBlacklist,
    #[name = "Debate Blacklist"]
    DebateBlacklist,
    #[name = "Event Blacklist"]
    EventBlacklist,
    #[name = "Suggestion Blacklist"]
    SuggestionBlacklist,
    #[name = "VC Blackist"]
    VCBlacklist,
    #[name = "Application Blacklist"]
    ApplicationBlacklist,
    #[name = "Creations Blacklist"]
    CreationsBlacklist,
    #[name = "Wiki Blacklist"]
    WikiBlacklist,
    #[name = "Challenges Blacklist"]
    ChallengesBlacklist,
    #[name = "Strategies Blacklist"]
    StrategiesBlacklist,
    #[name = "Ticket Blacklist"]
    TicketBlacklist,
    #[name = "Feedback Blacklist"]
    FeedbackBlacklist,
    #[name = "Gamebot Blacklist"]
    GamebotBlacklist,
    #[name = "Trusted VIP Host"]
    TrustedVIPHost,
    #[name = "Ask for Help Blacklist"]
    AskForHelpBlacklist,
}
#[derive(Debug, poise::ChoiceParameter)]
pub enum LogType {
    Addition,
    Removal,
}

#[poise::command(slash_command, prefix_command)]
pub async fn rolelog(
    interaction: Context<'_>,
    #[description = "The role."] #[rename = "type"] infraction_type: LogType,
    #[description = "The role."] #[rename = "role"] role: RoleEnums,
    #[description = "User ids for the command."] users: String,
    #[description = "Optional reason for giving the role."] reason: Option<String>,
) -> Result<(), Error> {
    interaction.reply("Making logs, please standby!").await?;
    let users = users.split(" ");
    let reason = reason.unwrap_or_default();
    for snowflake in users {
        let userid: UserId = UserId::from_str(snowflake).expect("something went wrong.");
        let roblox_handler = tokio::spawn(async move {
            let mut roblox_errors = Vec::new();
            let roblox_id = match helper::discord_id_to_roblox_id(userid).await {
                Ok(roblox_id) => roblox_id,
                Err(_) => {roblox_errors.push(format!("A error occured on Bloxlink's end when getting {}'s Roblox id. The user may be not verified with Bloxlink or Bloxlink is down.", userid));
                "null".to_string()}
            };
            let roblox_user = if roblox_id != "null".to_string() {RBX_CLIENT.user_details(roblox_id.parse::<u64>().expect("err")).await.expect("err").username} else { "null".to_string() };
            (roblox_id, roblox_user, roblox_errors)
        });
        let user: User = match userid.to_user(interaction).await {
            Ok(user) => user,
            Err(_) => {
                interaction.say(format!("A error occured attempting to process user `{}` skipping user's log.", snowflake)).await?;
                continue
            }
        };
        let (roblox_user, roblox_id, roblox_errors) = roblox_handler.await.unwrap();
        for error in roblox_errors {interaction.say(error).await?;}
        let mut response = format!("[{}]\n[{}]\n[{}:{} - {}:{}]", infraction_type.name(), role.name(), user.mention(), user.id, roblox_user, roblox_id);
        if reason.len() != 0 {response.push_str(format!("\n[{}]", reason).as_str())}
        interaction.say(response).await?;
    }
    Ok(())
}