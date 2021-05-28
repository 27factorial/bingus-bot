use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};

use crate::client;
use crate::command::imp::{self, data_keys};

#[group]
#[owners_only]
#[prefix("owner")]
#[commands(reload_json, add_admins, remove_admins)]
struct OwnersOnly;

#[command]
async fn reload_json(ctx: &Context, original_msg: &Message) -> CommandResult {
    let mut type_map = ctx.data.write().await;

    let paths = match type_map.get::<data_keys::GetJsonPaths>() {
        Some(paths) => paths.clone(),
        None => {
            original_msg
                .channel_id
                .say(ctx, "JSON paths not initialized in type map.")
                .await?;
            return Ok(());
        }
    };

    client::initialize_embed_map(&paths, &mut type_map).await;
    client::initialize_emoji_map(&paths, &mut type_map).await;

    original_msg
        .channel_id
        .say(ctx, "JSON values reloaded.")
        .await?;
    Ok(())
}

#[command]
async fn add_admins(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let mut type_map = ctx.data.write().await;
    let admins = type_map.entry::<data_keys::GetAdmins>().or_default();

    for (pos, result) in args.iter::<u64>().enumerate() {
        match result {
            Ok(id) => {
                admins.insert(id.into());
            }
            Err(_) => {
                imp::send_error_message(
                    ctx,
                    original_msg,
                    format!("Invalid user ID at position {}", pos),
                )
                .await?;
            }
        }
    }

    original_msg.react(ctx, '👍').await?;
    Ok(())
}

#[command]
async fn remove_admins(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let mut type_map = ctx.data.write().await;
    let admins = type_map.entry::<data_keys::GetAdmins>().or_default();

    for (pos, result) in args.iter::<u64>().enumerate() {
        match result {
            Ok(id) => {
                admins.remove(&id.into());
            }
            Err(_) => {
                imp::send_error_message(
                    ctx,
                    original_msg,
                    format!("Invalid user ID at position {}", pos),
                )
                .await?;
            }
        }
    }

    original_msg.react(ctx, '👍').await?;
    Ok(())
}
