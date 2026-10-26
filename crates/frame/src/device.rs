//! The game's hardware, as a 1990s setup program offered it: one slot per kind of device
//! (graphics, sound, music, controllers), and in each slot the cards and APIs the original
//! supported. The player picks one device per slot before the game starts, as in the
//! Manager or SETUP.EXE that came with the game.
//!
//! The graphics slot is the manifest's `renderers`, so a 1.0 game keeps working. Every other
//! slot is in the manifest's `devices`:
//!
//! ```json
//! "devices": [{ "slot": "sound", "label": "Sound card", "options": [
//!     { "id": "directsound", "label": "DirectSound", "grade": "exact", "default": true },
//!     { "id": "none", "label": "No sound", "grade": "exact" } ] }]
//! ```
//!
//! The same card has the same id in every game ([`ids`]), and [`known`] names the GameFrame
//! crate or wrapper that stands in for it. So `glide` means 3dfx Glide drawn by
//! `gameframe-voodoo` in POD, in Hardwar and in the next game.
//!
//! The frame sends `device.select` `{"slot", "device"}` when the player picks one, and puts the
//! choices in `frame.ready` and `game.start` as `"devices": {"video": "glide", "sound": "none"}`.
//! A kernel works out the devices of a start with [`Manifest::resolve`](crate::Manifest::resolve)
//! ([`crate::setup`]): the pick, a stand-in for one not ported yet, and the gaps to report.

