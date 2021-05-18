use crate::command::imp;

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use serenity::{
    builder::CreateEmbed,
    collector::ReactionAction,
    model::{
        channel::ReactionType,
        prelude::{ChannelId, Embed, GuildId, Mention, Message, UserId},
    },
    prelude::Context,
};

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct GuildData {
    guild_id: GuildId,
    activities: HashMap<u64, Activity>,
}

impl GuildData {
    pub fn new(guild_id: GuildId) -> Self {
        Self {
            guild_id,
            activities: HashMap::new(),
        }
    }

    pub fn add_activity(&mut self, activity: Activity) -> Result<(), Activity> {
        if !self.activities.contains_key(&activity.id) {
            self.activities.insert(activity.id, activity);
            Ok(())
        } else {
            Err(activity)
        }
    }

    pub fn remove_activity(&mut self, id: u64) -> Option<Activity> {
        self.activities.remove(&id)
    }

    pub fn get_activity(&self, id: u64) -> Option<&Activity> {
        self.activities.get(&id)
    }

    pub fn get_activity_mut(&mut self, id: u64) -> Option<&mut Activity> {
        self.activities.get_mut(&id)
    }
}

#[non_exhaustive]
pub enum ActivityError {
    FireteamFull,
    MemberAlreadyInFireteam,
    MemberNotInFireteam,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Activity {
    name: String,
    description: String,
    date: String,
    id: u64,
    size: u8,
    members: HashSet<UserId>,
}

impl Activity {
    pub fn new<S: ToString>(name: S, description: S, date: S, id: u64, size: u8) -> Self {
        let name = name.to_string();
        let description = description.to_string();
        let date = date.to_string();

        Self {
            name,
            description,
            date,
            id,
            size,
            members: HashSet::with_capacity(size as usize),
        }
    }

    pub fn add_member(&mut self, member: UserId) -> Result<(), ActivityError> {
        if self.members.len() < self.size as usize {
            if self.members.insert(member) {
                Ok(())
            } else {
                Err(ActivityError::MemberAlreadyInFireteam)
            }
        } else {
            Err(ActivityError::FireteamFull)
        }
    }

    pub fn remove_member(&mut self, member: UserId) -> Result<(), ActivityError> {
        if self.members.remove(&member) {
            Ok(())
        } else {
            Err(ActivityError::MemberNotInFireteam)
        }
    }

    pub fn as_create_embed(&self, color: u32) -> CreateEmbed {
        let mut embed = CreateEmbed::default();

        let members_string = self
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
            .collect::<String>();

        embed
            .color(color)
            .field("Activity:", &self.name, true)
            .field("Time:", &self.date, true)
            .field("Activity ID:", self.id, true)
            .field("Description:", &self.description, false)
            .field("Fireteam Members:", members_string, false);

        embed
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmbedWithReactions {
    pub embed: Embed,
    pub reactions: Vec<ReactionType>,
}

impl EmbedWithReactions {
    pub async fn send_and_await_reaction(
        &self,
        ctx: &Context,
        channel: ChannelId,
        timeout: Option<Duration>,
        from_user: Option<UserId>,
    ) -> serenity::Result<Option<MessageReactionData>> {
        let embed_msg = channel
            .send_message(ctx, |msg| msg.set_embed(imp::create_embed(&self.embed)))
            .await?;

        for reaction_type in self.reactions.iter().cloned() {
            embed_msg.react(ctx, reaction_type).await?;
        }

        let mut collector = embed_msg.await_reaction(ctx);

        if let Some(id) = from_user {
            collector = collector.author_id(id);
        }

        if let Some(duration) = timeout {
            collector = collector.timeout(duration);
        }

        // Interacting with Discord's API can be... verbose at times.
        match collector.await {
            Some(action) => {
                match &*action {
                    ReactionAction::Added(reaction) => Ok(Some(MessageReactionData {
                        message: embed_msg,
                        reaction: reaction.emoji.clone(),
                    })),
                    ReactionAction::Removed(_) => {
                        channel.say(ctx, "You removed a reaction before the bot was ready. Please try again.").await?;
                        Ok(None)
                    }
                }
            }
            None => {
                match timeout {
                    Some(duration) => {
                        channel.say(
                            ctx,
                            format!("You did not add a reaction in time. Please react within {} seconds.", duration.as_secs()),
                        ).await?;
                    }
                    None => {
                        channel.say(
                            ctx,
                            "Some error occurred with getting a reaction. Please contact Factorial about this."
                        ).await?;
                    }
                }

                Ok(None)
            }
        }
    }
}

pub struct MessageReactionData {
    pub message: Message,
    pub reaction: ReactionType,
}
