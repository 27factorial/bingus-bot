mod client;
mod command;
mod config;
mod util;

use structopt::StructOpt as _;

use crate::config::BotConfig;
use client::BotBuilder;
use config::ConfigMode;
use rand::Rng;
use serenity::model::prelude::UserId;
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
    let no_links = [
        "https://cdn.discordapp.com/attachments/761023449572311071/870117652000030731/az9og6Z_460swp.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875553389654712340/no-no-93.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875553414933798932/No.PNG",
        "https://cdn.discordapp.com/attachments/820186217974595595/875553470562836480/im_gonna_pretend_i_didnt_see_that.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875554649623646248/d518802fbfe0180d5c818f388e5979a8.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875554722390618163/unknown.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875554762437849159/Nogrumpycat-5ae79bb7c5542e00390dd621.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875554825809575946/110e3daa389718d1a33b751b62938dde.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875554907317489724/yellow-octopus-no-meme-6.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555461976449096/unknown.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555529097900092/395-3950272_no-stop-reaction-meme-memes-wtf-whatthehell-brendon.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555583011487774/4802887.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555645795991582/not-today-little-boy-meme.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555728784515112/yellow-octopus-no-meme-15.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555932652838942/962.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875555987199774740/f014e13efec361d2972e234d3dd1e6b792e8336b.png",
        "https://cdn.discordapp.com/attachments/820186217974595595/875556129822867496/How-About-No-Bear.png",
    ];

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

    if msg.content.to_ascii_lowercase().contains("bingus") {
        let res = msg.channel_id.say(&ctx, "hi").await;

        if let Err(e) = res {
            eprintln!("Error replying to bingus {:?}", e);
        }
    }

    if msg.author.id == UserId(213695908393517056) {
        let content = msg.content.to_ascii_lowercase();

        let arc_question = content.contains("gm")
            || content.contains("grandmaster")
            || content.contains("nightfall")
            || content.contains("nf");

        let index = rand::thread_rng().gen_range(0..no_links.len());

        if arc_question {
            let res = msg.reply(ctx, no_links[index]).await;

            if let Err(e) = res {
                eprintln!("Error replying to arc gm: {:?}", e);
            }
        }
    }

    if msg.content.to_ascii_lowercase().contains("linux") {
        let res = msg.reply(ctx, "https://preview.redd.it/ps4p9o323ub11.jpg?width=640&crop=smart&auto=webp&s=bd53639576973220c48940f8926d91349300950e").await;

        if let Err(e) = res {
            eprintln!("Error replying to linux: {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase().contains("soder") {
        let res = msg.reply(ctx, "https://youtu.be/p7LabSw36qs").await;

        if let Err(e) = res {
            eprintln!("Error replying to soder: {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase().contains("time") {
        let res = msg.reply(ctx, "Reset time. Same as usual. You can start the game though just to see the title screen.").await;

        if let Err(e) = res {
            eprintln!("Error replying to time: {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase() == "os" {
        let res = msg.reply(ctx, "https://cdn.discordapp.com/attachments/480613470367252500/854518024500936764/a9EyEKZ_460svvp9.webm").await;

        if let Err(e) = res {
            eprintln!("Error replying to os: {:?}", e);
        }
    }

    if msg.content.to_ascii_lowercase().contains("reddit") {
        let res = msg.reply(ctx, "reddit moment").await;

        if let Err(e) = res {
            eprintln!("Error replying to reddit: {:?}", e);
        }
    }
}
