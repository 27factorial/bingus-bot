use serenity::builder::CreateEmbed;
use serenity::model::misc::Mention;
use serenity::{
    framework::standard::{
        macros::{command, group, help},
        Args, CommandResult,
    },
    model::channel::{Message, Reaction},
    prelude::Context,
};

use crate::client;
use crate::command::imp::data_keys;

#[group]
#[owners_only]
#[prefix("admin")]
#[commands(reload_json)]
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
