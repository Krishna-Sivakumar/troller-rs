# Troller

A Discord bot to help me run D&D!

Currently Troller does two things:
1. Creates and manages Progress Clocks (Blades in the Dark style)
2. Rolls dice

Both of these features are available as the following commands:

## Dice Rolling

### `/roll`
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

## Progress Clock Management

### `/add_progress_clock`
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

### `/bump_progress_clock`
Advance a progress clock by filling in more segments.

**Options:**
- `name` (required, autocomplete) - The name of the clock to bump
- `count` (optional) - How many segments to fill (default: 1)

**Example Usage:**
- `/bump_progress_clock name:Escape Plan` - Advance "Escape Plan" by 1 segment
- `/bump_progress_clock name:Escape Plan count:2` - Advance "Escape Plan" by 2 segments

---

### `/display_clock`
Show an existing progress clock.

**Options:**
- `name` (required, autocomplete) - The name of the clock to display

**Example Usage:**
- `/display_clock name:Escape Plan` - Display the current state of "Escape Plan"

---

### `/remove_progress_clock`
Delete a progress clock.

**Options:**
- `name` (required, autocomplete) - The name of the clock to remove

**Example Usage:**
- `/remove_progress_clock name:Escape Plan` - Delete the "Escape Plan" clock

---

## Notes

1. Permissions are managed through the Server Integrations panel.
2. Progress clocks are namespaced by server (guild) or by user in direct messages
3. Clock names support autocomplete in commands that reference existing clocks

