use crate::command::data::{Activity, GuildData};
use crate::command::data::{ActivityError, MarkovInfo};
use crate::command::imp;
use crate::command::imp::data_keys;
use crate::util::CancelActivity;
use chrono::{FixedOffset, TimeZone, Utc};
use itertools::Itertools;
use serenity::builder::CreateEmbed;
use serenity::model::id::UserId;
use serenity::model::misc::Mention;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::Context,
};
use std::time::Duration;

#[group]
#[description = "General, everyday commands."]
#[commands(activity, markov)]
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
        "edit" => activity_edit(ctx, original_msg, args).await,
        "delete" => activity_delete(ctx, original_msg, args).await,
        "list" => activity_list(ctx, original_msg, args).await,
        _ => {
            imp::send_error_message(
                ctx,
                original_msg,
                "Invalid subcommand. Valid subcommands are `create`, `join`, `alt`, `leave`, `edit`, `delete`, and `list`.",
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

            let date_time_now =
                FixedOffset::west(3 * 3600).from_utc_datetime(&Utc::now().naive_utc());
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

            if let Some(guild_id_val) = guild_id {
                let mut type_map = ctx.data.write().await;
                let guild_data_map = type_map.entry::<data_keys::GetGuildData>().or_default();
                let guild_data = guild_data_map
                    .entry(guild_id_val.0)
                    .or_insert_with(|| GuildData::new(guild_id_val));

                let activity_id = guild_data.activity_id();

                let (cancel_future, cancel_token) = CancelActivity::new_pair();
                tokio::pin!(cancel_future);

                let activity = Activity::new(
                    data.activity_name.to_string(),
                    description,
                    date_time_str,
                    activity_id,
                    data.size,
                    original_msg.author.id,
                    data.message.clone(),
                    cancel_token,
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

                let sleep = tokio::time::sleep(duration_until_start);
                tokio::pin!(sleep);

                tokio::select! {
                    _ = &mut cancel_future => (),
                    _ = &mut sleep => {
                        let mut type_map = ctx.data.write().await;
                        imp::start_activity(ctx, &mut type_map, guild_id_val, activity_id).await?;
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

async fn activity_edit(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
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

            let creator_id = activity.creator;

            if original_msg.author.id == creator_id {
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
                            imp::send_error_message(ctx, original_msg, format!("You did not send a reply in time. Please reply within {} minutes", timeout.as_secs() / 60)).await?;
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
                            imp::send_error_message(ctx, original_msg, format!("You did not send a reply in time. Please reply within {} minutes", timeout.as_secs() / 60)).await?;
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
            } else {
                imp::send_error_message(ctx, original_msg, "You cannot edit that activity.")
                    .await?;
            }

            Ok(())
        }
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid activity ID.").await?;
            Ok(())
        }
    }
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
            if !activity.cancel_token.cancel() {
                eprintln!("Error: Could not cancel Activity {}: {}. It may leak memory until the duration times out.", activity_id, activity.name);
            }
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

async fn activity_list(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let page_opt = args
        .advance()
        .current()
        .map(|string| string.parse::<u64>().ok())
        .flatten();

    let page = match page_opt {
        Some(p) => {
            if p == 0 {
                0
            } else {
                p - 1
            }
        }
        None => {
            imp::send_error_message(ctx, original_msg, "Invalid page number.").await?;
            return Ok(());
        }
    };

    let type_map = ctx.data.read().await;

    let guild_data_map = match type_map.get::<data_keys::GetGuildData>() {
        Some(map) => map,
        None => {
            imp::send_error_message(
                ctx,
                original_msg,
                "No activities have been created in this server. Be the first to make one!",
            )
            .await?;
            return Ok(());
        }
    };

    let guild_data = match guild_data_map.get(&guild_id.0) {
        Some(data) => data,
        None => {
            imp::send_error_message(
                ctx,
                original_msg,
                "No activities have been created in this server. Be the first to make one!",
            )
            .await?;
            return Ok(());
        }
    };

    let activities = guild_data
        .activities()
        .iter()
        .sorted_by(|&(&a, _), &(&b, _)| a.cmp(&b))
        .map(|(_, activity)| activity)
        .collect::<Vec<_>>();

    let chunks = activities.chunks(3).collect::<Vec<_>>();

    if activities.is_empty() {
        imp::send_error_message(
            ctx,
            original_msg,
            "No activities are currently scheduled in this server.",
        )
        .await?;
    } else {
        let list_page = chunks.get(page as usize);

        match list_page {
            Some(activities) => {
                let mut list_embed = CreateEmbed::default();

                for &activity in activities.iter() {
                    let members_string = if !activity.members.is_empty() {
                        activity
                            .members
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(idx, id)| {
                                if idx == 0 {
                                    Mention::from(id).to_string()
                                } else {
                                    let mention = Mention::from(id).to_string();
                                    format!(", {}", mention)
                                }
                            })
                            .collect::<String>()
                    } else {
                        String::from("None")
                    };

                    let alternate_string = if !activity.alternate.is_empty() {
                        activity
                            .alternate
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(idx, id)| {
                                if idx == 0 {
                                    Mention::from(id).to_string()
                                } else {
                                    let mention = Mention::from(id).to_string();
                                    format!(", {}", mention)
                                }
                            })
                            .collect::<String>()
                    } else {
                        String::from("None")
                    };

                    list_embed
                        .field("Activity:", &activity.name, true)
                        .field("Time:", &activity.date, true)
                        .field("Activity ID:", activity.id, true)
                        .field("Description:", &activity.description, false)
                        .field("Member List:", members_string, false)
                        .field("Alternate Members:", alternate_string, false);
                }

                list_embed
                    .title("Activity List")
                    .footer(|footer| footer.text(format!("Page {}/{}", page + 1, chunks.len())));

                original_msg
                    .channel_id
                    .send_message(ctx, |msg| {
                        msg.embed(|embed| {
                            *embed = list_embed;
                            embed
                        })
                    })
                    .await?;
            }
            None => {
                imp::send_error_message(
                    ctx,
                    original_msg,
                    format!(
                        "That page number is out of range. The maximum page number is {}",
                        chunks.len()
                    ),
                )
                .await?;
            }
        }
    }

    Ok(())
}

#[command]
#[description = "Create, update a user's markov chain information. Used for sending messages that sound like the specified user."]
async fn markov(ctx: &Context, original_msg: &Message, args: Args) -> CommandResult {
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
        "create" => markov_create(ctx, original_msg, args).await,
        "say" => markov_say(ctx, original_msg, args).await,
        _ => {
            imp::send_error_message(
                ctx,
                original_msg,
                "Invalid subcommand. Valid subcommands are `create` and `say`.",
            )
            .await?;
            Ok(())
        }
    }
}

async fn markov_create(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let user_id_opt = args
        .advance()
        .current()
        .map(|s| s.parse::<u64>().ok())
        .flatten();

    let user_id = match user_id_opt {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "Please provide a valid user ID.").await?;
            return Ok(());
        }
    };

    let alias = match args.advance().current() {
        Some(s) => String::from(s),
        None => {
            imp::send_error_message(
                ctx,
                original_msg,
                "Please provide an alias for this markov chain.",
            )
            .await?;
            return Ok(());
        }
    };

    let mut data_guard = ctx.data.write().await;
    let guild_map = data_guard
        .entry::<imp::data_keys::GetGuildData>()
        .or_default();
    let guild_data = guild_map.entry(guild_id.0).or_default();

    let mut info = MarkovInfo::new(user_id.into(), 1);

    guild_data
        .messages_mut()
        .entry(user_id.into())
        .or_default()
        .iter()
        .for_each(|msg| info.feed_str(&msg));

    guild_data.markov_mut().insert(alias, info);

    original_msg
        .channel_id
        .say(
            ctx,
            format!(
                "Markov info successfully generated for {}",
                Mention::from(UserId::from(user_id))
            ),
        )
        .await?;

    Ok(())
}

async fn markov_say(ctx: &Context, original_msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = match original_msg.guild_id {
        Some(id) => id,
        None => {
            imp::send_error_message(ctx, original_msg, "This command is not supported in DMs.")
                .await?;
            return Ok(());
        }
    };

    let alias = match args.advance().current() {
        Some(s) => String::from(s),
        None => {
            imp::send_error_message(ctx, original_msg, "Please provide an alias.").await?;
            return Ok(());
        }
    };

    let mut data_guard = ctx.data.write().await;
    let guild_map = data_guard
        .entry::<imp::data_keys::GetGuildData>()
        .or_default();
    let guild_data = guild_map.entry(guild_id.0).or_default();

    match guild_data.markov().get(&alias) {
        Some(info) => {
            let generated = info.gen_string();
            original_msg.channel_id.say(ctx, generated).await?;
        }
        None => {
            imp::send_error_message(
                ctx,
                original_msg,
                "That alias does not exist in this server.",
            )
            .await?;
        }
    }

    Ok(())
}
