use std::process::Command;

use super::{Context, Error};

#[poise::command(slash_command, prefix_command)]
/// Command for updating the bot.
pub async fn update(
    interaction: Context<'_>,
) -> Result<(), Error> {
    interaction.reply("Updating!").await?;
    let output = Command::new("/usr/bin/systemctl")
        .arg("restart")
        .arg("ron-assista-bot")
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        println!("ron-assista-bot service restarted successfully.");
        if !output.stdout.is_empty() {
            println!("Output: {}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(())
    } else {
        panic!()
    }
}