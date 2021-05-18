use std::fmt::Display;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::{Embed, ReactionType};
use serenity::model::prelude::Message;
use serenity::prelude::Context;

pub(crate) fn create_embed(embed: &Embed) -> CreateEmbed {
    CreateEmbed::from(embed.clone())
}

pub(crate) async fn send_error_message<D: Display>(
    ctx: &Context,
    original_msg: &Message,
    content: D,
) -> CommandResult {
    original_msg.channel_id.say(ctx, content).await?;
    Ok(())
}

pub(crate) async fn add_reactions(
    ctx: &Context,
    msg: &Message,
    reactions: &[ReactionType],
) -> serenity::Result<()> {
    for reaction in reactions.iter().cloned() {
        msg.react(ctx, reaction).await?;
    }

    Ok(())
}

pub mod data_keys {
    use std::collections::HashMap;

    use serenity::{model::prelude::EmojiId, prelude::TypeMapKey};

    use crate::{client::JsonPaths, command::data::EmbedWithReactions};

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
}
