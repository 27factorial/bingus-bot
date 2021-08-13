use futures::future::BoxFuture;
use serde_json::{self as json, Map as JsonMap, Value as JsonValue};
use serenity::{
    framework::standard::{CommandGroup, StandardFramework},
    model::{
        gateway::Ready,
        prelude::{EmojiId, Message, UserId},
    },
    prelude::{Context, EventHandler, TypeMap},
    Client,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;

use crate::command::{data::EmbedWithMeta, imp::data_keys};

use crate::config::BotConfig;

pub async fn initialize_emoji_map(paths: &JsonPaths, type_map: &mut TypeMap) {
    let open = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&paths.assets)
        .await;

    match open {
        Ok(file) => {
            let result = json::from_reader::<_, JsonMap<String, JsonValue>>(file.into_std().await);

            match result {
                Ok(mut json_map) => {
                    if let Some(JsonValue::Object(emojis)) = json_map.remove("emojis") {
                        let mut emoji_map = HashMap::with_capacity(emojis.len());

                        for (name, val) in emojis.into_iter() {
                            if let Some(id) = val.as_u64() {
                                emoji_map.insert(name, EmojiId::from(id));
                            } else {
                                eprintln!("[ERR] JSON field {} is not a u64.", name);
                            }
                        }

                        type_map.insert::<data_keys::GetEmojiMap>(emoji_map);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[ERR] JSON deserialization error. Expected Object, got error: {}",
                        e
                    );
                }
            }
        }
        Err(e) => {
            eprintln!(
                "[ERR] Unable to read assets file {}. Error: {:?}",
                &paths.assets.to_string_lossy(),
                e
            );
        }
    }
}

pub async fn initialize_embed_map(paths: &JsonPaths, type_map: &mut TypeMap) {
    let open = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&paths.embeds)
        .await;

    match open {
        Ok(file) => {
            let result =
                json::from_reader::<_, HashMap<String, EmbedWithMeta>>(file.into_std().await);

            match result {
                Ok(map) => type_map.insert::<data_keys::GetEmbedMap>(map),
                Err(e) => {
                    eprintln!(
                        "[ERR] JSON deserialization error. Expected Map, got error: {}",
                        e
                    );
                }
            }
        }
        Err(e) => {
            eprintln!(
                "[ERR] Unable to read embeds file {}. Error: {:?}",
                &paths.embeds.to_string_lossy(),
                e
            );
        }
    }
}

