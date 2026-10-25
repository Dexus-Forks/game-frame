use gameframe_sound_blaster::{
    DspEvent, Mixer, MixerKind, PcmChannel, SoundBlaster, SoundBlasterConfig,
};

fn command(card: &mut SoundBlaster, value: u8, t: u64) {
    assert!(card.write_port_at(0x22C, value, t).unwrap());
}

#[test]
fn dsp_reset_waits_for_ready_signature_and_version_query_returns_profile() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    card.write_port_at(0x226, 1, 0).unwrap();
    card.write_port_at(0x226, 0, 3_000).unwrap();
    assert_eq!(card.read_port_at(0x22E, 102_999).unwrap(), Some(0x00));
    assert_eq!(card.read_port_at(0x22E, 103_000).unwrap(), Some(0x80));
    assert_eq!(card.read_port_at(0x22A, 103_000).unwrap(), Some(0xAA));

    command(&mut card, 0xE1, 103_001);
    assert_eq!(card.read_port_at(0x22A, 103_001).unwrap(), Some(4));
    assert_eq!(card.read_port_at(0x22A, 103_001).unwrap(), Some(5));
}

#[test]
fn dsp_reset_rejects_a_pulse_shorter_than_three_microseconds() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    card.write_port_at(0x226, 1, 0).unwrap();
    card.write_port_at(0x226, 0, 2_999).unwrap();
    assert_eq!(card.read_port_at(0x22E, 102_999).unwrap(), Some(0x00));
    assert_eq!(card.read_port_at(0x22A, 102_999).unwrap(), Some(0xFF));
}

#[test]
fn sb16_eight_bit_single_cycle_requests_dma_emits_pcm_and_latches_irq() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    command(&mut card, 0xD1, 0);
    command(&mut card, 0x41, 1);
    command(&mut card, 0x56, 1);
    command(&mut card, 0x22, 1);
    command(&mut card, 0xC0, 2);
    command(&mut card, 0x00, 2);
    command(&mut card, 0x01, 2);
    command(&mut card, 0x00, 2);

    let scheduled = card.advance_to(100_000).unwrap();
    assert_eq!(
        scheduled
            .iter()
            .filter(|event| matches!(event, DspEvent::DmaRequest(_)))
            .count(),
        2
    );
    let first = card.service_dma_read(0x00).unwrap();
    let second = card.service_dma_read(0xFF).unwrap();
    assert!(
        matches!(first, DspEvent::Pcm(sample) if sample.channel == PcmChannel::Mono && sample.value == i16::MIN)
    );
    assert!(matches!(second, DspEvent::Pcm(sample) if sample.value == 32_512));
    assert!(card.irq_pending());

    card.write_port_at(0x224, 0x82, 100_000).unwrap();
    assert_eq!(card.read_port_at(0x225, 100_000).unwrap(), Some(0x01));
    card.read_port_at(0x22E, 100_000).unwrap();
    card.write_port_at(0x224, 0x82, 100_000).unwrap();
    assert_eq!(card.read_port_at(0x225, 100_000).unwrap(), Some(0x00));
}

#[test]
fn sb16_sixteen_bit_stereo_decodes_alternating_signed_channels() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    command(&mut card, 0xD1, 0);
    command(&mut card, 0x41, 1);
    command(&mut card, 0x03, 1); // 1000 Hz.
    command(&mut card, 0xE8, 1);
    command(&mut card, 0xB0, 2);
    command(&mut card, 0x30, 2); // stereo, signed
    command(&mut card, 0x01, 2); // two words
    command(&mut card, 0x00, 2);

    let scheduled = card.advance_to(1_100_000).unwrap();
    assert_eq!(
        scheduled
            .iter()
            .filter(|event| matches!(event, DspEvent::DmaRequest(_)))
            .count(),
        2
    );
    assert!(
        matches!(card.service_dma_read(0x8000), Some(DspEvent::Pcm(sample)) if sample.channel == PcmChannel::Left && sample.value == i16::MIN)
    );
    assert!(
        matches!(card.service_dma_read(0x7FFF), Some(DspEvent::Pcm(sample)) if sample.channel == PcmChannel::Right && sample.value == i16::MAX)
    );
    assert!(card.irq_pending());
}

