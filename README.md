# Chat Memory and History for Horizon

(Formerly "More Chat History"; the id `more_chat_history` and the repository name stay.)

A native Horizon mod (mixin bundle) that raises the chat history from vanilla's 100 messages to 16384,
like [More Chat History](https://modrinth.com/mod/morechathistory) does on Fabric (CC0-1.0, JackFredMods).
No code of that mod is used; this one is written against Horizon's source.

It changes three limits in `horizon_client_render::chat`: the remembered messages (`Chat::push_message`),
the wrapped lines (`Chat::add_lines`) and the sent-message history of the up arrow (`Chat::add_recent`).
The saved command history (`command_history.txt`) keeps 1000 commands instead of vanilla's 50.

It also keeps the chat between sessions, a feature of [Chat Patches](https://modrinth.com/mod/chatpatches)
(LGPL-3.0, none of its code is used): every message is appended to `chat_history.log` in the game folder, and the
next session shows the last 16384 of them under a grey `--- earlier sessions ---` divider. Only the text is kept,
not colours, hover or click events. Horizon already has timestamps, repeat counting and chat search as extras, so
those are not duplicated.

Build: `cargo horizon-mod build`, then `horizon-launcher native-build . --instance <instance> --trust-native-code`.
The mod pins one Horizon build (`[native] horizon` in `horizon-mod.toml`), which must contain `#[replaceable]` on the
three functions (horizon-rust/horizon#18356).

## Settings page and message sound

Options > Mods > Chat Memory and History > Config:

| Setting | Default | What |
|---|---|---|
| Keep chat between sessions | on | Saves and restores the chat (see above). Off: nothing is written or read. |
| Saved chat size limit (MiB) | 4 | How much of the newest log is kept and read. |
| Sound on chat messages | on | A short keyboard click when a message from another player arrives. |
| Message sound volume | 0.6 | |
| Sound for your own messages | off | |
| Sound for server and system messages | off | Joins, deaths, command output. |

The sound is one event, `more_chat_history:message`, with six sounds in `assets/more_chat_history/sounds.json`; the game
picks one at random each time. It is played with `ClientHost::play_local_sound` from the `client_chat_received` hook.
The settings use `Registrar::register_setting`; the two history settings reach the mixins through
`horizon-mod-data/more_chat_history/settings.txt`, because a mixin helper runs inside the game's crate and cannot call
the host.

Chat animations like [Chat Animation](https://modrinth.com/mod/chatanimation) (MIT: a slide-in and fade for new lines,
and removing the indicator bar) are already in Horizon as extras (`chat_animation` and `chat_indicator` in Horizon
Settings), so the mod does not duplicate them.

## How it works

The limits are literals in functions whose bodies contain macros or a closure, which horizon-mixins refuses to
rewrite. Each of the three functions gets a `HEAD` mixin with `cancellable = true`: its helper
(`src/mixins/limit_*.rs`) is a copy of the body with the new limit, and the original body is skipped. A Horizon change
to those functions needs the helpers updated.

Privacy: `chat_history.log` holds everything the chat shows, private messages included, in plain text and in one file for
all worlds and servers (the mod API has no join hook to tell them apart). The settings page turns it off and caps its size:
the log is cut to its newest part every 256 messages once it passes twice the cap, and only its last `max_file_mib` MiB
is ever read.

Restoring waits for the first message that arrives 10 seconds into the session (or any message while the chat is
open): until then the restored lines would be drawn like new ones for their 200 ticks. Single player runs a second,
invisible `Chat` on the "Server thread" that gets the same messages; only the window's own (the `main` thread) reads and
writes the file, or each message would be saved twice.

## Sounds

The six sounds are keyboard clicks from [Pixabay](https://pixabay.com/sound-effects/), under the
[Pixabay Content License](https://pixabay.com/service/license-summary/) (free to use in a work, no attribution required;
not MIT, and not to be redistributed on their own). Converted to mono Ogg Vorbis and loudness-normalised, otherwise unchanged.

| File | Track | Author |
|---|---|---|
| `message_1.ogg` | keyboard chSpSil | soundsgreat |
| `message_2.ogg` | MECH-KEYBOARD-02 | freesound_community |
| `message_3.ogg` | Keyboard Click | CreatorsHome |
| `message_4.ogg` | Spacebar Click (Keyboard) | Lightningbulb |
| `message_5.ogg` | keyboard typing one short 1 | NCPRIME |
| `message_6.ogg` | Keyboard Spacebar Hit | freesound_community |

## Licence

MIT for the code. The helpers copy the bodies of three functions of [Horizon](https://github.com/horizon-rust/horizon) (MIT).
Idea and value from More Chat History by JackFredMods (CC0-1.0); none of its code is used.

## Status

Pinned to Nightly 2026-11-20 (`0a875da`). Needs a launcher with horizon-rust/horizon#18789 (launcher 0.2.89 and later);
older launchers fail with `unresolved import crate::command`. Verified with launcher 0.2.99 and the released package: 300 chat
lines sent through `horizon-ctl chat` all stay in the history; the same Nightly without the mod keeps 100.
The session history (0.2.0) is checked over nine consecutive launches of one world: the ninth shows all earlier messages
in order under the divider, each once.