async fn push_paths(paths: JsonPaths, type_map: &mut TypeMap) {
    type_map.insert::<data_keys::GetJsonPaths>(paths);
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
struct Handler {
    assets_file_path: PathBuf,
    embeds_file_path: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct JsonPaths {
    assets: PathBuf,
    embeds: PathBuf,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bingus is ready and connected to Discord.");
        println!("Initializing bot data.");
        eprintln!("Ready info: {:?}", ready);

        let paths = JsonPaths {
            assets: self.assets_file_path.clone(),
            embeds: self.embeds_file_path.clone(),
        };

        let mut type_map = ctx.data.write().await;

        initialize_emoji_map(&paths, &mut type_map).await;
        initialize_embed_map(&paths, &mut type_map).await;
        push_paths(paths, &mut type_map).await;
    }
}

#[derive(Default)]
pub struct BotClient {
    token: String,
    event_handler: Handler,
    framework: StandardFramework,
}

impl BotClient {
    // pub fn builder<T: Into<String>>(token: T) -> BotBuilder {
    //     BotBuilder::new(token)
    // }

    pub async fn start(self) -> serenity::Result<()> {
        let mut client = Client::builder(self.token)
            .event_handler(self.event_handler)
            .framework(self.framework)
            .await?;

        client.start().await
    }
}

#[derive(Clone, Default)]
pub struct BotBuilder {
    token: String,
    owner_ids: Option<HashSet<UserId>>,
    allow_dm: Option<bool>,
    ignore_bots: Option<bool>,
    prefix: Option<String>,
    assets_file_path: Option<PathBuf>,
    embeds_file_path: Option<PathBuf>,
    message_handler: Option<for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()>>,
    command_groups: Option<Vec<&'static CommandGroup>>,
}

impl BotBuilder {
    pub fn new<T: Into<String>>(token: T) -> Self {
        Self {
            token: token.into(),
            owner_ids: None,
            allow_dm: None,
            ignore_bots: None,
            prefix: None,
            assets_file_path: None,
            embeds_file_path: None,
            message_handler: None,
            command_groups: None,
        }
    }

    pub fn from_config(config: BotConfig) -> Self {
        let builder = Self::new(config.token)
            .allow_dm(config.allow_dm)
            .ignore_bots(!config.allow_dm)
            .prefix(config.prefix)
            .assets_file(&config.assets_file)
            .embeds_file(&config.embeds_file);

        match config.owner_ids {
            Some(ids) => builder.owners(ids),
            None => builder,
        }
    }

    // pub fn owner<T: Into<UserId>>(mut self, id: T) -> Self {
    //     match self.owner_ids {
    //         Some(ref mut set) => {
    //             set.insert(id.into());
    //         }
    //         None => {
    //             let mut owner_ids = HashSet::with_capacity(1);
    //             owner_ids.insert(id.into());
    //             self.owner_ids = Some(owner_ids);
    //         }
    //     }
    //
    //     self
    // }

    pub fn owners<T: Into<UserId>>(mut self, ids: Vec<T>) -> Self {
        match self.owner_ids {
            Some(ref mut set) => {
                for id in ids {
                    set.insert(id.into());
                }
            }
            None => {
                let mut owner_ids = HashSet::with_capacity(1);
                for id in ids {
                    owner_ids.insert(id.into());
                }
                self.owner_ids = Some(owner_ids);
            }
        }

        self
    }

    pub fn allow_dm(mut self, allow_dm: bool) -> Self {
        self.allow_dm = Some(allow_dm);
        self
    }

    pub fn ignore_bots(mut self, ignore_bots: bool) -> Self {
        self.ignore_bots = Some(ignore_bots);
        self
    }

    pub fn prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn assets_file<P: AsRef<Path>>(mut self, path: &P) -> Self {
        let path = path.as_ref();
        self.assets_file_path = Some(PathBuf::from(path));
        self
    }

    pub fn embeds_file<P: AsRef<Path>>(mut self, path: &P) -> Self {
        let path = path.as_ref();
        self.embeds_file_path = Some(PathBuf::from(path));
        self
    }

    pub fn message_handler(
        mut self,
        f: for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()>,
    ) -> Self {
        self.message_handler = Some(f);
        self
    }

    // pub fn group(mut self, group: &'static CommandGroup) -> Self {
    //     match self.command_groups {
    //         Some(ref mut command_groups) => {
    //             command_groups.push(group);
    //         }
    //         None => {
    //             let mut command_groups = Vec::with_capacity(1);
    //             command_groups.push(group);
    //             self.command_groups = Some(command_groups);
    //         }
    //     }
    //
    //     self
    // }

    pub fn group_slice(mut self, slice: &[&'static CommandGroup]) -> Self {
        match self.command_groups {
            Some(ref mut command_groups) => {
                command_groups.extend_from_slice(slice);
            }
            None => {
                let mut command_groups = Vec::with_capacity(slice.len());
                command_groups.extend_from_slice(slice);
                self.command_groups = Some(command_groups);
            }
        }

        self
    }

    pub fn build(self) -> BotClient {
        let allow_dm = match self.allow_dm {
            Some(val) => val,
            None => false,
        };

        let ignore_bots = match self.ignore_bots {
            Some(val) => val,
            None => true,
        };

        let prefix = match self.prefix {
            Some(prefix) => prefix,
            None => String::from("!"),
        };

        let command_groups = match self.command_groups {
            Some(cg) => cg,
            None => Vec::new(),
        };

        let owner_ids = match self.owner_ids {
            Some(ids) => ids,
            None => HashSet::new(),
        };

        let mut framework = StandardFramework::new().configure(|c| {
            c.allow_dm(allow_dm)
                .ignore_bots(ignore_bots)
                .prefix(&prefix)
                .owners(owner_ids)
        });

        for group in command_groups {
            framework.group_add(group);
        }

        // This is ugly, but required because the StandardFramework doesn't have a method
        // to register a normal message handler without returning Self.
        let framework_with_handler = match self.message_handler {
            Some(handler) => framework.normal_message(handler),
            None => framework,
        };

        let event_handler = Handler {
            assets_file_path: self.assets_file_path.unwrap_or_default(),
            embeds_file_path: self.embeds_file_path.unwrap_or_default(),
        };

        BotClient {
            token: self.token,
            event_handler,
            framework: framework_with_handler,
        }
    }
}