#[test]
fn sb20_auto_init_raises_irq_when_the_host_reports_dma_terminal_count() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb_20());
    command(&mut card, 0x1C, 0);
    card.dsp_mut().dma_terminal_count_at(1_000).unwrap();
    assert!(card.irq_pending());
    assert_eq!(card.read_port_at(0x22E, 1_000).unwrap(), Some(0x00));
    assert!(!card.irq_pending());
}

#[test]
fn sb20_high_speed_mode_ignores_commands_until_reset() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb_20());
    command(&mut card, 0x40, 0);
    command(&mut card, 0xFF, 0); // Maximum legacy rate for a short generated check.
    command(&mut card, 0x90, 1); // High-speed auto-initialize output.
    assert!(!card.write_port_at(0x22C, 0xD1, 2).unwrap());
    assert!(!card.dsp().speaker_on());

    card.write_port_at(0x226, 1, 3).unwrap();
    card.write_port_at(0x226, 0, 6_003).unwrap();
    assert!(card.write_port_at(0x22C, 0xD1, 6_004).unwrap());
    assert!(card.dsp().speaker_on());
}

#[test]
fn sb16_capture_rate_is_separate_and_dma_writes_captured_samples() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    command(&mut card, 0x41, 0);
    command(&mut card, 0x56, 0);
    command(&mut card, 0x22, 0); // output rate 22050 Hz
    command(&mut card, 0x42, 1);
    command(&mut card, 0x03, 1);
    command(&mut card, 0xE8, 1); // input rate 1000 Hz
    assert_eq!(card.dsp().sample_rate_hz(), 22_050);
    assert_eq!(card.dsp().input_sample_rate_hz(), 1_000);

    command(&mut card, 0xC2, 2);
    command(&mut card, 0x00, 2); // unsigned mono
    command(&mut card, 0x00, 2); // one byte
    command(&mut card, 0x00, 2);
    card.push_capture_sample(0x12_34);
    let scheduled = card.advance_to(1_100_000).unwrap();
    assert!(scheduled
        .iter()
        .any(|event| matches!(event, DspEvent::DmaWrite { value: 0x34, .. })));
    assert!(card.irq_pending());
}

#[test]
fn sb16_sixteen_bit_capture_preserves_the_dma_word() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    command(&mut card, 0x42, 0);
    command(&mut card, 0x03, 0);
    command(&mut card, 0xE8, 0); // 1000 Hz input.
    command(&mut card, 0xB2, 1); // 16-bit, single-cycle input.
    command(&mut card, 0x10, 1); // Signed mono.
    command(&mut card, 0x00, 1);
    command(&mut card, 0x00, 1);
    card.push_capture_sample(0x8123);

    let scheduled = card.advance_to(1_100_000).unwrap();
    assert!(scheduled.iter().any(|event| matches!(
        event,
        DspEvent::DmaWrite {
            value: 0x8123,
            width: gameframe_sound_blaster::TransferWidth::SixteenBit,
            ..
        }
    )));
    assert!(card.irq_pending());
}

#[test]
fn empty_dsp_data_port_is_still_mapped() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    assert_eq!(card.read_port_at(0x22A, 0).unwrap(), Some(0xFF));
    assert_eq!(card.read_port_at(0x239, 0).unwrap(), None);
}

