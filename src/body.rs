// First Person Body for Horizon: your own player model in first person, so
// you see your body, legs and feet when you look down. The head and what is
// worn on it are hidden (the camera is inside it), the body is moved back a
// little so the camera doesn't look into the chest, and the arms are either
// the vanilla first-person hand or the model's own. Inspired by tr7zw's
// First-person Model, written from scratch on Horizon (none of its code).
//
// This module is the mod's logic and its tests: the settings, where the body
// stands and which parts hide. The hooks are in lib.rs, the game patches in
// mixins/ (helpers the game calls, which reach this crate as
// `mod_first_person_body`).
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// The mod's id (its settings folder) and name; the version is in horizon-mod.toml.
pub const ID: &str = "first_person_body";
pub const NAME: &str = "First Person Body";

/// The key that turns the body on and off (unbound until the player picks one).
pub const TOGGLE_KEY: &str = "toggle";
pub const TOGGLE_KEY_DEFAULT: &str = "key.keyboard.unknown";
pub const TOGGLE_KEY_LABEL_KEY: &str = "first_person_body.key.toggle";
#[cfg(test)]
pub const TOGGLE_KEY_LABEL: &str = "Toggle First Person Body";

/// One option of the settings page: its file, its English label, the values
/// the player cycles through (with their names) and the default.
pub struct Opt {
    pub key: &'static str,
    pub label: &'static str,
    pub choices: &'static [(u8, &'static str)],
    pub default: u8,
}

const ON_OFF: &[(u8, &str)] = &[(0, "OFF"), (1, "ON")];
const ARMS: &[(u8, &str)] = &[(0, "Vanilla hand"), (1, "Model arms")];
/// How far back the body stands, in hundredths of a block.
const OFFSET: &[(u8, &str)] = &[(0, "None"), (15, "Small"), (25, "Normal"), (35, "Large")];
const SHOWN_HIDDEN: &[(u8, &str)] = &[(0, "Hidden"), (1, "Shown")];

pub const OPTS: [Opt; 6] = [
    Opt { key: "enabled", label: "First Person Body", choices: ON_OFF, default: 1 },
    Opt { key: "arms", label: "Arms", choices: ARMS, default: 0 },
    Opt { key: "offset", label: "Body offset", choices: OFFSET, default: 25 },
    Opt { key: "riding", label: "While riding", choices: ON_OFF, default: 1 },
    Opt { key: "look_down", label: "Lean back looking down", choices: ON_OFF, default: 1 },
    Opt { key: "helmet", label: "Helmet in first person", choices: SHOWN_HIDDEN, default: 0 },
];

/// Indexes into `OPTS`.
pub mod o {
    pub const ENABLED: usize = 0;
    pub const ARMS: usize = 1;
    pub const OFFSET: usize = 2;
    pub const RIDING: usize = 3;
    pub const LOOK_DOWN: usize = 4;
    pub const HELMET: usize = 5;
}

/// The settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    values: [u8; OPTS.len()],
}

impl Default for Settings {
    fn default() -> Self {
        Self { values: OPTS.map(|opt| opt.default) }
    }
}

impl Settings {
    /// From the settings page, where each option is a choice: `index` gives
    /// the option the player picked for a key (its position in `choices`).
    pub fn from_choices(index: impl Fn(&str) -> Option<u32>) -> Self {
        let mut s = Self::default();
        for (i, opt) in OPTS.iter().enumerate() {
            let picked = index(opt.key).and_then(|at| opt.choices.get(usize::try_from(at).ok()?));
            if let Some((value, _)) = picked {
                s.values[i] = *value;
            }
        }
        s
    }

    /// Option `i`'s value as the page keeps it: its position in `choices`.
    pub fn choice_index(&self, i: usize) -> usize {
        OPTS[i].choices.iter().position(|c| c.0 == self.values[i]).unwrap_or(0)
    }

    pub fn get(&self, i: usize) -> u8 {
        self.values[i]
    }

    pub fn enabled(&self) -> bool {
        self.values[o::ENABLED] == 1
    }

    /// The toggle key: what to keep (the key and its choice index).
    pub fn toggle(&mut self) -> (&'static str, String) {
        self.values[o::ENABLED] = u8::from(!self.enabled());
        (OPTS[o::ENABLED].key, self.choice_index(o::ENABLED).to_string())
    }
}

pub fn toggled_line(on: bool) -> String {
    format!("{NAME} {}", if on { "ON" } else { "OFF" })
}

