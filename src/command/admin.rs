use chrono::{FixedOffset, TimeZone, Utc};
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};
use std::time::Duration;

use crate::command::data::{ActivityError, GuildData};
use crate::command::imp::{self, data_keys};
use crate::util::CancelActivity;
use serenity::model::misc::Mention;
use serenity::model::prelude::UserId;

#[group]
#[prefix("admin")]
#[commands(activity, echo, pin, nick)]
struct AdminsOnly;

#[command]
async fn activity(ctx: &Context, original_msg: &Message, args: Args) -> CommandResult {
    if imp::is_admin(ctx, original_msg.author.id).await {
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
            "add" => admin_activity_add(ctx, original_msg, args).await?,
            "alt" => admin_activity_alt(ctx, original_msg, args).await?,
            "remove" => admin_activity_remove(ctx, original_msg, args).await?,
            "edit" => admin_activity_edit(ctx, original_msg, args).await?,
            "delete" => admin_activity_delete(ctx, original_msg, args).await?,
            "start" => admin_activity_start(ctx, original_msg, args).await?,
            "ping" => admin_activity_ping(ctx, original_msg, args).await?,
            _ => {
                imp::send_error_message(
                    ctx,
                    original_msg,
                    "Invalid subcommand. Valid subcommands are `add`, `alt`, `remove`, `delete`, `start`, and `ping`.",
                )
                .await?;
            }
        }
    }

    Ok(())
}

#[command]
async fn echo(ctx: &Context, original_msg: &Message, args: Args) -> CommandResult {
    if imp::is_admin(ctx, original_msg.author.id).await {
        let bingus_message = args.raw().collect::<Vec<&str>>().join(" ");

        original_msg.delete(ctx).await?;
        original_msg.channel_id.say(ctx, bingus_message).await?;
    }

    Ok(())
}

#[command]
async fn pin(ctx: &Context, original_msg: &Message, args: Args) -> CommandResult {
    if imp::is_admin(ctx, original_msg.author.id).await {
        let id_opt = args
            .current()
            .map(|string| string.parse::<u64>().ok())
            .flatten();

        let id = match id_opt {
            Some(id) => id,
            None => {
                imp::send_error_message(ctx, original_msg, "Please enter a valid message ID")
                    .await?;
                return Ok(());
            }
        };

        original_msg.channel_id.pin(ctx, id).await?;
    }

    Ok(())
}

