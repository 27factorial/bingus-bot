use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};

use crate::command::data::{ActivityError, GuildData};
use crate::command::imp::{self, data_keys};
use serenity::model::prelude::UserId;

#[group]
#[owners_only]
#[prefix("admin")]
#[commands(activity_manage, nick)]
struct AdminsOnly;

#[command]
async fn activity_manage(ctx: &Context, original_msg: &Message, args: Args) -> CommandResult {
    if imp::is_admin(ctx, original_msg.author.id).await? {
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
            "add" => activity_manage_add(ctx, original_msg, args).await?,
            "alt" => activity_manage_alt(ctx, original_msg, args).await?,
            "remove" => activity_manage_remove(ctx, original_msg, args).await?,
            "delete" => activity_manage_delete(ctx, original_msg, args).await?,
            _ => {
                imp::send_error_message(
                    ctx,
                    original_msg,
                    "Invalid subcommand. Valid subcommands are `add`, `alt`, `remove`, `delete`.",
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn activity_manage_add(
    ctx: &Context,
    original_msg: &Message,
    mut args: Args,
) -> CommandResult {
    let user_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let user_id = match user_opt {
        Some(id) => UserId::from(id),
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid user ID.").await?;
            return Ok(());
        }
    };

    let activity_id_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let activity_id = match activity_id_opt {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let mut type_map = ctx.data.write().await;

    let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
    let guild_data = guild_data_map
        .entry(guild_id.0)
        .or_insert_with(|| GuildData::new(guild_id));

    let activity = match guild_data.get_activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.add_member(user_id) {
        Err(ActivityError::MemberAlreadyInFireteam) => {
            Some("That user is already in that fireteam.")
        }
        Err(ActivityError::FireteamFull) => Some("The fireteam for that activity is already full."),
        Err(ActivityError::MemberNotInAlternate) => Some("Attempted to move that user to the fireteam, but they were not an alternate. Please try again."),
        Err(_) => Some("Some other error occurred adding that user to the fireteam."),
        Ok(()) => None,
    };

    if let Some(msg) = error {
        imp::send_error_message(ctx, original_msg, msg).await?;
        return Ok(());
    }

    let activity_embed = activity.as_create_embed(0x212121);

    activity
        .embed_msg
        .edit(ctx, |msg| {
            msg.embed(|embed| {
                *embed = activity_embed;
                embed
            })
        })
        .await?;

    original_msg.delete(ctx).await?;
    Ok(())
}

async fn activity_manage_alt(
    ctx: &Context,
    original_msg: &Message,
    mut args: Args,
) -> CommandResult {
    let user_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let user_id = match user_opt {
        Some(id) => UserId::from(id),
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid user ID.").await?;
            return Ok(());
        }
    };

    let activity_id_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let activity_id = match activity_id_opt {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let mut type_map = ctx.data.write().await;

    let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
    let guild_data = guild_data_map
        .entry(guild_id.0)
        .or_insert_with(|| GuildData::new(guild_id));

    let activity = match guild_data.get_activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.add_member_alt(user_id) {
        Err(ActivityError::MemberAlreadyInAlternate) => {
            Some("That user are already in that alternate fireteam.")
        }
        Err(ActivityError::AlternateFull) => {
            Some("The alternate fireteam for that activity is already full.")
        }
        Err(_) => Some("Some other error occurred adding that user to the fireteam."),
        Ok(()) => None,
    };

    if let Some(msg) = error {
        imp::send_error_message(ctx, original_msg, msg).await?;
        return Ok(());
    }

    let activity_embed = activity.as_create_embed(0x212121);

    activity
        .embed_msg
        .edit(ctx, |msg| {
            msg.embed(|embed| {
                *embed = activity_embed;
                embed
            })
        })
        .await?;

    original_msg.delete(ctx).await?;

    Ok(())
}

async fn activity_manage_remove(
    ctx: &Context,
    original_msg: &Message,
    mut args: Args,
) -> CommandResult {
    let user_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let user_id = match user_opt {
        Some(id) => UserId::from(id),
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid user ID.").await?;
            return Ok(());
        }
    };

    let activity_id_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let activity_id = match activity_id_opt {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let mut type_map = ctx.data.write().await;

    let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
    let guild_data = guild_data_map
        .entry(guild_id.0)
        .or_insert_with(|| GuildData::new(guild_id));

    let activity = match guild_data.get_activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.remove_member(user_id) {
        Err(_) => match activity.remove_member_alt(user_id) {
            Err(ActivityError::MemberNotInAlternate) => {
                Some("That user is not in that activity's fireteam.")
            }
            Err(_) => Some("Some other error occurred removing that user from the fireteam."),
            Ok(()) => None,
        },
        Ok(()) => None,
    };

    if let Some(msg) = error {
        imp::send_error_message(ctx, original_msg, msg).await?;
        return Ok(());
    }

    let activity_embed = activity.as_create_embed(0x212121);

    activity
        .embed_msg
        .edit(ctx, |msg| {
            msg.embed(|embed| {
                *embed = activity_embed;
                embed
            })
        })
        .await?;

    original_msg.delete(ctx).await?;

    Ok(())
}

async fn activity_manage_delete(
    ctx: &Context,
    original_msg: &Message,
    mut args: Args,
) -> CommandResult {
    let activity_id_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let activity_id = match activity_id_opt {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let mut type_map = ctx.data.write().await;

    let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
    let guild_data = guild_data_map
        .entry(guild_id.0)
        .or_insert_with(|| GuildData::new(guild_id));

    let activity_opt = guild_data.remove_activity(activity_id);
    if let Some(activity) = activity_opt {
        activity.embed_msg.delete(ctx).await?;
        original_msg
            .channel_id
            .say(
                ctx,
                format!("Deleted activity {}: {}.", activity.id, activity.name),
            )
            .await?;
    }

    Ok(())
}

#[command]
async fn nick(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let name = args
        .iter::<String>()
        .map(|result| {
            let mut string = result.unwrap();
            string.push(' ');
            string
        })
        .collect::<String>();

    if (0..33).contains(&name.len()) {
        let guild_id = match original_msg.guild_id {
            Some(id) => id,
            None => {
                imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                    .await?;
                return Ok(());
            }
        };

        if name.is_empty() {
            guild_id.edit_nickname(ctx, None).await?;
        } else {
            guild_id.edit_nickname(ctx, Some(&name)).await?;
        }

        original_msg.react(ctx, '👍').await?;
    } else {
        imp::send_error_message(
            ctx,
            original_msg,
            "Please ensure that the bot's new nickname is between \
            1 and 32 characters long.",
        )
        .await?;
    }

    Ok(())
}
