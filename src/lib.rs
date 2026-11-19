//! More Chat History: the work is in the three `[[native.mixin]]` rows of `horizon-mod.toml`
//! and their helpers in `src/mixins/`. The crate has no hooks of its own.
use mod_api::Hooks;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {}