async fn admin_activity_add(
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

    let activity = match guild_data.activity_mut(activity_id) {
        Some(activity) => activity,
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    let error = match activity.add_member(user_id) {
        Err(ActivityError::MemberAlreadyInList) => {
            Some("That user is already in that fireteam.")
        }
        Err(ActivityError::MemberListFull) => Some("The fireteam for that activity is already full."),
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

async fn admin_activity_alt(
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

    let activity = match guild_data.activity_mut(activity_id) {
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

async fn admin_activity_remove(
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

    let activity = match guild_data.activity_mut(activity_id) {
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

async fn admin_activity_edit(
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

    let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
    let guild_data = guild_data_map
        .entry(guild_id.0)
        .or_insert_with(|| GuildData::new(guild_id));

    match guild_data.activity(activity_id) {
        Some(activity) => {
            let mut activity = activity.clone();

            drop(type_map);

            let timeout = Duration::from_secs(120);

            let time_embed = match embed_map.get("activity_roster_time") {
                Some(embed) => embed,
                None => {
                    imp::send_error_message(ctx, original_msg, "An error has occurred getting embed `activity_roster_time`. Please contact Factorial about this.").await?;
                    return Ok(());
                }
            };

            let mut embed_msg = original_msg
                .channel_id
                .send_message(ctx, |msg| {
                    msg.embed(|embed| {
                        *embed = imp::create_embed(&time_embed.embed);
                        embed
                    })
                })
                .await?;

            let (date_time, date_time_str) = loop {
                let collector = embed_msg
                    .channel_id
                    .await_reply(ctx)
                    .author_id(original_msg.author.id)
                    .timeout(timeout);

                let time_message = match collector.await {
                    Some(message) => message,
                    None => {
                        imp::send_error_message(
                            ctx,
                            original_msg,
                            format!(
                                "You did not send a reply in time. Please reply within {} minutes",
                                timeout.as_secs() / 60
                            ),
                        )
                        .await?;
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

            let description_embed = match embed_map.get("activity_roster_description") {
                Some(embed) => embed,
                None => {
                    imp::send_error_message(ctx, original_msg, "An error has occurred getting embed `activity_roster_description`. Please contact Factorial about this.").await?;
                    return Ok(());
                }
            };

            embed_msg
                .edit(ctx, |msg| {
                    msg.embed(|embed| {
                        *embed = imp::create_embed(&description_embed.embed);
                        embed
                    })
                })
                .await?;

            let description = loop {
                let collector = embed_msg
                    .channel_id
                    .await_reply(ctx)
                    .author_id(original_msg.author.id)
                    .timeout(timeout);

                let description_message = match collector.await {
                    Some(message) => message,
                    None => {
                        imp::send_error_message(
                            ctx,
                            original_msg,
                            format!(
                                "You did not send a reply in time. Please reply within {} minutes",
                                timeout.as_secs() / 60
                            ),
                        )
                        .await?;
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

            let (cancel_future, cancel_token) = CancelActivity::new_pair();
            tokio::pin!(cancel_future);

            activity.date = date_time_str;
            activity.description = description;
            activity.cancel_token = cancel_token;

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

            embed_msg.delete(ctx).await?;

            let mut type_map = ctx.data.write().await;
            let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
            let guild_data = guild_data_map.entry(guild_id.0).or_default();

            let old_activity = match guild_data.activity_mut(activity_id) {
                Some(activity) => activity,
                None => {
                    imp::send_error_message(ctx, original_msg, "Could not get the old activity from GuildData. Please contact Factorial about this.").await?;
                    return Ok(());
                }
            };

            original_msg
                .channel_id
                .say(
                    ctx,
                    format!(
                        "Activity {} ({}) updated successfully.",
                        activity.id, activity.name
                    ),
                )
                .await?;

            *old_activity = activity;

            drop(type_map);

            let date_time_now =
                FixedOffset::west(4 * 3600).from_utc_datetime(&Utc::now().naive_utc());
            let duration_until_start = match (date_time - date_time_now).to_std() {
                Ok(duration) => duration,
                Err(_) => {
                    imp::send_error_message(
                        ctx,
                        original_msg,
                        "Invalid date and time. Please enter a valid date and time that is in the future.",
                    )
                        .await?;
                    return Ok(());
                }
            };

            let sleep = tokio::time::sleep(duration_until_start);
            tokio::pin!(sleep);

            tokio::select! {
                _ = &mut cancel_future => (),
                _ = &mut sleep => {
                    let mut type_map = ctx.data.write().await;
                    imp::start_activity(ctx, &mut type_map, guild_id, activity_id).await?;
                }
            }

            Ok(())
        }
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            Ok(())
        }
    }
}

async fn admin_activity_delete(
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
        activity.cancel_token.cancel();
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

async fn admin_activity_start(
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

    let activity_opt = guild_data.activity(activity_id);

    let activity_id = match activity_opt {
        Some(activity) => {
            activity.cancel_token.cancel();
            activity.id
        }
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            return Ok(());
        }
    };

    Ok(imp::start_activity(ctx, &mut type_map, guild_id, activity_id).await?)
}

async fn admin_activity_ping(
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

    let activity_opt = guild_data.activity(activity_id);
    match activity_opt {
        Some(activity) => {
            if !activity.members.is_empty() {
                let member_count = activity.members.len();

                let mention_string = activity
                    .members
                    .iter()
                    .enumerate()
                    .map(|(idx, &user)| {
                        if idx == 0 {
                            Mention::from(user).to_string()
                        } else if idx == member_count - 1 {
                            format!(", and {}", Mention::from(user))
                        } else {
                            format!(", {}", Mention::from(user))
                        }
                    })
                    .collect::<String>();

                let content = format!("{}", mention_string);

                activity.embed_msg.channel_id.say(ctx, content).await?;
            }
        }
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
        }
    }

    Ok(())
}

#[command]
async fn nick(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    if imp::is_admin(ctx, original_msg.author.id).await {
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
                    imp::send_error_message(
                        ctx,
                        original_msg,
                        "This command is not supported in DMs.",
                    )
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
    }

    Ok(())
}
