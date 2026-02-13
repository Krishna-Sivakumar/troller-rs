mod commands;
mod db;
mod dice;
mod svg;

use crate::db::ProgressClock;
use crate::{db::DB, dice::handle_dice_string, svg::render_progress_clock};
use commands::*;
use futures::lock::Mutex;
use poise::serenity_prelude::futures::{self};
use poise::serenity_prelude::{self as serenity};
use songbird::SerenityInit;
use std::env::args;

#[allow(unused)]
fn cli() {
    let mut arg_iter = args().skip(1);
    match arg_iter.next() {
        Some(command_name) => {
            match command_name.as_str() {
                "roll" => {
                    let dice_string: String =
                        arg_iter.next().expect("USAGE: troller roll [dice_string]");

                    match handle_dice_string(dice_string) {
                        Err(err) => println!("{err}"),
                        Ok(_) => {}
                    };
                }
                "clock" => {
                    static HELP_STRING: &str =
                        "USAGE: troller clock segments:[1-255] segments-filled:[1-255]";
                    let segments: u8 = arg_iter
                        .next()
                        .expect(HELP_STRING)
                        .parse()
                        .expect(HELP_STRING);
                    let segments_filled: u8 = arg_iter
                        .next()
                        .expect(HELP_STRING)
                        .parse()
                        .expect("HELP_STRING");
                    render_progress_clock(&ProgressClock {
                        namespace: "".to_owned(),
                        name: "".to_owned(),
                        segments,
                        segments_filled,
                        ephemeral: false,
                        color: None,
                    })
                    .expect("Could not render progress clock.");
                    println!("wrote progress clock to out.png.");
                }
                _ => {
                    println!("Invalid Command.");
                }
            };
        }
        None => {
            println!("USAGE: troller [roll | clock]");
        }
    }
}

#[tokio::main]
async fn main() {
    let token =
        std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN environment variable.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let database = Mutex::new(DB::new().expect("Could not initialize database."));

    let music_dir = std::env::var_os("MUSIC_DIR")
        .expect("no MUSIC_DIR variable set in the environment.")
        .to_owned();
    let music_dir = std::path::PathBuf::from(music_dir);
    music_dir
        .try_exists()
        .expect("MUSIC_DIR environment variable does not point to a valid path.");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                roll(),
                add_progress_clock(),
                display_clock(),
                remove_progress_clock(),
                bump_progress_clock(),
                help(),
                play_music::music(),
                play_music::leave(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    db: database,
                    music_dir,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await;

    client
        .expect("Could not build client.")
        .start()
        .await
        .expect("Could not start client.");
}
