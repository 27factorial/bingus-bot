use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};

use crate::command::data::ActivityError;
use crate::command::data::{Activity, GuildData};
use crate::command::imp;
use crate::command::imp::data_keys;
use chrono::{DateTime, Utc};
use serenity::builder::CreateEmbed;

use serenity::model::misc::Mention;
use std::time::Duration;

#[group]
#[description = "General, everyday commands."]
#[commands(activity)]
pub struct General;

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
        "join" => activity_join(ctx, original_msg, args).await,
        "alt" => activity_alt(ctx, original_msg, args).await,
        "leave" => activity_leave(ctx, original_msg, args).await,
        "delete" => activity_delete(ctx, original_msg, args).await,
        _ => {
            imp::send_error_message(
                ctx,
                original_msg,
                "Invalid subcommand. Valid subcommands are `create`, `join`, `alt`, `leave`, and `delete`.",
            )
            .await?;
            Ok(())
        }
    }
}

async fn activity_create(ctx: &Context, original_msg: &Message) -> CommandResult {
    let guild_id = original_msg.guild_id;

    let type_map = ctx.data.read().await;

    let embed_map = match type_map.get::<data_keys::GetEmbedMap>() {
        Some(map) => map.clone(),
        None => {
            original_msg
                .channel_id
                .say(ctx, "Embed map was not registered.")
                .await?;
            return Ok(());
        }
    };

    // Unlock read lock so that other contexts can use it.
    drop(type_map);

    match embed_map.get("activity_roster_start") {
        Some(embed_with_meta) => {
            let mut data = embed_with_meta
                .clone()
                .send_embed_chain(
                    ctx,
                    &embed_map,
                    original_msg.channel_id,
                    Some(Duration::from_secs(120)),
                    Some(original_msg.author.id),
                )
                .await?;

            data.message.delete_reactions(ctx).await?;

            let time_embed = match embed_map.get("activity_roster_time") {
                Some(embed) => embed,
                None => {
                    imp::send_error_message(ctx, &data.message, "An error has occurred getting embed `activity_roster_time`. Please contact Factorial about this.").await?;
                    return Ok(());
                }
            };

            data.message
                .edit(ctx, |msg| {
                    msg.embed(|embed| {
                        *embed = imp::create_embed(&time_embed.embed);
                        embed
                    })
                })
                .await?;

            let (date_time, date_time_str) = loop {
                let mut collector = data
                    .message
                    .channel_id
                    .await_reply(ctx)
                    .author_id(original_msg.author.id);

                if let Some(duration) = data.timeout {
                    collector = collector.timeout(duration);
                }

                let time_message = match collector.await {
                    Some(message) => message,
                    None => {
                        let error = match data.timeout {
                            Some(duration) => format!("You did not send a reply in time. Please reply within {} minutes", duration.as_secs() / 60),
                            None => "Some other error occurred getting a reply. Please contact Factorial about this.".into(),
                        };

                        imp::send_error_message(ctx, &data.message, error).await?;
                        return Ok(());
                    }
                };

                match imp::parse_date_time(&time_message.content) {
                    Some((date_time, date_time_str)) => {
                        time_message.delete(ctx).await?;
                        break (date_time, date_time_str);
                    }
                    None => {
                        imp::send_error_message(ctx, &time_message, "Please enter a valid date and time in the format `mm/dd/yyyy hh:mm am|pm`").await?;
                    }
                }
            };

            data.message.delete_reactions(ctx).await?;

            let description_embed = match embed_map.get("activity_roster_description") {
                Some(embed) => embed,
                None => {
                    imp::send_error_message(ctx, &data.message, "An error has occurred getting embed `activity_roster_description`. Please contact Factorial about this.").await?;
                    return Ok(());
                }
            };

            data.message
                .edit(ctx, |msg| {
                    msg.embed(|embed| {
                        *embed = imp::create_embed(&description_embed.embed);
                        embed
                    })
                })
                .await?;

            let description = loop {
                let mut collector = data
                    .message
                    .channel_id
                    .await_reply(ctx)
                    .author_id(original_msg.author.id);

                if let Some(duration) = data.timeout {
                    collector = collector.timeout(duration);
                }

                let description_message = match collector.await {
                    Some(message) => message,
                    None => {
                        let error = match data.timeout {
                            Some(duration) => format!("You did not send a reply in time. Please reply within {} minutes", duration.as_secs() / 60),
                            None => "Some other error occurred getting a reply. Please contact Factorial about this.".into(),
                        };

                        imp::send_error_message(ctx, &data.message, error).await?;
                        return Ok(());
                    }
                };

                let safe_content = description_message.content_safe(ctx).await;

                if safe_content.len() <= 1024 {
                    description_message.delete(ctx).await?;
                    break safe_content;
                } else {
                    imp::send_error_message(
                        ctx,
                        &description_message,
                        "Please enter a description that is less than or equal to 1024 characters.",
                    )
                    .await?;
                }
            };

            let date_time_now = DateTime::from_utc(Utc::now().naive_utc(), date_time.timezone());
            let duration_until_start = match (date_time - date_time_now).to_std() {
                Ok(duration) => duration,
                Err(_) => {
                    imp::send_error_message(
                        ctx,
                        &data.message,
                        "Invalid date and time. Please enter a valid date and time that is in the future.",
                    )
                    .await?;
                    return Ok(());
                }
            };

            if let Some(id) = guild_id {
                let mut type_map = ctx.data.write().await;
                let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
                let guild_data = guild_data_map
                    .entry(id.0)
                    .or_insert_with(|| GuildData::new(id));

                let activity_id = guild_data.activity_id();

                let activity = Activity::new(
                    data.activity_name.to_string(),
                    description,
                    date_time_str,
                    activity_id,
                    data.size,
                    original_msg.author.id,
                    data.message.clone(),
                );

                match guild_data.add_activity(activity) {
                    Ok(_) => (),
                    Err(_) => {
                        imp::send_error_message(ctx, &data.message, "Error: that activity already exists. Please contact Factorial about this.").await?;
                        return Ok(());
                    }
                };

                let activity_embed = guild_data
                    .activity(activity_id)
                    .unwrap()
                    .as_create_embed(0x212121);

                data.message
                    .edit(ctx, |msg| {
                        msg.embed(|embed| {
                            *embed = activity_embed;
                            embed
                        })
                    })
                    .await?;

                drop(type_map);

                tokio::time::sleep(duration_until_start).await;
                let mut type_map = ctx.data.write().await;

                if let Some(guild_map) = type_map.get_mut::<data_keys::GetGuildData>() {
                    if let Some(guild_data) = guild_map.get_mut(&id.0) {
                        if let Some(activity) = guild_data.remove_activity(activity_id) {
                            if !activity.members.is_empty() {
                                let member_count = activity.members.len();

                                let mention_string = activity
                                    .members
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, user)| {
                                        if idx == 0 {
                                            Mention::from(user).to_string()
                                        } else if idx == member_count - 1 {
                                            format!(", and {}", Mention::from(user))
                                        } else {
                                            format!(", {}", Mention::from(user))
                                        }
                                    })
                                    .collect::<String>();

                                let content = format!(
                                    "Hey {}! {} is starting now. Good luck and have fun!",
                                    mention_string, activity.name
                                );

                                activity.embed_msg.channel_id.say(ctx, content).await?;
                            }
                        }
                    }
                }
            }

            Ok(())
        }
        None => {
            original_msg.channel_id.say(ctx, "Embed not found.").await?;
            Ok(())
        }
    }
}

