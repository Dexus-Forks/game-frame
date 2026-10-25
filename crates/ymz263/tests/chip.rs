use gameframe_ymz263::{MmaAccessError, Ymz263, YMZ263_PORT_ACCESS_CLOCKS};

fn write_register(chip: &mut Ymz263, channel: u8, register: u8, value: u8, clock: &mut u64) {
    *clock = (*clock).max(chip.now() + YMZ263_PORT_ACCESS_CLOCKS);
    let index_port = channel * 2;
    chip.write_port_at(index_port, register, *clock).unwrap();
    *clock += YMZ263_PORT_ACCESS_CLOCKS;
    chip.write_port_at(index_port + 1, value, *clock).unwrap();
    *clock += YMZ263_PORT_ACCESS_CLOCKS;
}

fn read_register(chip: &mut Ymz263, channel: u8, register: u8, clock: &mut u64) -> Option<u8> {
    *clock = (*clock).max(chip.now() + YMZ263_PORT_ACCESS_CLOCKS);
    let index_port = channel * 2;
    chip.write_port_at(index_port, register, *clock).unwrap();
    *clock += YMZ263_PORT_ACCESS_CLOCKS;
    let result = chip.read_port_at(index_port + 1, *clock).unwrap();
    *clock += YMZ263_PORT_ACCESS_CLOCKS;
    result
}

fn start_playback(chip: &mut Ymz263, channel: u8, clock: &mut u64) {
    // Stereo route, PCM playback, initially stopped.
    write_register(chip, channel, 0x09, 0x66, clock);
    write_register(chip, channel, 0x0a, 0x55, clock);
}

#[test]
fn indexed_ports_enforce_eight_clock_spacing_and_monotonic_time() {
    let mut chip = Ymz263::new();
    chip.write_port_at(0, 0x09, 0).unwrap();
    assert_eq!(
        chip.write_port_at(1, 0x66, 7),
        Err(gameframe_ymz263::Ymz263PortError::Access(
            MmaAccessError::PortAccessTooSoon { wait_clocks: 1 }
        ))
    );
    chip.write_port_at(1, 0x66, 8).unwrap();
    assert_eq!(
        chip.read_port_at(0, 7),
        Err(gameframe_ymz263::Ymz263PortError::Access(
            MmaAccessError::TimeWentBackwards {
                previous: 8,
                received: 7
            }
        ))
    );
}

#[test]
fn pcm_playback_emits_signed_eight_bit_samples_at_the_documented_rate() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    start_playback(&mut chip, 0, &mut clock);
    write_register(&mut chip, 0, 0x0b, 0x80, &mut clock);
    write_register(&mut chip, 0, 0x09, 0x67, &mut clock);
    let deadline = chip.now() + 384;

    chip.advance_to(deadline - 1).unwrap();
    assert!(chip.drain_output_events().is_empty());
    chip.advance_to(deadline).unwrap();
    let events = chip.drain_output_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].clock, deadline);
    assert_eq!(events[0].channel, 0);
    assert_eq!(events[0].output.channels[0].sample, -2048);
    assert_eq!(events[0].output.channels[0].volume, 0x55);
    assert!(events[0].output.channels[0].left_enabled);
    assert!(events[0].output.channels[0].right_enabled);
}

#[test]
fn both_documented_twelve_bit_pcm_byte_layouts_decode_signed_samples() {
    for (format, bytes, expected) in [(0x20, [0xff, 0x08], -1793), (0x40, [0xf0, 0x80], -2033)] {
        let mut chip = Ymz263::new();
        let mut clock = 0;
        start_playback(&mut chip, 0, &mut clock);
        write_register(&mut chip, 0, 0x0c, format, &mut clock);
        write_register(&mut chip, 0, 0x0b, bytes[0], &mut clock);
        write_register(&mut chip, 0, 0x0b, bytes[1], &mut clock);
        write_register(&mut chip, 0, 0x09, 0x67, &mut clock);
        let deadline = chip.now() + 384;
        chip.advance_to(deadline).unwrap();
        let events = chip.drain_output_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].output.channels[0].sample, expected);
    }
}