#[test]
fn creative_adpcm_command_consumes_compressed_data_without_claiming_pcm_decode() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb_15());
    command(&mut card, 0xD1, 0);
    command(&mut card, 0x40, 1);
    command(&mut card, 0xA5, 1);
    command(&mut card, 0x75, 2);
    command(&mut card, 0x00, 2);
    command(&mut card, 0x00, 2);
    card.advance_to(200_000).unwrap();
    assert!(matches!(
        card.service_dma_read(0xA7),
        Some(DspEvent::AdpcmByte {
            value: 0xA7,
            bits_per_sample: 4,
            reference_byte: true,
            ..
        })
    ));
}

#[test]
fn mixer_register_profiles_keep_generation_specific_masks_and_defaults() {
    let mut early = Mixer::new(MixerKind::Ct1335);
    assert_eq!(early.read(0x02), 4);
    early.write(0x02, 0xFF);
    assert_eq!(early.read(0x02), 7);
    assert_eq!(early.read(0x22), 0xFF);

    let mut pro = Mixer::new(MixerKind::Ct1345);
    assert_eq!(pro.read(0x22), 0x88);
    pro.write(0x0E, 0xFF);
    assert_eq!(pro.read(0x0E), 0x22);

    let mut sb16 = Mixer::new(MixerKind::Ct1745);
    assert_eq!(sb16.read(0x30), 24);
    assert_eq!(sb16.read(0x80), 0x02);
    assert_eq!(sb16.read(0x81), 0x22);
    sb16.write(0x82, 0xFF);
    assert_eq!(sb16.read(0x82), 0);
}

#[test]
fn sb16_mpu_uart_acknowledges_mode_and_forwards_midi_data() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    card.mpu401_mut().unwrap().set_write_busy_ns(10);
    card.write_port_at(0x331, 0x3F, 0).unwrap();
    assert_eq!(card.read_port_at(0x331, 0).unwrap(), Some(0x40));
    assert_eq!(card.read_port_at(0x331, 10).unwrap(), Some(0x00));
    assert!(card.irq_pending());
    assert!(card.drain_events().iter().any(|event| matches!(
        event,
        DspEvent::InterruptRaised {
            source: gameframe_sound_blaster::InterruptSource::Mpu401,
            ..
        }
    )));
    assert_eq!(card.read_port_at(0x330, 10).unwrap(), Some(0xFE));
    assert!(!card.irq_pending());
    assert!(card.push_mpu_input_at(0x90, 11).unwrap());
    assert!(card.irq_pending());
    assert_eq!(card.read_port_at(0x330, 11).unwrap(), Some(0x90));
    assert!(!card.irq_pending());
    card.write_port_at(0x330, 0x90, 12).unwrap();
    card.write_port_at(0x330, 60, 13).unwrap();
    card.write_port_at(0x330, 100, 14).unwrap();
    assert_eq!(
        card.drain_mpu_output()
            .iter()
            .map(|event| event.value)
            .collect::<Vec<_>>(),
        [0x90, 60, 100]
    );
}

#[test]
fn sb16_interrupt_status_register_reports_8_and_16_bit_sources() {
    let mut card = SoundBlaster::new(SoundBlasterConfig::sb16());
    card.dsp_mut().push_midi_input_at(0xF8, 0).unwrap(); // Ignored until MIDI mode is entered.
    command(&mut card, 0xD1, 1);
    command(&mut card, 0x41, 2);
    command(&mut card, 0x03, 2);
    command(&mut card, 0xE8, 2);
    command(&mut card, 0xB0, 3);
    command(&mut card, 0x10, 3); // signed mono 16-bit
    command(&mut card, 0x00, 3);
    command(&mut card, 0x00, 3);
    card.advance_to(1_100_000).unwrap();
    card.service_dma_read(0).unwrap();
    card.write_port_at(0x224, 0x82, 1_100_000).unwrap();
    assert_eq!(card.read_port_at(0x225, 1_100_000).unwrap(), Some(0x02));
    card.read_port_at(0x22F, 1_100_000).unwrap();
    card.write_port_at(0x224, 0x82, 1_100_000).unwrap();
    assert_eq!(card.read_port_at(0x225, 1_100_000).unwrap(), Some(0x00));
}
