# More Chat History for Horizon

A native Horizon mod (mixin bundle) that raises the chat history from vanilla's 100 messages to 16384,
like [More Chat History](https://modrinth.com/mod/morechathistory) does on Fabric (CC0-1.0, JackFredMods).
No code of that mod is used; this one is written against Horizon's source.

It changes three limits in `horizon_client_render::chat`: the remembered messages (`Chat::push_message`),
the wrapped lines (`Chat::add_lines`) and the sent-message history of the up arrow (`Chat::add_recent`).
The saved command history (`command_history.txt`) keeps vanilla's own limit.

It also keeps the chat between sessions, a feature of [Chat Patches](https://modrinth.com/mod/chatpatches)
(LGPL-3.0, none of its code is used): every message is appended to `chat_history.log` in the game folder, and the
next session shows the last 16384 of them under a grey `--- earlier sessions ---` divider. Only the text is kept,
not colours, hover or click events. Horizon already has timestamps, repeat counting and chat search as extras, so
those are not duplicated.

Build: `cargo horizon-mod build`, then `horizon-launcher native-build . --instance <instance> --trust-native-code`.
The mod pins one Horizon build (`[native] horizon` in `horizon-mod.toml`), which must contain `#[replaceable]` on the
three functions (horizon-rust/horizon#18356).

## How it works

The limits are literals in functions whose bodies contain macros or a closure, which horizon-mixins refuses to
rewrite. Each of the three functions gets a `HEAD` mixin with `cancellable = true`: its helper
(`src/mixins/limit_*.rs`) is a copy of the body with the new limit, and the original body is skipped. A Horizon change
to those functions needs the helpers updated.

Restoring waits for the first message that arrives 10 seconds into the session (or any message while the chat is
open): until then the restored lines would be drawn like new ones for their 200 ticks. Single player runs a second,
invisible `Chat` on the "Server thread" that gets the same messages; only the window's own (the `main` thread) reads and
writes the file, or each message would be saved twice.

## Licence

MIT. The helpers copy the bodies of three functions of [Horizon](https://github.com/horizon-rust/horizon) (MIT).
Idea and value from More Chat History by JackFredMods (CC0-1.0); none of its code is used.

## Status

Pinned to Nightly 2026-11-20 (`0a875da`). Needs a launcher with horizon-rust/horizon#18789 (launcher 0.2.89 and later);
older launchers fail with `unresolved import crate::command`. Verified with launcher 0.2.99 and the released package: 300 chat
lines sent through `horizon-ctl chat` all stay in the history; the same Nightly without the mod keeps 100.
The session history (0.2.0) is checked over nine consecutive launches of one world: the ninth shows all earlier messages
in order under the divider, each once.
