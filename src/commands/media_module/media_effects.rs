use std::fmt::{self, Display};

use poise::ChoiceParameter;
use serenity::all::Attachment;

use super::{Context, Error, apply_mask};

#[derive(Debug, poise::ChoiceParameter)]
pub enum SpeechBubbleOverlays {
    #[name = "esm Bot Style"]
    EsmBotStyle,
    #[name = "RON Bot Style"]
    RONBotStyle
}

impl fmt::Display for SpeechBubbleOverlays {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeechBubbleOverlays::EsmBotStyle => write!(f, "EsmBotStyle"),
            SpeechBubbleOverlays::RONBotStyle => write!(f, "RONBotStyle"),
        }
    }
}

#[poise::command(slash_command, prefix_command,
    subcommands(
        "speechbubble",
    ))]
/// Command for adding a speech bubble to any image or video, with the ability to convert to gif.
pub async fn speechbubble(
    ctx: Context<'_>,
    #[description = "Attachment for command."] attachment: Attachment,
    #[description = "A bit technical, but what should the height of the overlay be divided by?"] height_float: Option<f32>,
    #[description = "Overlay type, defaults to esm bot style."] style: Option<SpeechBubbleOverlays>,
    #[description = "Should the speech bubble be flipped horizontally?"] flip: Option<bool>,
    #[description = "Should the speech bubble be transparent? By default set to true if image."] transparent: Option<bool>,
) -> Result<(), Error> {
    // let style = if style.is_some() {
    //     style.unwrap()
    // } else {
    //     SpeechBubbleOverlays::EsmBotStyle
    // };

    // let overlay_path = format!("default_masks/{}", style);
    // let input_path = format!("./tmp/{}.{}");
    // apply_mask(input_path, &overlay_path, output_path, flip, height_float, transparent);
    ctx.say("Coming soon.").await.unwrap();

    Ok(())
}