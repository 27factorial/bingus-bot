# BingusBot Usage Documentation

## About
### What is Bingus used for?
Bingus is a bot used on personal servers for scheduling activities in Destiny 2 as well as Discord server management
(soon).

Currently, BingusBot is under heavy development, and the current version is likely to look nothing like future versions
of the bot. Do not rely on the exact behavior of the application, as it is likely to change with nearly every revision.

### How do I host Bingus on my own servers?
Since I don't intend to release pre-compiled binaries, you will need to compile the application from source. Bingus is 
developed and tested using the `nightly-x86_64-pc-windows-msvc` Rust compiler toolchain. Pulling Bingus from this 
repository and running `cargo run -- file` will compile and run the bot for your own personal use.

The first time that you run Bingus with the `file` subcommand, a JSON file named `config.json` will be generated. This
file must be filled in with the correct settings, otherwise the bot may not work correctly. Here is an example of a 
valid configuration file. Note that the comments are only to describe the format, and they are not allowed in the actual
configuration file.

#### Config format:

```json5
{
  "token": "discord-bot-token-here-dont-leak-this-pls", // The bot token that Bingus authenticates with
  "owner_ids": [815012009993175090], // A list of Discord user IDs
  "allow_dm": false, // Whether Bingus should respond to DMs (This is not implemented yet)
  "allow_bots": false, // Whether Bingus should respond to bots
  "prefix": "~", // The command prefix for Bingus
  "assets_file": "./config/assets.json", // The location of assets.json
  "embeds_file": "./config/embeds.json" // The location of embeds.json
}
```

### How do I request a feature?
If you know me personally, you know to contact me directly on Discord or by other means. Otherwise, feel free
to open an issue describing the feature, or a PR with the feature implemented. Note that PRs will be reviewed manually
and should follow the same code style as other parts of the bot.

### How should I report bugs?
To report bugs, open an issue describing the bug and giving as much detail as is relevent, steps to reproduce it, and a 
minimal example, if possible. If the bug is a major security flaw, Please contact me on Discord directly (@Factorial#0027),
or open a PR with a fix for it.

## Command Usage
The following sections will outline what commands are available, each one's function, and the syntax for each. Note that
each command requires a prefix before it, and some require additional text AFTER the prefix to run them. For extra
parameters, angle brackets will be used (e.g. `activity join <id>`). The angle brackets should *not* be used when
running the command, and are only there to show extra parameters.

The general syntax for commands is `<prefix>(additional) <command> <param1> <param2> ... <paramN>`. For a few concrete
examples and a structure example:
```
~activity create

~activity join 0

~admin activity delete 0

~owner add_admins 141728791461494786 815012009993175090

prefix   additional   command   param1   param2              param3
|        |            |         |        |                   |
V        V            V         V        V                   V
~        admin        activity  add      815012009993175090  0
```

Spaces are not required between the prefix and commands, but should be used in the case where additional text is
required to run the command.

### Activity Commands
##### Additonal prefix: None

- `activity create` - Sets up an activity roster for each different activity type in Destiny 2. Follow the on-screen
  instructions to complete the roster setup. **NOTE:** Wait for Bingus to add all of its reactions before selecting an
  option.
  
- `activity join <id>` - Joins the main fireteam for the activity with the specified ID. Users in the main fireteam will 
  be automatically pinged in the channel that the activity was created in when the activity starts.
  
- `activity alt <id>` - Joins the alternate fireteam for the activity with the specified ID. Users in the alternate
  fireteam will *not* be pinged when the activity starts. The alternative fireteam list should be used as a reference for 
  possible replacement members, should one member in the main fireteam decide to leave or drop out.
  
- `activity leave <id>` - Leaves an activity's fireteam that you previously joined. This should be fairly self-
  explanatory. Sends an error message if you did not previously join either of the activity's fireteams.
  
- `activity delete <id>` - Deletes an activity with the specified ID. Members who had previously joined the activity
  will not be pinged when the activity was intended to start. Only the person who created the activity can use this
  command.

- `activity edit <id>` - Edits an activity with the specified ID. This will allow you to change the activity's
  description and start time. Only the person who created the activity can use this command.
  
- `activity list <page>` - Lists all currently scheduled activities in the guild this command is run in. Each page
  lists three activities, and activities are ordered by ID.
  
### Administrator Commands
##### Additional prefix: `admin`

- `activity add <user id> <act. id>` - Adds the specified user to the main fireteam of the activity with the 
  specified ID.
  
- `activity alt <user id> <act. id>` - Adds the specified user to the alternate fireteam of the activity with the 
  specified ID.
  
- `activity remove <user id> <act. id>` - Removes the specified user from either the main or alternate fireteam
  of the activity with the specified ID.
  
- `activity delete <act. id>` - Deletes the activity with the specified ID. Members who had previously joined the
  activity will not be pinged when the activity was intended to start. Any admin users can run this command on any
  activity.
  
- `nick <name>` - **Not Implemented** - Sets the bot's nickname in the server to the specified name.

### Owner Commands
##### Additional prefix: `owner`

- `reload_json` - Reloads all JSON configuration files from disk (excluding `config.json`).

- `add_admins <id1> <id2> ... <idN>` - Adds the specified user IDs as admins, allowing those users to run Administrator 
  commands.

  
