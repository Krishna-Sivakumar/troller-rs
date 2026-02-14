# Troller

A Discord bot to help me run D&D!

Currently Troller does three things:
1. Creates and manages Progress Clocks (Blades in the Dark style)
2. Rolls dice
3. Plays music in a local directory

## Installation (Linux)

1. Install Rust & Cargo from here: https://rust-lang.org/tools/install/
2. Follow these instructions:
```shell
cd troller-rs
cargo build -r
DISCORD_TOKEN='YOUR_TOKEN_HERE' ./target/release/troller-rs
```

## Command Reference

### Dice Rolling

#### `/roll`
Roll dice using standard dice notation.

**Options:**
- `dice_string` (required) - The dice string to roll (e.g., "2d6", "1d20+5")
- `keep_private` (optional) - Whether to keep the roll result visible only to you (default: false)

**Example Usage:**
- `/roll 1d20 + 5`: Rolls a d20 and adds 5 to the result.
- `/roll 1d20 + 6, 1d8 + 4`: Rolls a d20 and a d8 at the same time.
- `/roll hit: 1d20 + 5`: Rolls dice and adds the name "hit" to the particular roll.
- `/roll hit: 1d20 + 6, damage: 1d8 + 4`: Rolls the dice and attaches names to the rolls.
- `/roll 2d20h1`: Rolls 2 d20s and takes the highest one.
- `/roll 4d6h3`: Rolls 4 d6s and takes the highest three.
- `/roll 2d20l1`: Rolls 2 d20s and takes the lowest one.
- `/roll 5 * 3d6`: Multiplies 5 to the result of the 3d6 roll. It *does not* roll 15 sets of dice.

---

### Progress Clock Management

#### `/add_progress_clock`
Create a new progress clock to track goals or countdowns.

**Options:**
- `segments` (required) - The total number of segments in the clock
- `name` (required) - The name of the clock
- `segments_filled` (optional) - How many segments are already filled (default: 0)
- `ephemeral` (optional) - Whether to automatically delete the clock after one day (default: false)
- `color` (optional) - The color of the clock (HTML color name or hex code)
- `display_now` (optional) - Whether to immediately display the clock after creation (default: false)

**Example Usage:**
- `/add_progress_clock segments:6 name:Escape Plan` - Create a 6-segment clock named "Escape Plan"
- `/add_progress_clock segments:8 name:Ritual segments_filled:3 color:#FF5733 display_now:true` - Create an 8-segment clock with 3 segments already filled, custom color, and display immediately

---

#### `/bump_progress_clock`
Advance a progress clock by filling in more segments.

**Options:**
- `name` (required, autocomplete) - The name of the clock to bump
- `count` (optional) - How many segments to fill (default: 1)

**Example Usage:**
- `/bump_progress_clock name:Escape Plan` - Advance "Escape Plan" by 1 segment
- `/bump_progress_clock name:Escape Plan count:2` - Advance "Escape Plan" by 2 segments

---

#### `/display_clock`
Show an existing progress clock.

**Options:**
- `name` (required, autocomplete) - The name of the clock to display

**Example Usage:**
- `/display_clock name:Escape Plan` - Display the current state of "Escape Plan"

---

#### `/remove_progress_clock`
Delete a progress clock.

**Options:**
- `name` (required, autocomplete) - The name of the clock to remove

**Example Usage:**
- `/remove_progress_clock name:Escape Plan` - Delete the "Escape Plan" clock

---


### `/help`
Display help text and usage examples for any Troller command.

**Options:**
- `command_name` (required, autocomplete) - The name of the command to get help for

**Example Usage:**
- `/help command_name:roll` - Display detailed help for the `/roll` command
- `/help command_name:add_progress_clock` - Display help for creating progress clocks

---

## Music Playback

### `/music enqueue`
Add an audio file to the playback queue and optionally join the voice channel.

**Options:**
- `filename` (required, autocomplete) - Select an audio file from the music directory
- `play_now` (optional) - Whether to start playing the track immediately (default: false)

**Example Usage:**
- `/music enqueue filename:background_music.mp3` - Add a track to the queue
- `/music enqueue filename:boss_theme.mp3 play_now:true` - Add and immediately play a track

**Notes:**
- The bot will automatically join your current voice channel if not already connected
- Files are autocompleted from the configured music directory

---

### `/music control`
Control playback of the current track in the queue.

**Options:**
- `action` (required, choice) - The control action to perform:
  - `pause` - Pause the currently playing track
  - `play` - Resume a paused track
  - `stop` - Stop the current track completely
  - `skip` - Skip to the next track in the queue
  - `loop_toggle` - Toggle looping for the current track

**Example Usage:**
- `/music control action:pause` - Pause playback
- `/music control action:skip` - Skip to the next track
- `/music control action:loop_toggle` - Enable/disable looping

**Notes:**
- Requires an active track in the queue
- Loop toggle switches between looping and non-looping states

---

### `/music leave`
Make the bot leave the current voice channel.

**Example Usage:**
- `/music leave` - Disconnect from voice channel

**Notes:**
- Stops all playback and clears the queue
- Returns an error if the bot is not in a voice channel

### Overall Notes

1. Permissions are managed through the Server Integrations panel.
2. Progress clocks are namespaced by server (guild) or by user in direct messages
3. Clock names support autocomplete in commands that reference existing clocks

