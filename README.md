# More Chat History for Horizon

A native Horizon mod (mixin bundle) that raises the chat history from vanilla's 100 messages to 16384,
like [More Chat History](https://modrinth.com/mod/morechathistory) does on Fabric (CC0-1.0, JackFredMods).
No code of that mod is used; this one is written against Horizon's source.

It changes three limits in `horizon_client_render::chat`: the remembered messages (`Chat::push_message`),
the wrapped lines (`Chat::add_lines`) and the sent-message history of the up arrow (`Chat::add_recent`).
The saved command history (`command_history.txt`) keeps vanilla's own limit.

Build: `horizon-launcher native-build more-chat-history --instance <instance> --trust-native-code`.
The bundle pins one Horizon commit (`[compatibility]` in `more-chat-history/horizon-mixin.toml`).

## How it works

The limits are literals in functions whose bodies contain macros or closures, which horizon-mixins refuses to
rewrite. Each of the three functions gets a cancellable HEAD injection whose helper is a copy of the body with the
new limit; the original body is skipped. The copies follow the pinned commit, so a Horizon change to those
functions needs the helpers updated (a build then fails or leaves the mod out).

Checked against Horizon 0a6e409923 with horizon-mixins 5a64af9: `plan`, `apply`, `cargo check -p horizon-client-render`
and the 37 chat tests of the modded tree pass. Played: the modded game kept 248 of 250 chat lines sent through horizon-ctl (vanilla keeps 100).
