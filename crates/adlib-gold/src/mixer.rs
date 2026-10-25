//! Digital control model for the AdLib Gold's documented source mixer.
//!
//! This models the signed linear volume controls and final stereo selection. The TDA8425's
//! frequency-dependent tone and phase effects remain separate because their exact transfer
//! curves depend on analog parts and the card revision.

const MIX_VOLUME_SILENCE: i32 = 128;
const MIX_VOLUME_SPAN: i64 = 127;
const OUTPUT_VOLUME_LEFT: usize = 0x04;
const OUTPUT_VOLUME_RIGHT: usize = 0x05;
const OUTPUT_MODE: usize = 0x08;
const MICROPHONE_FEEDBACK: u8 = 1 << 3;
const PC_SPEAKER_ENABLE: u8 = 1 << 5;

/// Stereo signal in signed host PCM units before AdLib Gold source-volume controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StereoSignal {
    pub left: i32,
    pub right: i32,
}

/// Digital sources available at the AdLib Gold card mixer.
///
/// `sampling` should contain the signed samples after the MMA's channel routing. The analog
/// inputs are supplied by the host machine because they are external to the card's digital
/// devices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdlibGoldAudioInputs {
    pub fm: StereoSignal,
    pub sampling: StereoSignal,
    pub auxiliary: StereoSignal,
    pub microphone: i32,
    pub telephone: i32,
    pub pc_speaker: i32,
}

/// Output-selection mode stored in Control Chip register `0x08`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OutputMode {
    ForcedMono = 0,
    LinearStereo = 1,
    PseudoStereo = 2,
    SpatialStereo = 3,
}

impl OutputMode {
    const fn from_register(value: u8) -> Self {
        match (value >> 3) & 0x03 {
            0 => Self::ForcedMono,
            1 => Self::LinearStereo,
            2 => Self::PseudoStereo,
            _ => Self::SpatialStereo,
        }
    }
}

/// Tone settings passed by Control Chip registers `0x06` and `0x07`.
///
/// The nibbles are preserved because the toolkit gives their ranges but not a complete transfer
/// function. They are not converted to dB here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToneSettings {
    pub bass_code: u8,
    pub treble_code: u8,
}

/// Result of the documented linear mixing and output-level controls.
///
/// The sample uses `i64` so a host can apply its own output clipping after adding all card
/// sources. `mode` and `tone` report controls whose analog TDA8425 transfer is not applied by
/// this digital stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdlibGoldMixFrame {
    pub left: i64,
    pub right: i64,
    pub mode: Option<OutputMode>,
    pub tone: ToneSettings,
}

/// Mix sources using Control Chip registers `0x04` through `0x10` and `0x08`.
pub(crate) fn mix(
    inputs: AdlibGoldAudioInputs,
    registers: &[u8; 0x19],
    telephone_installed: bool,
) -> AdlibGoldMixFrame {
    let mut left = source(inputs.fm.left, registers[0x09]);
    let mut right = source(inputs.fm.right, registers[0x0a]);

    left += source(inputs.sampling.left, registers[0x0b]);
    right += source(inputs.sampling.right, registers[0x0c]);
    left += source(inputs.auxiliary.left, registers[0x0d]);
    right += source(inputs.auxiliary.right, registers[0x0e]);

    if registers[0x11] & MICROPHONE_FEEDBACK == 0 {
        let microphone = source(inputs.microphone, registers[0x0f]);
        left += microphone;
        right += microphone;
    }
    if telephone_installed {
        let telephone = source(inputs.telephone, registers[0x10]);
        left += telephone;
        right += telephone;
    }
    if registers[0x11] & PC_SPEAKER_ENABLE != 0 {
        left += i64::from(inputs.pc_speaker);
        right += i64::from(inputs.pc_speaker);
    }

    // SOURCE uses bit 1 for the left mixer channel and bit 2 for the right mixer channel.
    // Selecting one channel feeds it to both output channels.
    match registers[OUTPUT_MODE] & 0x06 {
        0x02 => right = left,
        0x04 => left = right,
        0x06 => {}
        _ => {
            left = 0;
            right = 0;
        }
    }

    let mode = OutputMode::from_register(registers[OUTPUT_MODE]);
    if mode == OutputMode::ForcedMono {
        let mono = (left + right) / 2;
        left = mono;
        right = mono;
    }

    if registers[OUTPUT_MODE] & (1 << 5) != 0 {
        left = 0;
        right = 0;
    } else {
        left = apply_output_volume(left, registers[OUTPUT_VOLUME_LEFT]);
        right = apply_output_volume(right, registers[OUTPUT_VOLUME_RIGHT]);
    }

    AdlibGoldMixFrame {
        left,
        right,
        mode: Some(mode),
        tone: ToneSettings {
            bass_code: registers[0x06] & 0x0f,
            treble_code: registers[0x07] & 0x0f,
        },
    }
}