async fn activity_join(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
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

    let activity = match guild_data.activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.add_member(original_msg.author.id) {
        Err(ActivityError::MemberAlreadyInList) => Some("You are already in that member list."),
        Err(ActivityError::MemberListFull) => Some("The member list for that activity is already full."),
        Err(ActivityError::MemberNotInAlternate) => {
            Some("Attempted to move you to the member list, but you were not an alternate. Please try again.")
        }
        Err(_) => Some("Some other error occurred adding you to the member list."),
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

async fn activity_alt(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
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

    let activity = match guild_data.activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.add_member_alt(original_msg.author.id) {
        Err(ActivityError::MemberAlreadyInAlternate) => {
            Some("You are already in that alternate member list.")
        }
        Err(ActivityError::AlternateFull) => {
            Some("The alternate member list for that activity is already full.")
        }
        Err(_) => Some("Some other error occurred adding you to the member list."),
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

async fn activity_leave(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
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

    let activity = match guild_data.activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.remove_member(original_msg.author.id) {
        Err(_) => match activity.remove_member_alt(original_msg.author.id) {
            Err(ActivityError::MemberNotInAlternate) => {
                Some("You are not in that activity's member list.")
            }
            Err(_) => Some("Some other error occurred removing you from the member list."),
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

async fn activity_delete(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
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

    let creator_id = match guild_data.activity(activity_id) {
        Some(activity) => activity.creator,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    if original_msg.author.id == creator_id {
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
    } else {
        imp::send_error_message(ctx, original_msg, "You cannot delete that activity.").await?;
    }

    Ok(())
}
