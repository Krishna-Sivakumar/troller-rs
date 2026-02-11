use crate::{
    db::{DB, ProgressClock},
    dice::handle_dice_string,
    svg::render_progress_clock,
};
use futures::lock::Mutex;
use poise::serenity_prelude::futures::{self, Stream};
use poise::serenity_prelude::{CreateAttachment, CreateEmbed};

pub struct Data {
    pub db: Mutex<DB>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

static EMBED_OK_TUPLE: &'static (u8, u8, u8) = &(118, 164, 93);
static EMBED_ERR_TUPLE: &'static (u8, u8, u8) = &(159, 7, 18);

fn capitalize_string(input: &String) -> String {
    let words: Vec<String> = input
        .split_whitespace()
        .into_iter()
        .map(|word| {
            let mut w = word.to_lowercase().to_owned();
            w.replace_range(0..1, &w[0..1].to_uppercase());
            w
        })
        .collect();
    words.join(" ")
}

#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Dice string to roll."] dice_string: String,
    #[description = "Keep roll private?"] keep_private: Option<bool>,
) -> Result<(), Error> {
    let response = match handle_dice_string(dice_string) {
        Ok(valid_response) => CreateEmbed::new()
            .color(*EMBED_OK_TUPLE)
            .title("Roll Result")
            .fields(
                valid_response
                    .iter()
                    .map(|result| (result.name.clone(), result.value.clone(), false)),
            ),
        Err(_) => CreateEmbed::new()
            .color(*EMBED_ERR_TUPLE)
        .title("Roll Error")
        .field("", "The entered dice text was not valid. Take a look at the /help command for a guide on how to use the bot!", false)
    };
    ctx.send(poise::CreateReply {
        embeds: vec![response],
        ephemeral: keep_private,
        reply: true,
        ..Default::default()
    })
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn add_progress_clock(
    ctx: Context<'_>,
    #[description = "How many segments does the clock have?"] segments: u8,
    #[description = "How many segments are already filled?"] segments_filled: Option<u8>,
    #[description = "Delete clock after a day?"] ephemeral: Option<bool>,
    #[description = "What's the name of the clock?"] name: String,
    #[description = "What's the colour of the clock (html name or hex code)"] color: Option<String>,
    #[description = "Display now?"] display_now: Option<bool>,
) -> Result<(), Error> {
    let progress_clock = ProgressClock {
        namespace: ctx
            .guild()
            .map(|guild| guild.name.clone())
            .unwrap_or(ctx.author().name.clone()),
        name: name.clone(),
        segments: segments,
        segments_filled: segments_filled.unwrap_or(0),
        ephemeral: ephemeral.unwrap_or(false),
        color: color,
    };

    let db = ctx.data().db.lock().await;

    match db.save_clock(&progress_clock) {
        Ok(_) => {
            let reply_embed = CreateEmbed::new()
                .color(*EMBED_OK_TUPLE)
                .title("Created the progress clock!")
                .field("", format!("Created the clock {name}."), false);

            ctx.send(poise::CreateReply {
                embeds: vec![reply_embed],
                ephemeral: Some(true),
                ..Default::default()
            })
            .await?;

            match display_now {
                Some(true) => {
                    let png_data = render_progress_clock(&progress_clock)?;
                    ctx.send(poise::CreateReply {
                        embeds: vec![
                            CreateEmbed::new()
                                .title(capitalize_string(&progress_clock.name))
                                .image("attachment://clock.png")
                                .color(*EMBED_OK_TUPLE),
                        ],
                        attachments: vec![CreateAttachment::bytes(png_data, "clock.png")],
                        ephemeral: Some(false),
                        ..Default::default()
                    })
                    .await?;
                }
                _ => {}
            }
        }
        Err(e) => {
            println!("{}", e.to_string());
            ctx.send(poise::CreateReply {
                embeds: vec![CreateEmbed::new().color(*EMBED_ERR_TUPLE).field(
                    "Internal Error",
                    format!("Could not save your clock: {}", e),
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

#[poise::command(slash_command)]
pub async fn remove_progress_clock(
    ctx: Context<'_>,

    #[description = "Name of clock?"]
    #[autocomplete = "display_clock_name_autocomplete"]
    name: String,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    match db.remove_clock(
        &ctx.guild()
            .map(|guild| guild.name.clone())
            .unwrap_or(ctx.author().name.clone()),
        &name,
    ) {
        Ok(_) => {
            let reply_embed = CreateEmbed::new()
                .color(*EMBED_OK_TUPLE)
                .title("Removed clock.");

            ctx.send(poise::CreateReply {
                content: None,
                embeds: vec![reply_embed],
                ephemeral: Some(true),
                ..Default::default()
            })
            .await?;
        }
        Err(e) => {
            println!("{}", e.to_string());
            ctx.send(poise::CreateReply {
                embeds: vec![CreateEmbed::new().color(*EMBED_ERR_TUPLE).field(
                    "Internal Error",
                    "Could not remove your clock.",
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

#[poise::command(slash_command)]
pub async fn bump_progress_clock(
    ctx: Context<'_>,

    #[description = "Name of clock?"]
    #[autocomplete = "display_clock_name_autocomplete"]
    name: String,
    #[description = "Bump by how much?"] count: Option<u8>,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    match db.bump_clock(
        &ctx.guild()
            .map(|guild| guild.name.clone())
            .unwrap_or(ctx.author().name.clone()),
        &name,
        count.unwrap_or(1),
    ) {
        Ok(_) => {
            let progress_clock = db.get_clock(
                &ctx.guild()
                    .map(|guild| guild.name.clone())
                    .unwrap_or(ctx.author().name.clone()),
                &name,
            )?;

            let png_data = render_progress_clock(&progress_clock)?;

            ctx.send(poise::CreateReply {
                embeds: vec![
                    CreateEmbed::new()
                        .title(capitalize_string(&progress_clock.name))
                        .image("attachment://clock.png")
                        .color(*EMBED_OK_TUPLE),
                ],
                attachments: vec![CreateAttachment::bytes(png_data, "clock.png")],
                ephemeral: Some(false),
                ..Default::default()
            })
            .await?;
        }
        Err(e) => {
            println!("{}", e.to_string());
            ctx.send(poise::CreateReply {
                embeds: vec![CreateEmbed::new().color(*EMBED_ERR_TUPLE).field(
                    "Internal Error",
                    "Could not bump clock's count.",
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

pub async fn display_clock_name_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let db = ctx.data().db.lock().await;
    let items = ctx
        .guild()
        .map_or_else(
            || db.get_available_clocks(&ctx.author().name, partial),
            |namespace| db.get_available_clocks(&namespace.name, partial),
        )
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
pub async fn display_clock(
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
                embeds: vec![
                    CreateEmbed::new()
                        .title(capitalize_string(&progress_clock.name))
                        .image("attachment://clock.png")
                        .color(*EMBED_OK_TUPLE),
                ],
                attachments: attachment,
                ephemeral: Some(false),
                reply: false,
                ..Default::default()
            })
            .await?;
        }
        None => {
            ctx.send(poise::CreateReply {
                embeds: vec![CreateEmbed::new().color(*EMBED_OK_TUPLE).field(
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
