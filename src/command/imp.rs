use std::fmt::Display;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Embed;
use serenity::model::prelude::{Message, UserId};
use serenity::prelude::Context;

pub(crate) fn create_embed(embed: &Embed) -> CreateEmbed {
    CreateEmbed::from(embed.clone())
}

pub(crate) fn create_embed_owned(embed: Embed) -> CreateEmbed {
    CreateEmbed::from(embed)
}

pub(crate) async fn send_error_message<D: Display>(
    ctx: &Context,
    original_msg: &Message,
    content: D,
) -> CommandResult {
    original_msg.channel_id.say(ctx, content).await?;
    Ok(())
}

// pub(crate) async fn add_reactions(
//     ctx: &Context,
//     msg: &Message,
//     reactions: &[ReactionType],
// ) -> serenity::Result<()> {
//     for reaction in reactions.iter().cloned() {
//         msg.react(ctx, reaction).await?;
//     }
//
//     Ok(())
// }

pub(crate) async fn is_admin(ctx: &Context, user: UserId) -> serenity::Result<bool> {
    let type_map = ctx.data.read().await;
    let admins = type_map.get::<data_keys::GetAdmins>();

    match admins {
        Some(set) => Ok(set.contains(&user)),
        None => Ok(false),
    }
}

pub(crate) fn parse_date_time(date_time_str: &str) -> Option<(DateTime<FixedOffset>, String)> {
    let mut split_by_space = date_time_str.split(" ");

    let month_day_str = match split_by_space.next() {
        Some(string) => string,
        None => return None,
    };

    let mut month_day_year_iter = month_day_str.split("/");

    let month_value = {
        let value = month_day_year_iter
            .next()
            .map(|string| string.parse::<u32>().ok())
            .flatten();
        match value {
            Some(month) if month <= 12 => month,
            _ => return None,
        }
    };

    let day_value = {
        let value = month_day_year_iter
            .next()
            .map(|string| string.parse::<u32>().ok())
            .flatten();
        match value {
            Some(day) if day <= max_day_value(month_value) => day,
            _ => return None,
        }
    };

    let year_value = {
        let value = month_day_year_iter
            .next()
            .map(|string| string.parse::<i32>().ok())
            .flatten();
        match value {
            Some(year) if year > 2000 && year < 2100 => year,
            _ => return None,
        }
    };

    if month_day_year_iter.next().is_some() {
        return None;
    }

    let time_str = match split_by_space.next() {
        Some(string) => string,
        None => return None,
    };

    let mut time_iter = time_str.split(":");

    let mut hour_value = {
        let value = time_iter
            .next()
            .map(|string| string.parse::<u32>().ok())
            .flatten();
        match value {
            Some(hour) if hour > 0 && hour <= 12 => hour,
            _ => return None,
        }
    };

    let minute_value = {
        let value = time_iter
            .next()
            .map(|string| string.parse::<u32>().ok())
            .flatten();
        match value {
            Some(min) if min <= 59 => min,
            _ => return None,
        }
    };

    if time_iter.next().is_some() {
        return None;
    }

    let am_or_pm_opt = split_by_space
        .next()
        .map(|string| string.to_ascii_lowercase());

    let am_or_pm = match am_or_pm_opt.as_deref() {
        Some(time_of_day @ "am" | time_of_day @ "pm") => time_of_day,
        _ => return None,
    };

    if am_or_pm == "pm" {
        if hour_value != 12 {
            hour_value += 12
        }
    }

    if split_by_space.next().is_some() {
        return None;
    }

    let timezone = FixedOffset::west(4 * 3600);
    let naive_date = NaiveDate::from_ymd(year_value, month_value, day_value);
    let naive_time = NaiveTime::from_hms(hour_value, minute_value, 0) - timezone;
    let naive_date_time = NaiveDateTime::new(naive_date, naive_time);

    Some((
        DateTime::from_utc(naive_date_time, timezone),
        date_time_str.to_ascii_lowercase(),
    ))
}

fn max_day_value(month: u32) -> u32 {
    match month {
        1 => 31,
        2 => 28,
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => panic!("Invalid month value"),
    }
}

pub mod data_keys {
    use std::collections::{HashMap, HashSet};

    use serenity::{
        model::prelude::{EmojiId, UserId},
        prelude::TypeMapKey,
    };

    use crate::{client::JsonPaths, command::data::EmbedWithReactions, command::data::GuildData};

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct GetEmojiMap;

    impl TypeMapKey for GetEmojiMap {
        type Value = HashMap<String, EmojiId>;
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct GetEmbedMap;

    impl TypeMapKey for GetEmbedMap {
        type Value = HashMap<String, EmbedWithReactions>;
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct GetJsonPaths;

    impl TypeMapKey for GetJsonPaths {
        type Value = JsonPaths;
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct GetGuildData;

    impl TypeMapKey for GetGuildData {
        type Value = HashMap<u64, GuildData>;
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct GetAdmins;

    impl TypeMapKey for GetAdmins {
        type Value = HashSet<UserId>;
    }
}
