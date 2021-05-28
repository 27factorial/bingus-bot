use crate::command::imp;

use std::{
    collections::{HashMap, HashSet},
    error, fmt,
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
use std::fmt::Formatter;

#[derive(Clone, Debug, Default)]
pub struct GuildData {
    guild_id: GuildId,
    activities: HashMap<u64, Activity>,
    next_activity_id: u64,
    free_activity_ids: Vec<u64>,
}

impl GuildData {
    pub fn new(guild_id: GuildId) -> Self {
        Self {
            guild_id,
            activities: HashMap::new(),
            next_activity_id: 0,
            free_activity_ids: Vec::new(),
        }
    }

    pub fn add_activity(&mut self, activity: Activity) -> Result<(), Activity> {
        if !self.activities.contains_key(&activity.id) {
            let idx_opt = self
                .free_activity_ids
                .iter()
                .copied()
                .enumerate()
                .find(|(_, id)| *id == activity.id)
                .map(|(idx, _)| idx);

            match idx_opt {
                Some(idx) => {
                    self.free_activity_ids.remove(idx);
                }
                None => self.next_activity_id += 1,
            }

            self.activities.insert(activity.id, activity);

            Ok(())
        } else {
            Err(activity)
        }
    }

    pub fn remove_activity(&mut self, id: u64) -> Option<Activity> {
        if self.activities.contains_key(&id) {
            self.free_activity_ids.push(id);
            self.activities.remove(&id)
        } else {
            None
        }
    }

    pub fn get_activity(&self, id: u64) -> Option<&Activity> {
        self.activities.get(&id)
    }

    pub fn get_activity_mut(&mut self, id: u64) -> Option<&mut Activity> {
        self.activities.get_mut(&id)
    }

    pub fn activity_id(&self) -> u64 {
        self.free_activity_ids
            .first()
            .copied()
            .unwrap_or(self.next_activity_id)
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActivityError {
    FireteamFull,
    MemberAlreadyInFireteam,
    MemberNotInFireteam,
    AlternateFull,
    MemberAlreadyInAlternate,
    MemberNotInAlternate,
}

#[derive(Clone, Debug)]
pub struct Activity {
    pub name: String,
    pub description: String,
    pub date: String,
    pub id: u64,
    pub size: u8,
    pub creator: UserId,
    pub embed_msg: Message,
    pub members: HashSet<UserId>,
    pub alternate: Vec<UserId>,
}

impl Activity {
    pub fn new<S: ToString>(
        name: S,
        description: S,
        date: S,
        id: u64,
        size: u8,
        creator: UserId,
        embed_msg: Message,
    ) -> Self {
        let name = name.to_string();
        let description = description.to_string();
        let date = date.to_string();

        Self {
            name,
            description,
            date,
            id,
            size,
            creator,
            embed_msg,
            members: HashSet::with_capacity(size as usize),
            alternate: Vec::with_capacity(size as usize),
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

    pub fn add_member_alt(&mut self, member: UserId) -> Result<(), ActivityError> {
        if !self.members.contains(&member) {
            if self.alternate.len() < self.size as usize {
                if !self.alternate.contains(&member) {
                    self.alternate.push(member);
                    Ok(())
                } else {
                    Err(ActivityError::MemberAlreadyInAlternate)
                }
            } else {
                Err(ActivityError::AlternateFull)
            }
        } else {
            Err(ActivityError::MemberAlreadyInFireteam)
        }
    }

    pub fn remove_member(&mut self, member: UserId) -> Result<(), ActivityError> {
        if self.members.remove(&member) {
            Ok(())
        } else {
            Err(ActivityError::MemberNotInFireteam)
        }
    }

    pub fn remove_member_alt(&mut self, member: UserId) -> Result<(), ActivityError> {
        let idx = self
            .alternate
            .iter()
            .position(|other_member| member == *other_member);

        match idx {
            Some(idx) => {
                self.alternate.remove(idx);
                Ok(())
            }
            None => Err(ActivityError::MemberNotInAlternate),
        }
    }

    pub fn as_create_embed(&self, color: u32) -> CreateEmbed {
        let mut embed = CreateEmbed::default();

        let members_string = if !self.members.is_empty() {
            self.members
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

        let alternate_string = if !self.alternate.is_empty() {
            self.alternate
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

        embed
            .color(color)
            .title("Destiny 2 Activity Roster")
            .field("Activity:", &self.name, true)
            .field("Time:", &self.date, true)
            .field("Activity ID:", self.id, true)
            .field("Description:", &self.description, false)
            .field("Fireteam Members:", members_string, false)
            .field("Alternate Members:", alternate_string, false)
            .field(
                "Important Information:",
                "Bingus will ping you in this channel when your activity is ready. \
                 Please keep this channel unmuted.",
                false,
            )
            .footer(|footer| {
                footer.text(format!(
                    "Use the command `~activity join {}` to join this activity.",
                    self.id
                ))
            });

        embed
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ReactionError {
    InvalidReaction,
    RemovedReaction,
    TimedOut,
    Other,
    Serenity(serenity::Error),
}

impl From<serenity::Error> for ReactionError {
    fn from(e: serenity::Error) -> Self {
        Self::Serenity(e)
    }
}

impl fmt::Display for ReactionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ReactionError::*;

        let display = match self {
            InvalidReaction => "Invalid reaction".into(),
            RemovedReaction => "Removed reaction".into(),
            TimedOut => "Timed out".into(),
            Other => "Other".into(),
            Serenity(e) => format!("Serenity error: {}", e),
        };

        write!(f, "{}", display)
    }
}

impl error::Error for ReactionError {}

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
        message_name: String,
    ) -> Result<MessageData, ReactionError> {
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

        helpers::await_reaction(ctx, collector, embed_msg, timeout, from_user, message_name).await
    }
}

pub struct MessageData {
    pub message: Message,
    pub reaction: ReactionType,
    pub timeout: Option<Duration>,
    pub user: Option<UserId>,
    pub message_name: String,
}

impl MessageData {
    pub async fn edit_and_await_reaction<F>(
        mut self,
        ctx: &Context,
        f: F,
    ) -> Result<Self, ReactionError>
    where
        F: FnOnce(ReactionType) -> Option<(EmbedWithReactions, String)>,
    {
        let (next_embed, reactions, message_name) = match f(self.reaction) {
            Some((data, name)) => (data.embed, data.reactions, name),
            None => {
                self.message
                    .channel_id
                    .say(
                        ctx,
                        "Invalid reaction. Please react with one of the listed reactions.",
                    )
                    .await?;
                return Err(ReactionError::InvalidReaction);
            }
        };

        // Remove the current reactions.
        self.message.delete_reactions(ctx).await?;
        self.message
            .edit(ctx, |msg| {
                msg.embed(|embed| {
                    *embed = imp::create_embed_owned(next_embed);
                    embed
                })
            })
            .await?;

        for reaction_type in reactions.into_iter() {
            self.message.react(ctx, reaction_type).await?;
        }

        let mut collector = self.message.await_reaction(ctx);

        if let Some(id) = self.user {
            collector = collector.author_id(id);
        }

        if let Some(duration) = self.timeout {
            collector = collector.timeout(duration);
        }

        helpers::await_reaction(
            ctx,
            collector,
            self.message,
            self.timeout,
            self.user,
            message_name,
        )
        .await
    }
}

mod helpers {
    use super::*;
    use serenity::collector::CollectReaction;

    pub async fn await_reaction(
        ctx: &Context,
        collector: CollectReaction<'_>,
        message: Message,
        timeout: Option<Duration>,
        user: Option<UserId>,
        message_name: String,
    ) -> Result<MessageData, ReactionError> {
        // Interacting with Discord's API can be... verbose at times.
        match collector.await {
            Some(action) => {
                match &*action {
                    ReactionAction::Added(reaction) => Ok(MessageData {
                        message,
                        reaction: reaction.emoji.clone(),
                        timeout,
                        user,
                        message_name,
                    }),
                    ReactionAction::Removed(_) => {
                        message.channel_id.say(ctx, "You removed a reaction before the bot was ready. Please try again.").await?;
                        Err(ReactionError::RemovedReaction)
                    }
                }
            }
            None => match timeout {
                Some(duration) => {
                    message.channel_id.say(
                            ctx,
                            format!("You did not add a reaction in time. Please react within {} seconds.", duration.as_secs()),
                        ).await?;
                    Err(ReactionError::TimedOut)
                }
                None => {
                    message.channel_id.say(
                            ctx,
                            "Some error occurred with getting a reaction. Please contact Factorial about this."
                        ).await?;
                    Err(ReactionError::Other)
                }
            },
        }
    }
}