use crate::manifest::Renderer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One device a slot offers: the same fields as a renderer (a grade, how many calls are done,
/// notes, the default).
pub type Device = Renderer;

/// The slots the frame knows. A game may name its own.
pub mod slots {
    /// How the game draws: the manifest's `renderers`.
    pub const VIDEO: &str = "video";
    /// The sound card or API for effects and speech.
    pub const SOUND: &str = "sound";
    /// The music device, when the original chose it apart from the sound card (OPL, MIDI).
    pub const MUSIC: &str = "music";
    /// The controller: keyboard, joystick, force feedback.
    pub const INPUT: &str = "input";
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Slot {
    /// The slot's id, such as `sound` ([`slots`]).
    pub slot: String,
    /// The heading the player sees, such as "Sound card".
    pub label: String,
    pub options: Vec<Device>,
}

impl Slot {
    /// The device to select first: the one marked default, else the best graded.
    pub fn default_device(&self) -> Option<&Device> {
        self.options.iter().find(|d| d.default).or_else(|| self.options.iter().max_by_key(|d| d.grade))
    }
}

/// The device ids every game uses for the same card or API.
pub mod ids {
    // Graphics.
    /// The original's own software renderer.
    pub const SOFTWARE: &str = "software";
    /// The software renderer's MMX build.
    pub const MMX: &str = "mmx";
    /// 3dfx Voodoo through Glide 2.x.
    pub const GLIDE: &str = "glide";
    /// Direct3D through DirectDraw.
    pub const DIRECT3D: &str = "direct3d";
    /// PowerVR PCX through SGL.
    pub const POWERVR: &str = "powervr";
    /// ATI 3D Rage through ATI3DCIF.
    pub const ATI_RAGE: &str = "ati-rage";
    /// S3 ViRGE through the S3D Toolkit.
    pub const S3_VIRGE: &str = "s3-virge";
    // Sound.
    /// No sound card: the game runs silent, as the original did without one.
    pub const NONE: &str = "none";
    /// DirectSound, the Windows games' sound.
    pub const DIRECTSOUND: &str = "directsound";
    /// Guillemot Maxi Sound 64 through its own driver.
    pub const MAXI_SOUND_64: &str = "maxi-sound-64";
    /// Creative Sound Blaster, its DSP and mixer.
    pub const SOUND_BLASTER: &str = "sound-blaster";
    /// AdLib: the YM3812 (OPL2) alone.
    pub const ADLIB: &str = "adlib";
    /// Sound Blaster 16 or Pro 2 music: the YMF262 (OPL3).
    pub const OPL3: &str = "opl3";
    /// AdLib Gold: OPL3 and the YMZ263B.
    pub const ADLIB_GOLD: &str = "adlib-gold";
    /// Creative Music System and Game Blaster: two SAA1099.
    pub const GAME_BLASTER: &str = "game-blaster";
    /// The PCjr and Tandy 1000's SN76496.
    pub const TANDY: &str = "tandy";
    /// The PC speaker.
    pub const PC_SPEAKER: &str = "pc-speaker";
    /// General MIDI through an MPU-401.
    pub const GENERAL_MIDI: &str = "general-midi";
}

/// What GameFrame knows of a device id: its usual name, the slot it sits in, and the GameFrame
/// crate or wrapper that models it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Known {
    pub id: &'static str,
    pub slot: &'static str,
    pub label: &'static str,
    /// The crate (`crates/`) or wrapper (`wrappers/`) a port draws or plays it with. Empty
    /// when the port's own code does it.
    pub model: &'static str,
}

const KNOWN: &[Known] = &[
    Known { id: ids::SOFTWARE, slot: slots::VIDEO, label: "Software", model: "" },
    Known { id: ids::MMX, slot: slots::VIDEO, label: "Software, MMX", model: "crates/mmx" },
    Known { id: ids::GLIDE, slot: slots::VIDEO, label: "3dfx Voodoo (Glide)", model: "crates/voodoo" },
    Known { id: ids::DIRECT3D, slot: slots::VIDEO, label: "Direct3D", model: "wrappers/ddraw" },
    Known { id: ids::POWERVR, slot: slots::VIDEO, label: "PowerVR (SGL)", model: "wrappers/sgl" },
    Known { id: ids::ATI_RAGE, slot: slots::VIDEO, label: "ATI 3D Rage (ATI3DCIF)", model: "crates/rage" },
    Known { id: ids::S3_VIRGE, slot: slots::VIDEO, label: "S3 ViRGE (S3D Toolkit)", model: "crates/virge" },
    Known { id: ids::NONE, slot: slots::SOUND, label: "No sound", model: "" },
    Known { id: ids::DIRECTSOUND, slot: slots::SOUND, label: "DirectSound", model: "" },
    Known { id: ids::MAXI_SOUND_64, slot: slots::SOUND, label: "Guillemot Maxi Sound 64", model: "wrappers/maxi94" },
    Known { id: ids::SOUND_BLASTER, slot: slots::SOUND, label: "Sound Blaster", model: "crates/sound-blaster" },
    Known { id: ids::ADLIB, slot: slots::MUSIC, label: "AdLib (OPL2)", model: "crates/opl2" },
    Known { id: ids::OPL3, slot: slots::MUSIC, label: "Sound Blaster 16 (OPL3)", model: "crates/opl3" },
    Known { id: ids::ADLIB_GOLD, slot: slots::MUSIC, label: "AdLib Gold", model: "crates/adlib-gold" },
    Known { id: ids::GAME_BLASTER, slot: slots::MUSIC, label: "Game Blaster (CMS)", model: "crates/saa1099" },
    Known { id: ids::TANDY, slot: slots::MUSIC, label: "Tandy 3-voice", model: "crates/sn76496" },
    Known { id: ids::PC_SPEAKER, slot: slots::SOUND, label: "PC speaker", model: "" },
    Known { id: ids::GENERAL_MIDI, slot: slots::MUSIC, label: "General MIDI (MPU-401)", model: "" },
];

/// What GameFrame knows of `id`, or `None` for a game's own device.
pub fn known(id: &str) -> Option<Known> {
    KNOWN.iter().find(|k| k.id == id).copied()
}

/// Every device id GameFrame knows, in the order above.
pub fn all_known() -> &'static [Known] {
    KNOWN
}

/// The device chosen for `slot` in the data of `frame.ready` or `game.start`: its `devices`
/// object, and for the graphics slot the 1.0 `renderer` field when `devices` lacks it.
pub fn chosen<'a>(data: &'a Value, slot: &str) -> Option<&'a str> {
    data["devices"][slot]
        .as_str()
        .or_else(|| if slot == slots::VIDEO { data["renderer"].as_str() } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_known_id_once_in_a_known_slot() {
        let mut seen = std::collections::HashSet::new();
        for k in all_known() {
            assert!(seen.insert(k.id), "{} twice", k.id);
            assert!([slots::VIDEO, slots::SOUND, slots::MUSIC, slots::INPUT].contains(&k.slot));
        }
        assert_eq!(known(ids::GLIDE).unwrap().model, "crates/voodoo");
        assert!(known("pod-render").is_none());
    }

    #[test]
    fn chosen_reads_devices_then_the_renderer() {
        let data = json!({ "renderer": "software", "devices": { "sound": "none" } });
        assert_eq!(chosen(&data, slots::SOUND), Some("none"));
        assert_eq!(chosen(&data, slots::VIDEO), Some("software"));
        assert_eq!(chosen(&data, slots::MUSIC), None);
        let both = json!({ "renderer": "software", "devices": { "video": "glide" } });
        assert_eq!(chosen(&both, slots::VIDEO), Some("glide"));
    }
}
