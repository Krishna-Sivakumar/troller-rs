mod db;
mod dice;
mod svg;

use crate::db::ProgressClock;
use crate::{db::DB, dice::handle_dice_string, svg::render_progress_clock};
use futures::lock::Mutex;
use poise::serenity_prelude::futures::{self, Stream};
use poise::serenity_prelude::{self as serenity, CreateAttachment, CreateEmbed};
use std::env::args;
use std::time::Instant;

struct Data {
    db: Mutex<DB>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn roll(
    ctx: Context<'_>,
    #[description = "Dice string to roll."] dice_string: String,
    #[description = "Keep roll private?"] keep_private: Option<bool>,
) -> Result<(), Error> {
    let now = Instant::now();
    let response = match handle_dice_string(dice_string) {
        Ok(valid_response) => CreateEmbed::new()
            .color((118, 164, 93))
            .title("Roll Result")
            .fields(
                valid_response
                    .iter()
                    .map(|result| (result.name.clone(), result.value.clone(), false)),
            ),
        Err(_) => CreateEmbed::new()
            .color((159, 7, 18))
        .title("Roll Error")
        .field("", "The entered dice text was not valid. Take a look at the /help command for a guide on how to use the bot!", false)
    };
    println!(
        "Took {}ms to handle dice command",
        now.elapsed().as_millis()
    );
    ctx.send(poise::CreateReply {
        content: None,
        embeds: vec![response],
        attachments: vec![],
        ephemeral: keep_private,
        components: None,
        allowed_mentions: None,
        reply: true,
        __non_exhaustive: (),
    })
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn add_progress_clock(
    ctx: Context<'_>,
    #[description = "How many segments does the clock have?"] segments: u8,
    #[description = "How many segments are already filled?"] segments_filled: Option<u8>,
    #[description = "Delete clock after a day?"] ephemeral: Option<bool>,
    #[description = "What's the name of the clock?"] name: String,
) -> Result<(), Error> {
    let _progress_clock = ProgressClock {
        namespace: ctx
            .guild()
            .map(|guild| guild.name.clone())
            .unwrap_or(ctx.author().name.clone()),
        name: name.clone(),
        segments: segments,
        segments_filled: segments_filled.unwrap_or(0),
        creation_date: 0,
        ephemeral: ephemeral.unwrap_or(false),
        color: None,
    };

    let _db = ctx.data().db.lock().await;
    let result: Result<(), Error> = Ok(()); // TODO add a put clock method and get the resultant from that meethod

    match result {
        Ok(_) => {
            let reply_embed = CreateEmbed::new()
                .color((118, 164, 93))
                .title("Created the progress clock!")
                .field("", format!("Created the clock {name}."), false);

            ctx.send(poise::CreateReply {
                content: None,
                embeds: vec![reply_embed],
                attachments: vec![],
                ephemeral: Some(true),
                ..Default::default()
            })
            .await?;
        }
        Err(_) => {
            ctx.send(poise::CreateReply {
                embeds: vec![CreateEmbed::new().color((159, 7, 18)).field(
                    "",
                    "Could not find the clock you were looking for.",
                    false,
                )],
                ephemeral: Some(true),
                reply: true,
                ..Default::default()
            })
            .await?;
        }
    }

    Ok(())
}

async fn display_clock_name_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let db = ctx.data().db.lock().await;
    let items = ctx
        .guild()
        .map(|namespace| db.get_available_clocks(&namespace.name, partial))
        .map(|progress_clocks_result| progress_clocks_result.unwrap_or(vec![]))
        .map(|progress_clocks| {
            progress_clocks
                .iter()
                .map(|pclock| pclock.name.clone())
                .collect()
        })
        .unwrap_or(vec![]);

    futures::stream::iter(items)
}

#[poise::command(slash_command)]
async fn display_clock(
    ctx: Context<'_>,
    #[description = "Name of clock?"]
    #[autocomplete = "display_clock_name_autocomplete"]
    name: String,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let items = ctx
        .guild()
        .map_or_else(
            || db.get_available_clocks(&ctx.author().name, ""),
            |guild| db.get_available_clocks(&guild.name, ""),
        )
        .unwrap_or_default();

    match items.iter().find(|item| (**item).name.cmp(&name).is_eq()) {
        Some(progress_clock) => {
            let attachment = match render_progress_clock(progress_clock) {
                Ok(png_data) => vec![CreateAttachment::bytes(png_data, "clock.png")],
                Err(_) => {
                    vec![]
                }
            };

            ctx.send(poise::CreateReply {
                attachments: attachment,
                ephemeral: Some(false),
                reply: false,
                ..Default::default()
            })
            .await?;
        }
        None => {
            ctx.send(poise::CreateReply {
                embeds: vec![CreateEmbed::new().color((159, 7, 18)).field(
                    "",
                    "Could not find the clock you were looking for.",
                    false,
                )],
                ephemeral: Some(true),
                reply: true,
                ..Default::default()
            })
            .await?;
        }
    };

    Ok(())
}

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
                        creation_date: 0,
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
    let intents = serenity::GatewayIntents::non_privileged();

    let database = Mutex::new(DB::new().expect("Could not initialize database."));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![roll(), add_progress_clock(), display_clock()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db: database })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client
        .expect("Could not build client.")
        .start()
        .await
        .expect("Could not start client.");
}
