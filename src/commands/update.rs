use std::process::Command;

use super::{Context, Error};

#[poise::command(slash_command, prefix_command)]
/// Command for updating the bot.
pub async fn update(
    interaction: Context<'_>,
) -> Result<(), Error> {
    interaction.reply("Updating!").await?;
    let output = Command::new("sh")
        .arg("-c")
        .arg("/root/rabby-stuff/ron-assista-bot/update.sh")
        .output()?;

    if output.status.success() {
        println!("Script executed successfully");
        println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Script failed to execute");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}