use std::{
    convert::Infallible,
    fs::OpenOptions,
    io::prelude::*,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json as json;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ConfigPath {
    #[structopt(parse(from_os_str))]
    buf: PathBuf,
}

impl Default for ConfigPath {
    fn default() -> Self {
        Self {
            buf: PathBuf::from("./config/config.json"),
        }
    }
}

impl ToString for ConfigPath {
    fn to_string(&self) -> String {
        self.buf.to_string_lossy().into()
    }
}

impl FromStr for ConfigPath {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let buf = PathBuf::from(s);

        Ok(Self { buf })
    }
}

#[derive(Debug, StructOpt)]
pub enum ConfigMode {
    #[structopt(about = "Load the bot configuration from a file")]
    File {
        #[structopt(default_value)]
        path: ConfigPath,
    },

    #[structopt(about = "Load the bot configuration from command line arguments")]
    Cmd(BotConfig),
}

impl ConfigMode {
    pub fn into_config(self) -> anyhow::Result<BotConfig> {
        match self {
            ConfigMode::File { path } => BotConfig::from_path(&path.buf),
            ConfigMode::Cmd(cfg) => Ok(cfg),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
pub struct BotConfig {
    #[structopt(
        short,
        long,
        env = "DISCORD_TOKEN",
        help = "The Discord API token for the bot"
    )]
    pub token: String,

    #[structopt(short, long, help = "A list of owner IDs")]
    pub owner_ids: Option<Vec<u64>>,

    #[structopt(
        short = "d",
        long,
        help = "If present, the bot will respond to Direct Messages"
    )]
    pub allow_dm: bool,

    #[structopt(
        short = "b",
        long,
        help = "If present, will respond to other bots, including itself"
    )]
    pub allow_bots: bool,

    #[structopt(short, long, help = "The message prefix for bot commands")]
    pub prefix: String,

    #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "The location of the assets.json file"
    )]
    pub assets_file: PathBuf,

    #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "The location of the embeds.json file"
    )]
    pub embeds_file: PathBuf,
}

impl BotConfig {
    pub fn from_path<P: AsRef<Path>>(path: &P) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.as_ref())
            .with_context(|| {
                format!(
                    "Failed to read bot config from {:?}",
                    path.as_ref().as_os_str()
                )
            })?;

        Ok(json::from_reader(file)?)
    }

    pub fn gen_default_file<P: AsRef<Path>>(path: &P) -> anyhow::Result<Self> {
        let path_ref = path.as_ref();

        let mut default_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path_ref)?;

        let json_string = json::to_string_pretty(&Self::default())?;

        default_file.write_all(json_string.as_bytes())?;
        default_file.flush()?;

        Ok(Self::default())
    }
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            token: Default::default(),
            owner_ids: Default::default(),
            allow_dm: true,
            allow_bots: false,
            prefix: String::from("!"),
            assets_file: PathBuf::from("./config/assets.json"),
            embeds_file: PathBuf::from("./config/embeds.json"),
        }
    }
}