#[test]
fn adpcm_playback_decodes_the_high_nibble_before_the_low_nibble() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x09, 0x62, &mut clock); // stopped ADPCM playback
    write_register(&mut chip, 0, 0x0b, 0x0f, &mut clock);
    write_register(&mut chip, 0, 0x09, 0x63, &mut clock); // ADPCM playback, 22.05 kHz

    let deadline = chip.now() + 2 * 768;
    chip.advance_to(deadline).unwrap();
    let events = chip.drain_output_events();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events
            .iter()
            .map(|event| event.output.channels[0].sample)
            .collect::<Vec<_>>(),
        [0, -14]
    );
}

#[test]
fn empty_playback_fifo_sets_digital_overrun_status() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x09, 0x67, &mut clock);
    chip.advance_to(chip.now() + 384).unwrap();
    assert_ne!(
        chip.read_port_at(0, chip.now() + 8).unwrap().unwrap() & 0x80,
        0
    );
}

#[test]
fn recording_writes_each_documented_pcm_layout_to_the_fifo() {
    for (format, expected) in [
        (0x00, [0x34, 0]),
        (0x20, [0x45, 0x03]),
        (0x40, [0x50, 0x34]),
    ] {
        let mut chip = Ymz263::new();
        let mut clock = 0;
        write_register(&mut chip, 0, 0x0c, format, &mut clock);
        chip.set_record_input(0, 0x345).unwrap();
        write_register(&mut chip, 0, 0x09, 0x65, &mut clock); // record, PCM, GO
        chip.advance_to(chip.now() + 384).unwrap();
        assert_eq!(
            read_register(&mut chip, 0, 0x0b, &mut clock),
            Some(expected[0])
        );
        if format != 0 {
            assert_eq!(
                read_register(&mut chip, 0, 0x0b, &mut clock),
                Some(expected[1])
            );
        }
    }
}

#[test]
fn fifo_threshold_drives_status_dma_and_masked_interrupt_independently() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x09, 0x66, &mut clock); // playback, stopped
    write_register(&mut chip, 0, 0x0c, 0x01, &mut clock); // DMA enabled, threshold 112
    write_register(&mut chip, 0, 0x0d, 0x35, &mut clock); // mask MIDI and timer requests
    assert_eq!(chip.dma_request(0), Some(true));
    assert_ne!(chip.read_port_at(0, clock).unwrap().unwrap() & 1, 0);
    assert!(chip.irq_pending());

    for _ in 0..113 {
        write_register(&mut chip, 0, 0x0b, 0x00, &mut clock);
    }
    assert_eq!(chip.fifo_level(0), Some(113));
    assert_eq!(chip.dma_request(0), Some(false));
    assert_eq!(chip.read_port_at(0, clock).unwrap().unwrap() & 1, 0);
    assert!(!chip.irq_pending());

    write_register(&mut chip, 0, 0x0c, 0x03, &mut clock); // mask FIFO IRQ, retain DMA
    assert_eq!(chip.dma_request(0), Some(false));
}

#[test]
fn interleaved_playback_alternates_channel_fifo_samples() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    start_playback(&mut chip, 0, &mut clock);
    start_playback(&mut chip, 1, &mut clock);
    write_register(&mut chip, 0, 0x0c, 0x81, &mut clock); // interleave and DMA enabled
    write_register(&mut chip, 1, 0x0c, 0x01, &mut clock); // both channels require DMA
    write_register(&mut chip, 0, 0x0b, 0x10, &mut clock);
    write_register(&mut chip, 1, 0x0b, 0x20, &mut clock);
    write_register(&mut chip, 0, 0x09, 0x67, &mut clock);
    write_register(&mut chip, 1, 0x09, 0x67, &mut clock);

    chip.advance_to(chip.now() + 2 * 384).unwrap();
    let events = chip.drain_output_events();
    assert_eq!(
        events.iter().map(|event| event.channel).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(events[0].output.channels[0].sample, 0x100);
    assert_eq!(events[1].output.channels[1].sample, 0x200);
}

