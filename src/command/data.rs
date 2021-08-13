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

    pub fn activity(&self, id: u64) -> Option<&Activity> {
        self.activities.get(&id)
    }

    pub fn activity_mut(&mut self, id: u64) -> Option<&mut Activity> {
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
        if !self.alternate.contains(&member) {
            if self.members.len() < self.size as usize {
                if self.members.insert(member) {
                    Ok(())
                } else {
                    Err(ActivityError::MemberAlreadyInFireteam)
                }
            } else {
                Err(ActivityError::FireteamFull)
            }
        } else {
            let idx = self
                .alternate
                .iter()
                .position(|other_member| member == *other_member);

            match idx {
                Some(idx) => {
                    self.alternate.remove(idx);
                    self.members.insert(member);
                    Ok(())
                }
                None => Err(ActivityError::MemberNotInAlternate),
            }
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
            if self.alternate.len() < self.size as usize {
                self.members.remove(&member);
                self.alternate.push(member);
                Ok(())
            } else {
                Err(ActivityError::AlternateFull)
            }
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
                "Joining And Leaving:",
                format!(
                    "Use ~activity join {}, ~activity alt {}, or ~activity leave {} to \
                     join, join as an alternate, or leave an activity.",
                    self.id, self.id, self.id
                ),
                false,
            )
            .field(
                "Important Information:",
                "Bingus will ping you in this channel when your activity is ready. \
                 Please keep this channel unmuted.",
                false,
            )
            .footer(|footer| {
                footer.text("For command documentation, please see Bingus's GitHub page.")
            });

        embed
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum EmbedError {
    InvalidReaction,
    RemovedReaction,
    TimedOut,
    Other,
    Serenity(serenity::Error),
}

impl From<serenity::Error> for EmbedError {
    fn from(e: serenity::Error) -> Self {
        Self::Serenity(e)
    }
}

impl fmt::Display for EmbedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use EmbedError::*;

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

impl error::Error for EmbedError {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmbedWithMeta {
    pub embed: Embed,
    pub meta: Option<Vec<SelectionInfo>>,
}

impl EmbedWithMeta {
    pub async fn send_embed_chain(
        self,
        ctx: &Context,
        embed_map: &HashMap<String, EmbedWithMeta>,
        channel: ChannelId,
        timeout: Option<Duration>,
        from_user: Option<UserId>,
    ) -> Result<RosterData, EmbedError> {
        let mut embed_with_meta = self;
        let mut embed_msg = None;

        loop {
            if embed_msg.is_none() {
                embed_msg = Some(
                    channel
                        .send_message(ctx, |msg| {
                            msg.set_embed(imp::create_embed(&embed_with_meta.embed))
                        })
                        .await?,
                );
            } else if let Some(ref mut edited_msg) = embed_msg {
                edited_msg.delete_reactions(ctx).await?;

                edited_msg
                    .edit(ctx, |msg| {
                        msg.embed(|edited_embed| {
                            *edited_embed = imp::create_embed(&embed_with_meta.embed);
                            edited_embed
                        })
                    })
                    .await?;
            }

            let embed_msg = embed_msg.clone().ok_or(EmbedError::Other)?;
            let meta = embed_with_meta.meta.ok_or(EmbedError::Other)?;

            for selection_info in meta.iter() {
                embed_msg
                    .react(ctx, ReactionType::Unicode(selection_info.name.clone()))
                    .await?;
            }

            let mut collector = embed_msg.await_reaction(ctx);

            if let Some(id) = from_user {
                collector = collector.author_id(id);
            }

            if let Some(duration) = timeout {
                collector = collector.timeout(duration);
            }

            let selection_info =
                helpers::await_reaction(ctx, collector, embed_msg.clone(), timeout, meta).await?;

            match selection_info.kind {
                RosterKind::SelectNext(name) => {
                    embed_with_meta = embed_map
                        .get(&name)
                        .cloned()
                        .ok_or(EmbedError::InvalidReaction)?;
                }
                RosterKind::Finished {
                    activity_name,
                    size,
                } => {
                    return Ok(RosterData {
                        activity_name,
                        size,
                        message: embed_msg,
                        timeout,
                    })
                }
            }
        }
    }

    // pub async fn send_and_await_reaction(
    //     &self,
    //     ctx: &Context,
    //     channel: ChannelId,
    //     timeout: Option<Duration>,
    //     from_user: Option<UserId>,
    // ) -> Result<MessageData, ReactionError> {
    //     let embed_msg = channel
    //         .send_message(ctx, |msg| msg.set_embed(imp::create_embed(&self.embed)))
    //         .await?;
    //
    //     for selection_info in self.meta.iter() {
    //         embed_msg.react(ctx, selection_info.name).await?;
    //     }
    //
    //     let mut collector = embed_msg.await_reaction(ctx);
    //
    //     if let Some(id) = from_user {
    //         collector = collector.author_id(id);
    //     }
    //
    //     if let Some(duration) = timeout {
    //         collector = collector.timeout(duration);
    //     }
    //
    //     helpers::await_reaction(
    //         ctx,
    //         collector,
    //         embed_msg,
    //         timeout,
    //         from_user,
    //         embed_with_meta.meta,
    //     )
    //     .await
    // }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectionInfo {
    pub name: String,
    pub kind: RosterKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RosterKind {
    SelectNext(String),
    Finished { activity_name: String, size: u8 },
}

#[derive(Clone, Debug)]
pub struct RosterData {
    pub activity_name: String,
    pub size: u8,
    pub message: Message,
    pub timeout: Option<Duration>,
}

mod helpers {
    use super::*;
    use serenity::collector::CollectReaction;

    pub async fn await_reaction(
        ctx: &Context,
        collector: CollectReaction<'_>,
        message: Message,
        timeout: Option<Duration>,
        selections: Vec<SelectionInfo>,
    ) -> Result<SelectionInfo, EmbedError> {
        // Interacting with Discord's API can be... verbose at times.
        match collector.await {
            Some(action) => {
                match &*action {
                    ReactionAction::Added(reaction) => {
                        let name = reaction.emoji.as_data();
                        selections
                            .into_iter()
                            .find(|info| info.name == name)
                            .ok_or(EmbedError::InvalidReaction)
                    }
                    ReactionAction::Removed(_) => {
                        message.channel_id.say(ctx, "You removed a reaction before the bot was ready. Please try again.").await?;
                        Err(EmbedError::RemovedReaction)
                    }
                }
            }
            None => match timeout {
                Some(duration) => {
                    message.channel_id.say(
                        ctx,
                        format!("You did not add a reaction in time. Please react within {} seconds.", duration.as_secs()),
                    ).await?;
                    Err(EmbedError::TimedOut)
                }
                None => {
                    message.channel_id.say(
                        ctx,
                        "Some error occurred with getting a reaction. Please contact Factorial about this."
                    ).await?;
                    Err(EmbedError::Other)
                }
            },
        }
    }
}