/// The translation key of option `key`'s label, and of its choice `value`.
pub fn label_key(key: &str) -> String {
    format!("{ID}.option.{key}")
}

pub fn choice_key(key: &str, value: u8) -> String {
    format!("{ID}.option.{key}.{value}")
}

/// `assets/first_person_body/lang/en_us.json`: every label the page and the
/// key mapping show, in English.
#[cfg(test)]
pub fn english() -> Vec<(String, String)> {
    let mut lines = Vec::new();
    for opt in &OPTS {
        lines.push((label_key(opt.key), opt.label.to_owned()));
        for (value, name) in opt.choices {
            lines.push((choice_key(opt.key, *value), (*name).to_owned()));
        }
    }
    lines.push((TOGGLE_KEY_LABEL_KEY.to_owned(), TOGGLE_KEY_LABEL.to_owned()));
    lines
}

/// Where the body stands relative to the player, in blocks: moved back along
/// the way the player faces (yaw in degrees, 0 south), more when looking
/// down (pitch in degrees, positive down) and when crouching.
pub fn body_offset(settings: &Settings, yaw: f32, pitch: f32, crouching: bool) -> [f32; 3] {
    let mut back = f32::from(settings.get(o::OFFSET)) / 100.0;
    if back == 0.0 {
        return [0.0; 3];
    }
    if settings.get(o::LOOK_DOWN) == 1 {
        back += (pitch.max(0.0) / 90.0) * 0.15;
    }
    if crouching {
        back += 0.05;
    }
    // Facing (-sin yaw, 0, cos yaw); back is the opposite.
    let (s, c) = yaw.to_radians().sin_cos();
    [s * back, 0.0, -c * back]
}

/// Whether the body shows in first person: the player stands or crouches
/// (not swimming, crawling, gliding or spinning), isn't a spectator and,
/// unless the settings say otherwise, isn't riding.
pub fn body_shows(settings: &Settings, upright: bool, gliding: bool, spectator: bool, riding: bool) -> bool {
    settings.enabled()
        && upright
        && !gliding
        && !spectator
        && (settings.get(o::RIDING) == 1 || !riding)
}

// The settings the game's patches read: kept as plain numbers so any thread may read them.
static LOADED: AtomicBool = AtomicBool::new(false);
static VALUES: [AtomicU8; OPTS.len()] = [const { AtomicU8::new(0) }; OPTS.len()];
/// Whether the body showed in first person when the game last asked.
static SHOWN: AtomicBool = AtomicBool::new(false);

pub fn set_settings(settings: Settings) {
    for (slot, v) in VALUES.iter().zip(settings.values) {
        slot.store(v, Ordering::Relaxed);
    }
    LOADED.store(true, Ordering::Release);
}

/// The kept settings (the defaults before they are read).
pub fn current() -> Settings {
    if !LOADED.load(Ordering::Acquire) {
        return Settings::default();
    }
    let mut s = Settings::default();
    for (v, slot) in s.values.iter_mut().zip(&VALUES) {
        *v = slot.load(Ordering::Relaxed);
    }
    s
}

/// The game asked whether the own body is in view, in first person: `shown`.
pub fn set_shown(shown: bool) {
    SHOWN.store(shown, Ordering::Relaxed);
}

/// Whether the body shows in first person (the hand hooks read it).
pub fn shown() -> bool {
    SHOWN.load(Ordering::Relaxed)
}

thread_local! {
    /// Set while the game draws the player's own model in first person: where
    /// the body stands, and how many vertices the frame held before the model.
    static ACTIVE: Cell<Option<([f32; 3], usize)>> = const { Cell::new(None) };
}

/// The own first-person model's drawing starts: `offset` is where the body
/// stands (from `body_offset`), `start` the vertex count before the model.
pub fn set_active(offset: [f32; 3], start: usize) {
    ACTIVE.with(|a| a.set(Some((offset, start))));
}

/// Whether the own first-person model is being drawn.
pub fn active() -> bool {
    ACTIVE.with(Cell::get).is_some()
}

/// The own model's drawing ends: the offset and start it was drawn with, if
/// it was the first-person model. Afterwards nothing is active.
pub fn finish() -> Option<([f32; 3], usize)> {
    ACTIVE.with(|a| a.take())
}

/// A vertex of the own model, which the game places relative to the camera,
/// moved by the body offset: the body stands behind the view, as if drawn
/// from a camera moved forward by the offset.
pub fn moved(pos: [f32; 3], offset: [f32; 3]) -> [f32; 3] {
    [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]]
}

