//! Chat Memory and History. The history work is in the `[[native.mixin]]` rows of
//! `horizon-mod.toml` and their helpers in `src/mixins/`. This file adds the settings page
//! (Options > Mods) and the message sound.
use mod_api::{
    ClientChatReceivedEvent, ClientHost, ConfigPath, Flow, Hooks, Registrar, RegistrationContext,
    SettingChangedEvent, SettingSpec, SoundPlay,
};
use std::sync::Mutex;

const ID: &str = "more_chat_history";
/// The settings the mixins read: the mixin helpers run inside the game's own crate and cannot
/// call the host, so the values go through this file in the mod's data folder.
const SHARED: &str = "settings.txt";

#[derive(Default)]
struct Mod {
    shared: Mutex<String>,
}

impl Mod {
    /// Writes `save_history` and `max_file_mib` for the mixins when they changed.
    fn share(&self, host: &dyn ClientHost) {
        let save = host.setting_bool("save_history").unwrap_or(true);
        let mib = host.setting_f64("max_file_mib").unwrap_or(4.0).round() as u64;
        let text = format!("save_history={save}\nmax_file_mib={mib}\n");
        let Ok(mut last) = self.shared.lock() else {
            return;
        };
        if *last != text
            && let Ok(path) = ConfigPath::new(SHARED)
            && host.data_files().write(&path, text.as_bytes()).is_ok()
        {
            *last = text;
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        let p = "more_chat_history.setting.";
        for spec in [
            SettingSpec::toggle("save_history", format!("{p}save_history"), true)
                .tooltip(format!("{p}save_history.tooltip")),
            SettingSpec::range(
                "max_file_mib",
                format!("{p}max_file_mib"),
                1.0,
                64.0,
                1.0,
                4.0,
            )
            .tooltip(format!("{p}max_file_mib.tooltip")),
            SettingSpec::toggle("sound", format!("{p}sound"), true)
                .tooltip(format!("{p}sound.tooltip")),
            SettingSpec::range("sound_volume", format!("{p}sound_volume"), 0.0, 1.0, 0.05, 0.6),
            SettingSpec::toggle("sound_own", format!("{p}sound_own"), false),
            SettingSpec::toggle("sound_system", format!("{p}sound_system"), false)
                .tooltip(format!("{p}sound_system.tooltip")),
        ] {
            let _ = r.register_setting(spec);
        }
    }

    fn setting_changed(&self, host: &mut dyn ClientHost, _: &mut SettingChangedEvent<'_>) {
        self.share(&*host);
    }

    fn client_chat_received(
        &self,
        host: &mut dyn ClientHost,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        self.share(&*host);
        if event.overlay || !host.setting_bool("sound").unwrap_or(true) {
            return Flow::Continue;
        }
        let wanted = if event.system {
            host.setting_bool("sound_system").unwrap_or(false)
        } else {
            let own = host.local_player().is_some_and(|p| p.uuid() == event.sender);
            !own || host.setting_bool("sound_own").unwrap_or(false)
        };
        if wanted {
            let volume = host.setting_f64("sound_volume").unwrap_or(0.6) as f32;
            // One event with six sounds: the engine picks one at random.
            host.play_local_sound(
                &SoundPlay::ui(&format!("{ID}:message")).with_volume(volume),
                0,
            );
        }
        Flow::Continue
    }
}
