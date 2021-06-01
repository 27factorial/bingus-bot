mod client;
mod command;
mod config;

use structopt::StructOpt as _;

use crate::config::BotConfig;
use client::BotBuilder;
use config::ConfigMode;
use serenity::{
    framework::standard::{macros::hook, CommandGroup},
    model::prelude::Message,
    prelude::Context,
};

static GROUPS: &'static [&'static CommandGroup] = &[
    &command::general::GENERAL_GROUP,
    &command::owner::OWNERSONLY_GROUP,
    &command::admin::ADMINSONLY_GROUP,
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_mode: ConfigMode = ConfigMode::from_args_safe()?;
    let bot_config = match config_mode.into_config() {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Could not find config file. Generating default config.");
            BotConfig::gen_default_file(&"./config/config.json")?
        }
    };

    Ok(BotBuilder::from_config(bot_config)
        .message_handler(handle_normal)
        .group_slice(GROUPS)
        .build()
        .start()
        .await?)
}

#[hook]
async fn handle_normal(ctx: &Context, msg: &Message) {
    if msg.content.to_ascii_lowercase().contains("sompies") {
        let res = msg
            .channel_id
            .say(&ctx, "sompies to be removed fro mgame")
            .await;

        if let Err(e) = res {
            eprintln!("Error replying to sompies: {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase().contains("monke") {
        let res = msg
            .channel_id
            .say(&ctx, "https://youtu.be/XZ5Uv4JKTU4")
            .await;

        if let Err(e) = res {
            eprintln!("Error replying to monke {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase().contains("admin") {
        let res = msg
            .channel_id
            .say(&ctx, "https://cdn.discordapp.com/attachments/754073658842153112/763105918899847198/savetweetvid_EVypQCTXgAATeNQ.gif")
            .await;

        if let Err(e) = res {
            eprintln!("Error replying to admin {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase().contains("bingus") {
        let res = msg.channel_id.say(&ctx, "hi").await;

        if let Err(e) = res {
            eprintln!("Error replying to bingus {:?}", e);
        }
    }
}
