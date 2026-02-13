use std::{collections::HashMap, path::PathBuf};

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
    pub music_dir: PathBuf,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

static EMBED_OK_TUPLE: &'static (u8, u8, u8) = &(118, 164, 93);
static EMBED_ERR_TUPLE: &'static (u8, u8, u8) = &(159, 7, 18);

/// Breaks up a string and capitalizes every word.
fn capitalize_string(input: &str) -> String {
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

/// Returns an error-flavoured `CreateEmbed`  with a capitalized `title` and a `message`
fn create_error_embed<'a>(title: &'a str, message: &'a str) -> CreateEmbed {
    CreateEmbed::new()
        .color(*EMBED_ERR_TUPLE)
        .title(capitalize_string(title))
        .field("", message, false)
}

pub async fn help_command_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let name_refs: Vec<String> = ctx
        .framework()
        .options()
        .commands
        .iter()
        .map(|cmd| cmd.name.clone())
        .filter(|name| name.starts_with(partial))
        .collect();

    futures::stream::iter(name_refs)
}

/// Displays help text for commands in Troller.
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Name of the command"]
    #[autocomplete = "help_command_autocomplete"]
    command_name: String,
) -> Result<(), Error> {
    let cmd_descriptions = ctx
        .framework()
        .options()
        .commands
        .iter()
        .map(|cmd| (cmd.name.clone(), cmd.help_text.clone()))
        .filter(|(name, _)| name.cmp(&"help".to_owned()).is_ne())
        .fold(HashMap::new(), |mut map, (cmd_name, help_text)| {
            map.insert(cmd_name, help_text.unwrap_or_default());
            map
        });

    let embed = match cmd_descriptions.get(&command_name) {
        Some(help_text) => CreateEmbed::new()
            .color(*EMBED_OK_TUPLE)
            .title(format!("Help for `/{}`", command_name.clone()))
            .field("", help_text, false),
        None => create_error_embed(
            "Invalid command name",
            "The command you're looking for doesn't exist.",
        ),
    };

    ctx.send(poise::CreateReply {
        embeds: vec![embed],
        ephemeral: Some(true),
        reply: true,
        ..Default::default()
    })
    .await?;

    Ok(())
}

/// Roll dice using standard dice notation.
///
/// **Example Usage:**
/// `/roll 1d20 + 5`: Rolls a d20 and adds 5 to the result.
/// `/roll 1d20 + 6, 1d8 + 4`: Rolls a d20 and a d8 at the same time.
/// `/roll hit: 1d20 + 5`: Rolls dice and adds the name "hit" to the particular roll.
/// `/roll hit: 1d20 + 6, damage: 1d8 + 4`: Rolls the dice and attaches names to the rolls.
/// `/roll 2d20h1`: Rolls 2 d20s and takes the highest one.
/// `/roll 4d6h3`: Rolls 4 d6s and takes the highest three.
/// `/roll 2d20l1`: Rolls 2 d20s and takes the lowest one.
/// `/roll 5 * 3d6`: Multiplies 5 to the result of the 3d6 roll. It *does not* roll 15 sets of dice.
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
        Err(_) => create_error_embed(
            "Roll Error",
            "The entered dice text was not valid. Take a look at the /help command for a guide on how to use the bot!",
        ),
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

