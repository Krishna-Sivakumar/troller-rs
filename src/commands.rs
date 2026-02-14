use std::{collections::HashMap, path::PathBuf, sync::Arc};

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
    pub track_list: Arc<Mutex<Vec<String>>>,
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
            let mut w = word.to_lowercase();
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

fn create_quick_success_embed<'a>(title: &'a str, message: &'a str) -> CreateEmbed {
    CreateEmbed::new()
        .color(*EMBED_OK_TUPLE)
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

/// Display help text and usage examples for any Troller command.
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
        Some(help_text) => {
            create_quick_success_embed(&format!("Help for `/{}`", command_name.clone()), &help_text)
        }
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

    use crate::commands::{Data, Error, create_error_embed, create_quick_success_embed};
    use songbird::{
        Call, TrackEvent,
        events::{Event, EventContext, EventHandler},
        tracks::LoopState,
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
        self,
        futures::{self, Stream},
    };
    use tokio::sync::MutexGuard;
    type Context<'a> = poise::Context<'a, Data, Error>;

    async fn perform_call_action<F, A, B>(
        ctx: Context<'_>,
        join_if_absent: bool,
        operation: F,
        args: B,
    ) -> Result<A, Error>
    where
        F: AsyncFn(
            (
                Arc<songbird::Songbird>,
                MutexGuard<'_, Call>,
                poise::serenity_prelude::GuildId,
            ),
            B,
        ) -> Result<A, Error>,
    {
        let manager = songbird::get(&ctx.serenity_context())
            .await
            .expect("Could not find serenity maanger");
        match ctx.guild_id() {
            Some(guild_id) => match manager.get(guild_id) {
                Some(call) => {
                    let inner_call = call.lock().await;
                    operation((manager, inner_call, guild_id), args).await
                }

                None => {
                    if join_if_absent {
                        let channel_id = ctx
                            .guild()
                            .unwrap()
                            .voice_states
                            .get(&ctx.author().id)
                            .and_then(|voice_state| voice_state.channel_id);

                        match channel_id {
                            Some(channel_id) => {
                                let call = manager.join(guild_id, channel_id).await?;
                                let inner_call = call.lock().await;
                                operation((manager, inner_call, guild_id), args).await
                            }
                            None => Err("Not in a voice chat.".into()),
                        }
                    } else {
                        Err("Not in a voice chat.".into())
                    }
                }
            },
            None => Err("Not in a voice chat.".into()),
        }
    }

    pub async fn music_file_autocomplete<'a>(
        ctx: Context<'a>,
        partial: &'a str,
    ) -> impl Stream<Item = String> + 'a {
        let music_dir = ctx.data().music_dir.clone();
        let completions: Vec<String> = music_dir
            .read_dir()
            .map(move |entries| {
                entries.filter_map(move |entry| match entry {
                    Ok(entry) => {
                        let interim_entry = entry
                            .path()
                            .strip_prefix(&music_dir)
                            .expect("Could not strip prefix.")
                            .to_str()
                            .expect("Couldn't convert non-utf8 path to string.")
                            .to_owned();
                        if interim_entry.starts_with(&partial) {
                            Some(interim_entry)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                })
            })
            .expect("Could not read directory.")
            .take(12) // this restriction is required to have the options load in time.
            .collect();

        futures::stream::iter(completions)
    }

    /// Make the bot leave the current voice channel.
    ///
    /// Stops all playback and clears the queue
    #[poise::command(slash_command)]
    pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
        let not_in_vc_error = create_error_embed(
            "Not in a voice chat.",
            "Troller is not in a voice chat in this guild.",
        );

        ctx.defer().await?;

        let manager = songbird::get(&ctx.serenity_context())
            .await
            .expect("Could not find serenity maanger");

        let embed = match manager.remove(ctx.guild_id().unwrap()).await {
            Ok(_) => create_quick_success_embed(
                "left voice channel",
                "Successfully left the voice channel.",
            ),
            Err(_) => not_in_vc_error,
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

    /// Add an audio file to the playback queue and optionally join the voice channel.
    ///
    /// **Options:**
    /// - `filename` (required, autocomplete) - Select an audio file from the music directory
    /// - `play_now` (optional) - Whether to start playing the track immediately (default: false)
    ///
    /// **Example Usage:**
    /// - `/music enqueue filename:background_music.mp3` - Add a track to the queue
    /// - `/music enqueue filename:boss_theme.mp3 play_now:true` - Add and immediately play a track
    ///
    /// **Notes:**
    /// - The bot will automatically join your current voice channel if not already connected
    /// - Files are autocompleted from the configured music directory
    #[poise::command(slash_command)]
    pub async fn enqueue(
        ctx: Context<'_>,
        #[description = "pick file"]
        #[autocomplete = "music_file_autocomplete"]
        filename: Option<String>,
        #[description = "youtube url"] youtube_url: Option<String>,
        #[description = "play now?"] play_now: Option<bool>,
    ) -> Result<(), Error> {
        let path = ctx.data().music_dir.clone();

        let not_in_vc_error = create_error_embed(
            "not in a voice chat",
            "Troller is not in a voice chat in this guild.",
        );

        ctx.defer_ephemeral().await?;

        let embed = perform_call_action(
            ctx,
            true,
            async move |(_, mut call, _), (path, filename, youtube_url)| {
                call.add_global_event(Event::Track(TrackEvent::Error), TrackErrorNotifier);
                let mut path_clone = path.clone();
                let mut track_name: String = String::new();
                let input: Option<songbird::input::Input> = match filename {
                    Some(filename) => {
                        path_clone.push(&filename);
                        track_name.push_str(&filename);
                        Some(songbird::input::File::new(path_clone.clone()).into())
                    }
                    None => match youtube_url {
                        Some(url) => {
                            track_name.push_str(&url);
                            let http_client = reqwest::Client::new();
                            Some(songbird::input::YoutubeDl::new(http_client, url).into())
                        }
                        None => None,
                    },
                };

                match input {
                    Some(input) => {
                        let handle = call.enqueue_input(input.into()).await;
                        if !play_now.unwrap_or(false) {
                            handle.pause()?;
                        }

                        Ok(create_quick_success_embed(
                            "action successful",
                            &format!("Enqueued Track {}", track_name),
                        ))
                    }
                    None => Ok(create_error_embed(
                        "action unsuccessful",
                        "No input specified.",
                    )),
                }
            },
            (path, filename, youtube_url),
        )
        .await;

        ctx.send(poise::CreateReply {
            embeds: vec![embed.unwrap_or(not_in_vc_error)],
            ephemeral: Some(true),
            reply: true,
            ..Default::default()
        })
        .await?;

        Ok(())
    }

    /// Control playback of the current track in the queue.
    ///
    /// **Options:**
    /// - `action` (required, choice) - The control action to perform:
    /// - `pause` - Pause the currently playing track
    /// - `play` - Resume a paused track
    /// - `stop` - Stop the current track completely
    /// - `skip` - Skip to the next track in the queue
    /// - `loop_toggle` - Toggle looping for the current track
    ///   
    /// **Example Usage:**
    /// - `/music control action:pause` - Pause playback
    /// - `/music control action:skip` - Skip to the next track
    /// - `/music control action:loop_toggle` - Enable/disable looping
    ///     
    /// **Notes:**
    /// - Requires an active track in the queue
    /// - Loop toggle switches between looping and non-looping states
    #[poise::command(slash_command)]
    pub async fn control(
        ctx: Context<'_>,
        #[choices("pause", "play", "stop", "skip", "loop_toggle")] action: &'static str,
    ) -> Result<(), Error> {
        let not_in_vc_error = create_error_embed(
            "Not in a voice chat.",
            "Troller is not in a voice chat in this guild.",
        );

        ctx.defer_ephemeral().await?;

        let action_string = action.to_owned();

        let embed = perform_call_action(
            ctx,
            true,
            async move |(_, call, _), _| {
                let header = "action successful";
                Ok(match call.queue().current() {
                    Some(track_handle) => match action_string.as_str() {
                        "pause" => {
                            track_handle.pause()?;
                            create_quick_success_embed(header, "Paused track.")
                        }
                        "play" => {
                            track_handle.play()?;
                            create_quick_success_embed(header, "Playing track.")
                        }
                        "stop" => {
                            track_handle.stop()?;
                            create_quick_success_embed(header, "Stopped track.")
                        }
                        "skip" => {
                            call.queue().skip()?;
                            create_quick_success_embed(header, "Skipped track.")
                        }
                        "loop_toggle" => {
                            let info = track_handle.get_info().await.unwrap();
                            match info.loops {
                                LoopState::Finite(0) => {
                                    track_handle.enable_loop()?;
                                    create_quick_success_embed(header, "Looping track.")
                                }
                                _ => {
                                    track_handle.disable_loop()?;
                                    create_quick_success_embed(header, "Stopped looping track.")
                                }
                            }
                        }
                        _ => create_error_embed("invalid action", "Cannot perform that action."),
                    },
                    None => create_error_embed("invalid action", "No items in the queue."),
                })
            },
            (),
        )
        .await;

        ctx.send(poise::CreateReply {
            embeds: vec![embed.unwrap_or(not_in_vc_error)],
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
        subcommands("leave", "control", "enqueue"),
        guild_only
    )]
    pub async fn music(_: Context<'_>) -> Result<(), Error> {
        Ok(())
    }
}