#[test]
fn timer_zero_uses_32_clock_units_and_mask_only_gates_irq() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x02, 2, &mut clock);
    write_register(&mut chip, 0, 0x03, 0, &mut clock);
    write_register(&mut chip, 0, 0x0d, 0x35, &mut clock);
    write_register(&mut chip, 0, 0x08, 0x01, &mut clock);

    let deadline = chip.now() + 64;
    chip.advance_to(deadline - 1).unwrap();
    assert_eq!(
        chip.read_port_at(0, deadline - 1).unwrap().unwrap() & 0x10,
        0
    );
    chip.advance_to(deadline).unwrap();
    assert_ne!(
        chip.read_port_at(0, deadline + 8).unwrap().unwrap() & 0x10,
        0
    );
    assert!(chip.irq_pending());

    write_register(&mut chip, 0, 0x08, 0x11, &mut clock);
    assert!(!chip.irq_pending());
}

#[test]
fn timer_one_and_two_count_base_ticks_and_timer_two_low_read_latches_high() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x04, 1, &mut clock); // base period = 32 clocks
    write_register(&mut chip, 0, 0x05, 0x20, &mut clock); // timer 1 reload = 2
    write_register(&mut chip, 0, 0x06, 2, &mut clock); // timer 2 reload = 2
    write_register(&mut chip, 0, 0x07, 0, &mut clock);
    write_register(&mut chip, 0, 0x08, 0x0e, &mut clock); // base, timer 1, timer 2

    chip.advance_to(chip.now() + 32).unwrap();
    assert_eq!(read_register(&mut chip, 0, 0x06, &mut clock), Some(1));
    chip.advance_to(chip.now() + 32).unwrap();
    let status = chip.read_port_at(0, chip.now() + 8).unwrap().unwrap();
    assert_ne!(status & 0x20, 0);
    assert_ne!(status & 0x40, 0);
    assert_eq!(read_register(&mut chip, 0, 0x07, &mut clock), Some(0));
}

#[test]
fn midi_uart_transmits_at_31250_baud_and_receive_fifo_overrun_is_resettable() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x0d, 0x35, &mut clock);
    let first_tx_start = clock + YMZ263_PORT_ACCESS_CLOCKS;
    write_register(&mut chip, 0, 0x0e, 0x90, &mut clock);
    write_register(&mut chip, 0, 0x0e, 0x40, &mut clock);
    write_register(&mut chip, 0, 0x0e, 0x7f, &mut clock);
    let first_done = first_tx_start + 5419;
    chip.advance_to(first_done).unwrap();
    chip.advance_to(first_done + 2 * 5419).unwrap();
    let tx = chip.drain_midi_events();
    assert_eq!(
        tx.iter().map(|event| event.value).collect::<Vec<_>>(),
        [0x90, 0x40, 0x7f]
    );
    assert_eq!(tx[0].clock, first_done);
    assert_eq!(tx[1].clock, first_done + 5419);
    assert_eq!(tx[2].clock, first_done + 2 * 5419);

    let mut receive_clock = chip.now();
    for value in 0..17 {
        chip.receive_midi_byte(value, receive_clock).unwrap();
        receive_clock += 1;
    }
    assert_eq!(
        chip.read_port_at(0, receive_clock + 8).unwrap().unwrap() & 0x84,
        0x84
    );
    write_register(&mut chip, 0, 0x0d, 0x3d, &mut receive_clock); // retain masks, reset RX
    assert_eq!(
        chip.read_port_at(0, receive_clock).unwrap().unwrap() & 0x8c,
        0x08
    );
    assert_eq!(read_register(&mut chip, 0, 0x0e, &mut receive_clock), None);
}

#[test]
fn channel_one_has_an_independent_register_pair_and_invalid_offsets_are_rejected() {
    let mut chip = Ymz263::new();
    let mut clock = 0;
    write_register(&mut chip, 0, 0x0a, 0x12, &mut clock);
    write_register(&mut chip, 1, 0x0a, 0x34, &mut clock);
    assert_eq!(chip.sample_output().channels[0].volume, 0x12);
    assert_eq!(chip.sample_output().channels[1].volume, 0x34);
    assert!(chip.read_port_at(4, clock).is_err());
    assert!(chip.write_port_at(4, 0, clock).is_err());
    assert_eq!(chip.set_record_input(2, 0), Err(2));
}
