use gameframe_adlib_gold::{AdlibGold, AdlibGoldAudioInputs, AdlibGoldConfig, StereoSignal};

#[test]
fn public_card_mixer_uses_control_chip_configuration() {
    let mut config = AdlibGoldConfig::default();
    let registers = &mut config.initial_control_registers;
    registers[0x04] = 0xfc; // 0 dB left final output
    registers[0x05] = 0xfc; // 0 dB right final output
    registers[0x08] = 0xce; // linear stereo, both mixer channels, not muted
    registers[0x09] = 0xff; // FM left at unity
    registers[0x0a] = 0x80; // FM right silent

    let card = AdlibGold::new(config);
    let frame = card.mix_sources(AdlibGoldAudioInputs {
        fm: StereoSignal {
            left: 12_000,
            right: 12_000,
        },
        ..AdlibGoldAudioInputs::default()
    });

    assert_eq!((frame.left, frame.right), (12_000, 0));
}

#[test]
fn telephone_audio_requires_option_and_engaged_line() {
    let mut config = AdlibGoldConfig::default();
    config.identity.telephone = true;
    let registers = &mut config.initial_control_registers;
    registers[0x04] = 0xfc;
    registers[0x05] = 0xfc;
    registers[0x08] = 0xce;
    registers[0x10] = 0xff;
    let card = AdlibGold::new(config);
    let inputs = AdlibGoldAudioInputs {
        telephone: 1234,
        ..AdlibGoldAudioInputs::default()
    };
    assert_eq!(
        (
            card.mix_sources(inputs).left,
            card.mix_sources(inputs).right
        ),
        (0, 0)
    );

    let mut card = card;
    card.write_port_at(0x38a, 0xff, 0).unwrap();
    card.write_port_at(0x38a, 0x01, 8).unwrap();
    card.write_port_at(0x38b, 0x01, 40).unwrap();
    let frame = card.mix_sources(inputs);
    assert_eq!((frame.left, frame.right), (1234, 1234));
}
