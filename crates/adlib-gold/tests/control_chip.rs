use gameframe_adlib_gold::{
    AdlibGold, AdlibGoldConfig, AdlibGoldPortError, BusModel, ControlChipError, InterruptSource,
    CONTROL_REGISTER_BUSY_CLOCKS, MIXER_REGISTER_BUSY_CLOCKS, OPL3_MASTER_CLOCK_HZ,
};
use gameframe_ymz263::YMZ263_MASTER_CLOCK_HZ;

const BASE: u16 = 0x388;

fn enter_control(card: &mut AdlibGold, clock: u64) {
    assert!(card.write_port_at(BASE + 2, 0xff, clock).unwrap());
}

fn write_register(card: &mut AdlibGold, register: u8, value: u8, clock: u64) -> u64 {
    card.write_port_at(BASE + 2, register, clock).unwrap();
    card.write_port_at(BASE + 3, value, clock + 32).unwrap();
    clock + 32
}

fn read_register(card: &mut AdlibGold, register: u8, clock: u64) -> u8 {
    card.write_port_at(BASE + 2, register, clock).unwrap();
    card.read_port_at(BASE + 3, clock + 32).unwrap().unwrap()
}

fn read_status(card: &mut AdlibGold, clock: u64) -> u8 {
    card.read_port_at(BASE + 2, clock).unwrap().unwrap()
}

#[test]
fn default_ports_relocate_and_control_chip_status_exposes_active_low_inputs() {
    let mut config = AdlibGoldConfig::default();
    config.identity = gameframe_adlib_gold::BoardIdentity {
        bus_model: BusModel::MicroChannel,
        telephone: true,
        surround: true,
        cd_rom: true,
    };
    let mut card = AdlibGold::new(config);
    assert_eq!(card.audio_base(), BASE);
    enter_control(&mut card, 0);
    card.write_port_at(BASE + 4, 0x0d, 8).unwrap();
    card.write_port_at(BASE + 5, 0x35, 16).unwrap(); // original driver masks idle MIDI requests

    assert_eq!(read_register(&mut card, 0, 40), 0x08); // MCA model, all options fitted
    assert_eq!(read_status(&mut card, 80) & 0x0f, 0x0f);
    card.set_interrupt_pending(InterruptSource::Fm, true);
    card.set_interrupt_pending(InterruptSource::Sampling, true);
    let status = read_status(&mut card, 88);
    assert_eq!(status & 0x0f, 0x0c);
    assert_eq!(read_status(&mut card, 96) & 0x0f, 0x0c); // reads do not clear requests

    card.set_telephone_ringing(true);
    let data = read_register(&mut card, 1, 104);
    assert_eq!(data & 0x02, 0); // ring input is active low
}

#[test]
fn mma_fifo_interrupt_is_reflected_in_control_chip_status() {
    let mut card = AdlibGold::default();
    card.write_port_at(BASE + 4, 0x0d, 0).unwrap();
    card.write_port_at(BASE + 5, 0x35, 8).unwrap(); // mask idle MIDI requests
    card.write_port_at(BASE + 4, 0x0c, 16).unwrap();
    card.write_port_at(BASE + 5, 0x01, 24).unwrap(); // DMA enabled, threshold 112
    card.write_port_at(BASE + 4, 0x09, 32).unwrap();
    card.write_port_at(BASE + 5, 0x66, 40).unwrap(); // PCM playback, left and right
    enter_control(&mut card, 48);

    let status = read_status(&mut card, 56);
    assert_eq!(status & 0x02, 0); // sampling interrupt is active low
}

#[test]
fn control_register_busy_windows_reject_access_until_the_exact_deadline() {
    let mut card = AdlibGold::default();
    enter_control(&mut card, 0);
    let written_at = write_register(&mut card, 9, 0x44, 8);
    assert_ne!(
        read_status(&mut card, written_at + CONTROL_REGISTER_BUSY_CLOCKS - 1) & 0x40,
        0
    );
    assert_eq!(
        read_status(&mut card, written_at + CONTROL_REGISTER_BUSY_CLOCKS) & 0x40,
        0
    );

    let next_write = write_register(&mut card, 4, 0xff, 200);
    assert_ne!(
        read_status(&mut card, next_write + MIXER_REGISTER_BUSY_CLOCKS - 1) & 0x40,
        0
    );
    assert_eq!(
        read_status(&mut card, next_write + MIXER_REGISTER_BUSY_CLOCKS) & 0x40,
        0
    );

    let pending = write_register(
        &mut card,
        5,
        0xff,
        next_write + MIXER_REGISTER_BUSY_CLOCKS + 1,
    );
    let rejected = card.write_port_at(BASE + 2, 6, pending + 1);
    assert!(matches!(
        rejected,
        Err(AdlibGoldPortError::ControlChip(
            ControlChipError::Busy { .. }
        ))
    ));
}

#[test]
fn eeprom_save_and_restore_preserve_registers_and_block_reads_while_busy() {
    let mut card = AdlibGold::default();
    enter_control(&mut card, 0);
    write_register(&mut card, 9, 0x5a, 8);
    write_register(&mut card, 0, 0x02, 120); // save current image
    assert_ne!(read_status(&mut card, 152) & 0x80, 0);
    assert_eq!(read_status(&mut card, 153) & 0x80, 0);

    write_register(&mut card, 9, 0xa5, 200);
    let restore_at = 320;
    write_register(&mut card, 0, 0x01, restore_at); // restore saved image
    let restore_done = restore_at
        + 32
        + AdlibGoldConfig::default()
            .control_timing
            .eeprom_restore_busy_clocks;
    assert_ne!(read_status(&mut card, restore_at + 40) & 0x80, 0);
    assert!(matches!(
        card.write_port_at(BASE + 2, 9, restore_at + 48),
        Err(AdlibGoldPortError::ControlChip(
            ControlChipError::Busy { .. }
        ))
    ));
    assert_eq!(read_register(&mut card, 9, restore_done), 0x5a);
    assert_eq!(read_status(&mut card, restore_done + 40) & 0x80, 0);
}

#[test]
fn relocation_moves_all_eight_ports_on_the_data_write() {
    let mut card = AdlibGold::default();
    card.write_port_at(BASE + 4, 0x0d, 0).unwrap();
    card.write_port_at(BASE + 5, 0x35, 8).unwrap();
    enter_control(&mut card, 16);
    let data_at = write_register(&mut card, 0x15, 0x72, 24);
    assert_eq!(card.audio_base(), 0x390);
    assert_eq!(card.read_port_at(BASE, data_at + 8).unwrap(), None);
    assert_eq!(
        card.read_port_at(0x390 + 2, data_at + 72).unwrap(),
        Some(0x0f)
    );
    assert!(card.write_port_at(0x390 + 2, 0xfe, data_at + 80).unwrap());
    assert!(!card.write_port_at(BASE + 2, 0xff, data_at + 88).unwrap());
}

#[test]
fn card_clock_conversion_advances_mma_on_the_shared_timeline() {
    let mut card = AdlibGold::default();
    let clock = OPL3_MASTER_CLOCK_HZ + 123;
    card.advance_to(clock).unwrap();
    let expected = (u128::from(clock) * u128::from(YMZ263_MASTER_CLOCK_HZ)
        / u128::from(OPL3_MASTER_CLOCK_HZ)) as u64;
    assert_eq!(card.mma().now(), expected);
}
