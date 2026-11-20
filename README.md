# More Chat History for Horizon

A native Horizon mod (mixin bundle) that raises the chat history from vanilla's 100 messages to 16384,
like [More Chat History](https://modrinth.com/mod/morechathistory) does on Fabric (CC0-1.0, JackFredMods).
No code of that mod is used; this one is written against Horizon's source.

It changes three limits in `horizon_client_render::chat`: the remembered messages (`Chat::push_message`),
the wrapped lines (`Chat::add_lines`) and the sent-message history of the up arrow (`Chat::add_recent`).
The saved command history (`command_history.txt`) keeps vanilla's own limit.

Build: `cargo horizon-mod build`, then `horizon-launcher native-build . --instance <instance> --trust-native-code`.
The mod pins one Horizon build (`[native] horizon` in `horizon-mod.toml`), which must contain `#[replaceable]` on the
three functions (horizon-rust/horizon#18356).

## How it works

The limits are literals in functions whose bodies contain macros or a closure, which horizon-mixins refuses to
rewrite. Each of the three functions gets a `HEAD` mixin with `cancellable = true`: its helper
(`src/mixins/limit_*.rs`) is a copy of the body with the new limit, and the original body is skipped. A Horizon change
to those functions needs the helpers updated.

## Licence

MIT. The helpers copy the bodies of three functions of [Horizon](https://github.com/horizon-rust/horizon) (MIT).
Idea and value from More Chat History by JackFredMods (CC0-1.0); none of its code is used.

## Status

Pinned to Nightly 2026-11-20 (`0a875da`). The chat markers (horizon-rust/horizon#18356) are in since `39f1a51b3c`; building
needs a launcher with horizon-rust/horizon#18789 (merged as `04b92a5b4b`, not in a launcher release yet). With a launcher built
from `main`, the mod builds and works: 300 chat lines sent through `horizon-ctl chat` all stay in the history; the same Nightly
without the mod keeps 100. Launchers 0.2.78 and 0.2.86 fail with `unresolved import crate::command`.
