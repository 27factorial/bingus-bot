use serenity::{
    framework::standard::{
        macros::{command, group, help},
        Args, CommandResult,
    },
    model::channel::{Message, Reaction},
    prelude::Context,
};

use crate::client::data_keys;
use serenity::builder::CreateEmbed;
use serenity::model::misc::Mention;

#[group]
#[description = "General, everyday commands."]
#[commands(create_embed, send_emoji, send_embed)]
pub struct General;

#[command]
#[description = "Creates an embed."]
async fn create_embed(ctx: &Context, original_msg: &Message) -> CommandResult {
    let channel = original_msg.channel_id;

    channel
        .send_message(&ctx, |msg| {
            msg.embed(|embed| embed.color(0xFF00FF).description("Test Embed"))
        })
        .await?;

    Ok(())
}

#[command]
#[description = "Create, edit, or delete an activity roster. Subcommands are create, edit, delete."]
async fn activity(ctx: &Context, original_msg: &Message, args: Args) -> CommandResult {
    let subcommand = match args.current() {
        Some(arg) => arg,
        None => {
            original_msg
                .channel_id
                .say(&ctx, "Please provide a subcommand.")
                .await?;
            return Ok(());
        }
    };

    match subcommand {
        "create" => activity_create(ctx, original_msg, args).await,
        "edit" => activity_edit(ctx, original_msg, args).await,
        "delete" => activity_delete(ctx, original_msg, args).await,
        _ => {
            original_msg
                .channel_id
                .say(
                    &ctx,
                    "Invalid subcommand. Valid subcommands are `create`, `edit`, and `delete`.",
                )
                .await?;
            Ok(())
        }
    }
}

async fn activity_create(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

async fn activity_edit(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

async fn activity_delete(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
async fn send_emoji(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let emoji_name = match args.current() {
        Some(name) => name,
        None => {
            original_msg
                .channel_id
                .say(ctx, "Please provide a valid name.")
                .await?;
            return Ok(());
        }
    };

    let type_map = ctx.data.read().await;

    let emoji_map = match type_map.get::<data_keys::GetEmojiMap>() {
        Some(map) => map,
        None => {
            original_msg
                .channel_id
                .say(ctx, "Emoji map was not registered.")
                .await?;
            return Ok(());
        }
    };

    match emoji_map.get(emoji_name) {
        Some(&emoji) => {
            let mention = Mention::from(emoji);

            original_msg.channel_id.say(ctx, mention).await?;
            Ok(())
        }
        None => {
            original_msg.channel_id.say(ctx, "Emoji not found.").await?;
            Ok(())
        }
    }
}

#[command]
async fn send_embed(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let embed_name = match args.current() {
        Some(name) => name,
        None => {
            original_msg
                .channel_id
                .say(ctx, "Please provide a valid name.")
                .await?;
            return Ok(());
        }
    };

    let type_map = ctx.data.read().await;

    let embed_map = match type_map.get::<data_keys::GetEmbedMap>() {
        Some(map) => map,
        None => {
            original_msg
                .channel_id
                .say(ctx, "Embed map was not registered.")
                .await?;
            return Ok(());
        }
    };

    match embed_map.get(embed_name) {
        Some(embed) => {
            original_msg
                .channel_id
                .send_message(ctx, |msg| msg.set_embed(CreateEmbed::from(embed.clone())))
                .await?;
            Ok(())
        }
        None => {
            original_msg.channel_id.say(ctx, "Embed not found.").await?;
            Ok(())
        }
    }
}
