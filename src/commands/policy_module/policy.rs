use crate::Data;

use super::{Context, Error};
use poise::Modal;

#[poise::command(slash_command, prefix_command, 
    subcommands("edit", "delete", "publish", "list"),
    subcommand_required)]
/// Command for managing policies
pub async fn policy(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[derive(Debug, Modal)]
#[name = "Policy Editor:"] // Struct name by default
struct EditModal {
    order: String,
    #[paragraph]
    content: String,
}


#[poise::command(slash_command)]
/// Edit an existing policy
pub async fn edit(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "Policy internal name"] internal_name: String,
    // #[description = "New title"] title: String,
    // #[description = "New content"] content: String,
    // #[description = "Order value for the policy"] order: u64,
) -> Result<(), Error> {
    let policy_system = &ctx.data().policy_system;

    let data = EditModal::execute(ctx).await?;
    let data = data.unwrap();
    // Edit the policy
    policy_system.edit(&internal_name, data.content, data.order.parse::<u64>().unwrap()).unwrap();
    
    // Notify the user
    ctx.say(format!("Policy '{}' updated and changes cached.", internal_name)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
/// Delete an existing policy
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Policy internal name"] internal_name: String,
) -> Result<(), Error> {
    let policy_system = &ctx.data().policy_system;
    // Delete the policy
    policy_system.remove(&internal_name).unwrap();
    
    // Notify the user
    ctx.say(format!("Policy '{}' deleted and changes cached.", internal_name)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
/// Publish all cached changes
pub async fn publish(
    ctx: Context<'_>
) -> Result<(), Error> {
    let policy_system = &ctx.data().policy_system;
    ctx.say(format!("Policy cached changes applying.")).await?;
    policy_system.update_policy(&ctx.serenity_context().clone()).await.unwrap();
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
/// List all policies and their internal names
pub async fn list(
    ctx: Context<'_>
) -> Result<(), Error> {
    let policy_system = &ctx.data().policy_system;
    let policies = policy_system.list_policies_internal_names().unwrap();
    let mut policy_list_string = String::from("Current Policy Internal Names:");

    for (internal_name, _) in policies.iter() {
        policy_list_string.push_str(format!("\n{}", internal_name).as_str());
    }

    ctx.say(policy_list_string).await?;

    Ok(())
}
