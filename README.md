# More Chat History for Horizon

A native Horizon mod (mixin bundle) that raises the chat history from vanilla's 100 messages to 16384,
like [More Chat History](https://modrinth.com/mod/morechathistory) does on Fabric (CC0-1.0, JackFredMods).
No code of that mod is used; this one is written against Horizon's source.

It changes three limits in `horizon_client_render::chat`: the remembered messages (`Chat::add_message`),
the wrapped lines (`Chat::add_lines`) and the sent-message history of the up arrow (`Chat::add_recent`).
The saved command history (`command_history.txt`) keeps vanilla's own limit.

Build: `horizon-launcher native-build more-chat-history --instance <instance> --trust-native-code`.
The bundle pins one Horizon commit (`[compatibility]` in `more-chat-history/horizon-mixin.toml`).