/// Whether the model's arms hide (the vanilla hand draws instead).
pub fn arms_hidden(settings: &Settings) -> bool {
    settings.get(o::ARMS) == 0
}

/// Whether the worn helmet is left out of the own first-person model: it sits around the
/// camera, so the view's sway and bob (Living Camera's too) show its edges.
pub fn helmet_hidden(settings: &Settings) -> bool {
    settings.get(o::HELMET) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn defaults_are_choices() {
        for opt in &OPTS {
            assert!(opt.choices.iter().any(|c| c.0 == opt.default), "{}", opt.key);
        }
    }

    #[test]
    fn choices_read_as_their_values_and_toggle_keeps_an_index() {
        assert_eq!(Settings::from_choices(|_| None), Settings::default());
        let mut s = Settings::from_choices(|key| match key {
            "arms" => Some(1),
            "offset" => Some(3),
            "riding" => Some(9),
            _ => None,
        });
        assert_eq!((s.get(o::ARMS), s.get(o::OFFSET)), (1, 35));
        assert_eq!(s.get(o::RIDING), 1, "not a choice: the default");
        assert_eq!(s.choice_index(o::OFFSET), 3);
        assert_eq!(s.toggle(), ("enabled", "0".to_owned()));
        assert!(!s.enabled());
        assert_eq!(toggled_line(false), "First Person Body OFF");
    }

    #[test]
    fn body_moves_back_from_the_view() {
        let s = Settings::default();
        // Facing south (+Z): back is -Z.
        let [x, y, z] = body_offset(&s, 0.0, 0.0, false);
        assert!(close(x, 0.0) && close(y, 0.0) && close(z, -0.25));
        // Facing west (-X): back is +X.
        let [x, _, z] = body_offset(&s, 90.0, 0.0, false);
        assert!(close(x, 0.25) && close(z, 0.0), "{x} {z}");
        // Looking straight down and crouching: further back.
        let [_, _, z] = body_offset(&s, 0.0, 90.0, true);
        assert!(close(z, -0.45));
        let mut none = s;
        none.values[o::OFFSET] = 0;
        assert_eq!(body_offset(&none, 30.0, 60.0, true), [0.0; 3]);
    }

    #[test]
    fn the_body_shows_standing_and_steps_aside_otherwise() {
        let mut s = Settings::default();
        assert!(body_shows(&s, true, false, false, false));
        assert!(!body_shows(&s, false, false, false, false), "swimming or crawling");
        assert!(!body_shows(&s, true, true, false, false), "gliding");
        assert!(!body_shows(&s, true, false, true, false), "spectator");
        assert!(body_shows(&s, true, false, false, true), "riding, by default");
        s.values[o::RIDING] = 0;
        assert!(!body_shows(&s, true, false, false, true));
        s.toggle();
        assert!(!body_shows(&s, true, false, false, false), "off");
    }

    #[test]
    fn arms_follow_their_option() {
        let mut s = Settings::default();
        assert!(arms_hidden(&s));
        s.values[o::ARMS] = 1;
        assert!(!arms_hidden(&s));
    }

    #[test]
    fn active_is_per_thread_and_finish_ends_it() {
        assert!(!active());
        assert_eq!(finish(), None);
        set_active([0.0, 0.0, -0.25], 7);
        assert!(active());
        std::thread::spawn(|| assert!(!active())).join().unwrap();
        assert_eq!(finish(), Some(([0.0, 0.0, -0.25], 7)));
        assert!(!active());
        assert_eq!(finish(), None);
    }

    #[test]
    fn vertices_move_by_the_offset() {
        // Positions are camera-relative: a body 0.25 behind (-Z) moves its vertices 0.25 back.
        assert_eq!(moved([1.0, 2.0, 3.0], [0.0, 0.0, -0.25]), [1.0, 2.0, 2.75]);
        assert_eq!(moved([1.0, 2.0, 3.0], [0.0; 3]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn settings_are_kept() {
        let mut s = Settings::default();
        s.values[o::ARMS] = 1;
        set_settings(s);
        assert_eq!(current().get(o::ARMS), 1);
        set_shown(true);
        assert!(shown());
    }

    #[test]
    fn every_label_is_in_english() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/first_person_body/lang/en_us.json");
        let file = std::fs::read_to_string(path).unwrap_or_default();
        for (key, text) in english() {
            let line = format!("{key:?}: {text:?}");
            assert!(file.contains(&line), "{line} missing from en_us.json");
        }
    }
}