fn source(sample: i32, control: u8) -> i64 {
    (i64::from(sample) * (i64::from(control) - i64::from(MIX_VOLUME_SILENCE))) / MIX_VOLUME_SPAN
}

fn apply_output_volume(sample: i64, control: u8) -> i64 {
    // The toolkit's volume encoding is +6 dB at 63, falls in 2 dB steps through -64 dB at 28,
    // and specifies codes 0 and 27 as the -80 dB floor. Values 1 through 26 are not documented;
    // use the same floor rather than inventing extra attenuation steps.
    let code = control & 0x3f;
    let decibels = if code < 28 {
        -80
    } else {
        i16::from(code) * 2 - 120
    };
    let gain = 10.0_f64.powf(f64::from(decibels) / 20.0);
    (sample as f64 * gain).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registers() -> [u8; 0x19] {
        let mut registers = [0; 0x19];
        registers[0x04] = 0xfc; // 0 dB
        registers[0x05] = 0xfc; // 0 dB
        registers[0x08] = 0xce; // required ones, linear stereo, both mixer channels
        registers[0x09..=0x10].fill(0xff); // unity source gains
        registers
    }

    #[test]
    fn applies_documented_linear_source_and_master_gains() {
        let mut registers = registers();
        registers[0x04] = 0xff; // +6 dB
        registers[0x05] = 0xfc; // 0 dB
        let inputs = AdlibGoldAudioInputs {
            fm: StereoSignal {
                left: 1000,
                right: -1000,
            },
            ..AdlibGoldAudioInputs::default()
        };
        let frame = mix(inputs, &registers, false);
        assert_eq!(frame.left, 1995);
        assert_eq!(frame.right, -1000);
    }

    #[test]
    fn source_volume_128_is_silent_and_lower_codes_invert_polarity() {
        let mut registers = registers();
        registers[0x09] = 0x80;
        registers[0x0a] = 0x00;
        let frame = mix(
            AdlibGoldAudioInputs {
                fm: StereoSignal {
                    left: 800,
                    right: 800,
                },
                ..AdlibGoldAudioInputs::default()
            },
            &registers,
            false,
        );
        assert_eq!(frame.left, 0);
        assert_eq!(frame.right, -806);
    }

    #[test]
    fn selecting_one_mixer_channel_duplicates_it_to_both_outputs() {
        let mut registers = registers();
        registers[0x08] = 0xc2; // left mixer channel only
        let frame = mix(
            AdlibGoldAudioInputs {
                fm: StereoSignal {
                    left: 1000,
                    right: 3000,
                },
                ..AdlibGoldAudioInputs::default()
            },
            &registers,
            false,
        );
        assert_eq!((frame.left, frame.right), (1000, 1000));
    }

    #[test]
    fn output_mode_mute_and_tone_registers_are_reported() {
        let mut registers = registers();
        registers[0x08] = 0xce; // linear stereo, not muted
        registers[0x06] = 0xf9;
        registers[0x07] = 0xf3;
        let inputs = AdlibGoldAudioInputs {
            fm: StereoSignal {
                left: 100,
                right: -100,
            },
            ..AdlibGoldAudioInputs::default()
        };
        let frame = mix(inputs, &registers, false);
        assert_eq!((frame.left, frame.right), (100, -100));
        assert_eq!(frame.mode, Some(OutputMode::LinearStereo));
        assert_eq!(
            frame.tone,
            ToneSettings {
                bass_code: 9,
                treble_code: 3
            }
        );

        registers[0x08] |= 1 << 5;
        let muted = mix(inputs, &registers, false);
        assert_eq!((muted.left, muted.right), (0, 0));
    }

    #[test]
    fn mono_mode_and_optional_sources_follow_control_bits() {
        let mut registers = registers();
        registers[0x08] = 0xc6; // forced mono, not muted
        registers[0x11] = PC_SPEAKER_ENABLE;
        registers[0x0f] = 0xff;
        registers[0x10] = 0xff;
        let inputs = AdlibGoldAudioInputs {
            fm: StereoSignal {
                left: 200,
                right: 600,
            },
            microphone: 100,
            telephone: 200,
            pc_speaker: 300,
            ..AdlibGoldAudioInputs::default()
        };

        let with_phone = mix(inputs, &registers, true);
        assert_eq!((with_phone.left, with_phone.right), (1000, 1000));
        let without_phone = mix(inputs, &registers, false);
        assert_eq!((without_phone.left, without_phone.right), (800, 800));

        registers[0x11] |= MICROPHONE_FEEDBACK;
        let microphone_removed = mix(inputs, &registers, true);
        assert_eq!(
            (microphone_removed.left, microphone_removed.right),
            (900, 900)
        );
    }
}
