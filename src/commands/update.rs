use std::process::Command;

use super::{Context, Error};

#[poise::command(slash_command, prefix_command)]
/// Command for updating the bot.
pub async fn update(
    interaction: Context<'_>,
) -> Result<(), Error> {
    interaction.reply("Updating!").await?;
    let output = Command::new("nohup")
        .arg("sh")
        .arg("-c")
        .arg("/root/rabby-stuff/ron-assista-bot/update.sh")
        .arg("&")
        .output()?;

    if output.status.success() {
        println!("Script started successfully in the background");
        println!("Initial output: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Failed to start the script");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}