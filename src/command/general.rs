use serenity::{
    framework::standard::{
        macros::{command, group, help},
        Args, CommandResult,
    },
    model::channel::{Message, Reaction},
    prelude::Context,
};

use crate::command::imp;
use crate::command::imp::data_keys;
use serenity::builder::CreateEmbed;
use serenity::model::channel::ReactionType;
use serenity::model::misc::Mention;
use std::time::Duration;

#[group]
#[description = "General, everyday commands."]
#[commands(create_embed, send_emoji, send_embed, send_embed_with_reactions)]
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
        "create" => activity_create(ctx, original_msg).await,
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

async fn activity_create(ctx: &Context, original_msg: &Message) -> CommandResult {
    // let mut type_map = ctx.data.write().await;
    //
    // let embeds = match type_map.get::<data_keys::GetEmbedMap>() {
    //     Some(map) => map,
    //     None => {
    //         return imp::send_error_message(
    //             ctx,
    //             original_msg,
    //             "Embed map was not loaded. Please notify Factorial about this.",
    //         )
    //         .await;
    //     }
    // };
    // let roster_start_embed = match embeds.get("activity_roster_start") {
    //     Some(embed) => embed,
    //     None => {
    //         return imp::send_error_message(
    //             ctx,
    //             original_msg,
    //             "Could not find embed `activity_roster_start` in embed map. Please notify Factorial about this.",
    //         ).await;
    //     }
    // };
    // let roster_start_msg = original_msg
    //     .channel_id
    //     .send_message(ctx, |msg| {
    //         msg.set_embed(imp::create_embed(roster_start_embed))
    //     })
    //     .await?;
    // let roster_start_reactions = [
    //     imp::byte_to_reaction(1).unwrap(),
    //     imp::byte_to_reaction(2).unwrap(),
    //     imp::byte_to_reaction(3).unwrap(),
    //     imp::byte_to_reaction(4).unwrap(),
    //     imp::byte_to_reaction(5).unwrap(),
    // ];
    //
    // imp::add_reactions(ctx, &roster_start_msg, &roster_start_reactions);

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
        Some(embed_with_reaction) => {
            original_msg
                .channel_id
                .send_message(ctx, |msg| {
                    msg.set_embed(CreateEmbed::from(embed_with_reaction.embed.clone()))
                })
                .await?;
            Ok(())
        }
        None => {
            original_msg.channel_id.say(ctx, "Embed not found.").await?;
            Ok(())
        }
    }
}

#[command]
async fn send_embed_with_reactions(
    ctx: &Context,
    original_msg: &Message,
    args: Args,
) -> CommandResult {
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
        Some(embed_with_reaction) => {
            embed_with_reaction
                .send_and_await_reaction(
                    ctx,
                    original_msg.channel_id,
                    Some(Duration::from_secs(30)),
                    Some(original_msg.author.id),
                )
                .await?;
            Ok(())
        }
        None => {
            original_msg.channel_id.say(ctx, "Embed not found.").await?;
            Ok(())
        }
    }
}
