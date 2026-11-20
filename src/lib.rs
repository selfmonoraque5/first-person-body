//! First Person Body on Horizon's mod API. The game patches in `src/mixins/`
//! (see horizon-mod.toml) draw the own model in first person, move it back
//! from the camera and leave out what would sit at the camera; the hooks here
//! keep the settings and the toggle key, hide the head (and the arms with the
//! vanilla hand) and leave out the vanilla hand with the model's arms.
use mod_api::{
    self, ClientHost, ClientTickPostEvent, Flow, Hooks, HumanoidPart, KeyRegistration, Registrar,
    RegistrationContext, RenderArmEvent, RenderHandEvent, RenderPlayerPreEvent,
    SettingChangedEvent, SettingSpec, Text,
};

/// The mod's logic, also read by the game patches.
pub mod body;

use body::{OPTS, Settings};

/// The settings as the page keeps them.
fn read(host: &dyn ClientHost) -> Settings {
    Settings::from_choices(|key| host.setting_choice(key))
}

#[derive(Default)]
struct FirstPersonBody;

#[mod_api::horizon_mod]
impl Hooks for FirstPersonBody {
    /// The settings page (one choice per option) and the toggle key.
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        let defaults = Settings::default();
        for (i, opt) in OPTS.iter().enumerate() {
            let options = opt.choices.iter().map(|(v, _)| body::choice_key(opt.key, *v));
            let _ = r.register_setting(SettingSpec::choice(
                opt.key,
                body::label_key(opt.key),
                options,
                defaults.choice_index(i),
            ));
        }
        r.register_key_mapping(
            KeyRegistration::new(body::TOGGLE_KEY, body::TOGGLE_KEY_DEFAULT)
                .label(body::TOGGLE_KEY_LABEL_KEY),
        );
    }

    /// The player changed an option on the page.
    fn setting_changed(&self, host: &mut dyn ClientHost, _: &mut SettingChangedEvent<'_>) {
        body::set_settings(read(&*host));
    }

    /// Each tick: the settings for the patches, and the toggle key.
    fn client_tick_post(&self, host: &mut dyn ClientHost, _: &mut ClientTickPostEvent) {
        let mut settings = read(&*host);
        while host.consume_click(body::ID, body::TOGGLE_KEY) {
            let (key, index) = settings.toggle();
            host.set_setting(body::ID, key, &index);
            let _ = host.chat_line(Text::literal(body::toggled_line(settings.enabled())));
        }
        body::set_settings(settings);
    }

    /// The own model in first person: no head (the camera is inside it), and no
    /// arms when the vanilla hand draws instead.
    fn render_player_pre(&self, _: &dyn ClientHost, event: &mut RenderPlayerPreEvent<'_>) -> Flow {
        if !event.who.local || !body::active() {
            return Flow::Continue;
        }
        event.hidden.hide(HumanoidPart::Head);
        if body::arms_hidden(&body::current()) {
            event.hidden.hide(HumanoidPart::RightArm);
            event.hidden.hide(HumanoidPart::LeftArm);
        }
        Flow::Continue
    }

    /// With the model's arms, the vanilla first-person hand isn't drawn: the
    /// model's arms and held items show instead.
    fn render_hand(&self, _: &dyn ClientHost, _: &mut RenderHandEvent<'_, '_>) -> Flow {
        model_arms_flow()
    }

    fn render_arm(&self, _: &dyn ClientHost, _: &mut RenderArmEvent<'_>) -> Flow {
        model_arms_flow()
    }
}

/// Cancels the vanilla hand while the body shows with the model's arms.
fn model_arms_flow() -> Flow {
    let settings = body::current();
    if body::shown() && settings.enabled() && !body::arms_hidden(&settings) {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}