/// Create a new progress clock to track goals or countdowns.
///
/// **Example Usage:**
/// `/add_progress_clock segments:6 name:Escape Plan` - Create a 6-segment clock named "Escape Plan"
/// `/add_progress_clock segments:8 name:Ritual segments_filled:3 color:#FF5733 display_now:true` - Create an 8-segment clock with 3 segments already filled, custom color, and display immediately
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
                embeds: vec![create_error_embed(
                    "internal error",
                    &format!("Could not save your clock: {}", e),
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

/// Delete a progress clock.
///
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
                embeds: vec![create_error_embed(
                    "internal error",
                    "Could not remove your clock.",
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

/// Advance a progress clock by filling in more segments.
///
/// **Example Usage:**
/// `/bump_progress_clock name:Escape Plan` - Advance "Escape Plan" by 1 segment
/// `/bump_progress_clock name:Escape Plan count:2` - Advance "Escape Plan" by 2 segments
///
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
                embeds: vec![create_error_embed(
                    "internal error",
                    "Could not bump clock's count.",
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

/// Show an existing progress clock.
///
/// **Example Usage:**
/// `/display_clock name:Escape Plan` - Display the current state of "Escape Plan"
///
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

pub mod play_music {
    use std::sync::Arc;

    use crate::commands::{Data, EMBED_OK_TUPLE, Error, create_error_embed};
    use songbird::{
        TrackEvent,
        events::{Event, EventContext, EventHandler},
        tracks::PlayMode,
    };

    struct TrackErrorNotifier;

    #[serenity_prelude::async_trait]
    impl EventHandler for TrackErrorNotifier {
        async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
            if let EventContext::Track(track_list) = ctx {
                for (state, handle) in *track_list {
                    println!(
                        "Track {:?} encountered an error: {:?}",
                        handle.uuid(),
                        state.playing
                    );
                }
            }

            None
        }
    }

    use poise::serenity_prelude::{
        self, CreateEmbed,
        futures::{self, Stream},
    };
    type Context<'a> = poise::Context<'a, Data, Error>;

    pub async fn music_file_autocomplete<'a>(
        ctx: Context<'_>,
        partial: &'a str,
    ) -> impl Stream<Item = String> + 'a {
        let completions: std::io::Result<Vec<String>> = ctx
            .data()
            .music_dir
            .read_dir()
            .map(|entries| {
                entries.map(|entry| match entry {
                    Ok(entry) => entry
                        .path()
                        .strip_prefix(&ctx.data().music_dir)
                        .expect("Could not strip prefix.")
                        .to_str()
                        .expect("Couldn't convert non-utf8 path to string.")
                        .to_owned(),
                    Err(_) => String::new(),
                })
            })
            .map(|paths| {
                paths
                    .filter(|path| path.len() > 0 && path.starts_with(partial))
                    .take(15)
                    .collect()
            });
        futures::stream::iter(completions.unwrap_or(vec![]))
    }

    #[poise::command(slash_command)]
    pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
        let not_in_vc_error = create_error_embed(
            "Not in a voice chat.",
            "Troller is not in a voice chat in this guild.",
        );

        let manager = songbird::get(&ctx.serenity_context())
            .await
            .expect("could not find serenity manager.");

        let embed = match ctx.guild_id() {
            Some(guild_id) => {
                let has_handler = manager.get(guild_id).is_some();
                if has_handler {
                    if let Err(e) = manager.remove(guild_id).await {
                        create_error_embed(
                            "Could not leave voice channel",
                            &format!("Could not leave voice channel due to this error: {e}"),
                        )
                    } else {
                        CreateEmbed::new()
                            .color(*EMBED_OK_TUPLE)
                            .title("Left voice channel.")
                            .field("", "Successfully left the voice channel.", false)
                    }
                } else {
                    not_in_vc_error
                }
            }
            None => not_in_vc_error,
        };

        ctx.send(poise::CreateReply {
            embeds: vec![embed],
            ephemeral: Some(true),
            reply: true,
            ..Default::default()
        })
        .await?;

        Ok(())
    }

    #[poise::command(slash_command)]
    pub async fn file(
        ctx: Context<'_>,
        #[description = "pick file"]
        #[autocomplete = "music_file_autocomplete"]
        filename: String,
    ) -> Result<(), Error> {
        let mut path = ctx.data().music_dir.clone();
        path.push(&filename);

        let not_in_vc_error = create_error_embed(
            "Not in a voice chat.",
            "Troller is not in a voice chat in this guild.",
        );

        ctx.defer_ephemeral().await?;

        let manager = songbird::get(&ctx.serenity_context())
            .await
            .expect("could not find serenity manager.");

        let embed = match ctx.guild_id() {
            Some(guild_id) => {
                let channel_id = ctx
                    .guild()
                    .unwrap()
                    .voice_states
                    .get(&ctx.author().id)
                    .and_then(|voice_state| voice_state.channel_id);

                match channel_id {
                    Some(channel_id) => {
                        let call = manager.join(guild_id, channel_id).await?;
                        let mut inner_call = call.lock().await;

                        inner_call
                            .add_global_event(Event::Track(TrackEvent::Error), TrackErrorNotifier);

                        let file = songbird::input::File::new(path);

                        inner_call.queue().stop();

                        let handle = inner_call.enqueue_input(file.into());
                        handle.await.play()?;

                        not_in_vc_error
                    }
                    None => not_in_vc_error,
                }
            }
            _ => not_in_vc_error,
        };

        ctx.send(poise::CreateReply {
            embeds: vec![embed],
            ephemeral: Some(true),
            reply: true,
            ..Default::default()
        })
        .await?;

        Ok(())
    }

    /// If a track is playing, toggle its paused state.
    #[poise::command(slash_command)]
    pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
        let not_in_vc_error = create_error_embed(
            "Not in a voice chat.",
            "Troller is not in a voice chat in this guild.",
        );

        ctx.defer_ephemeral().await?;

        let manager = songbird::get(&ctx.serenity_context())
            .await
            .expect("could not find serenity manager.");

        let embed = match ctx.guild_id() {
            Some(guild_id) => match manager.get(guild_id) {
                Some(manager) => {
                    let inner_call = manager.lock().await;
                    match inner_call.queue().current() {
                        Some(track_handle) => {
                            match track_handle.get_info().await.unwrap().playing {
                                PlayMode::Play => {
                                    track_handle.pause()?;
                                }
                                PlayMode::Pause => {
                                    track_handle.play()?;
                                }
                                _ => {}
                            }
                        }
                        None => {
                            println!("no current tracks");
                        }
                    }

                    not_in_vc_error // TODO fix these embeds
                }
                None => not_in_vc_error,
            },
            _ => not_in_vc_error,
        };

        ctx.send(poise::CreateReply {
            embeds: vec![embed],
            ephemeral: Some(true),
            reply: true,
            ..Default::default()
        })
        .await?;

        Ok(())
    }

    #[poise::command(
        slash_command,
        subcommand_required,
        subcommands("file", "leave", "pause"),
        guild_only
    )]
    pub async fn music(_: Context<'_>) -> Result<(), Error> {
        Ok(())
    }
}
