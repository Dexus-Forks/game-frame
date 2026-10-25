use super::{
    ActiveTransfer, DmaDirection, Dsp, DspEvent, MidiMode, PendingCommand, TimeError, TransferWidth,
};

impl Dsp {
    /// Write one byte to DSP command/data port 2xCh. False means the DSP was still busy.
    pub fn write_command_at(&mut self, value: u8, at_ns: u64) -> Result<bool, TimeError> {
        self.advance_clock_to(at_ns)?;
        if self
            .active_transfer
            .is_some_and(|transfer| transfer.high_speed)
        {
            return Ok(false);
        }
        if self.write_status() & 0x80 != 0 {
            return Ok(false);
        }
        self.write_busy_until_ns = at_ns.saturating_add(self.write_busy_ns);

        if self.midi_mode.uart() {
            self.events.push_back(DspEvent::MidiOutput { at_ns, value });
            return Ok(true);
        }
        if self.midi_output_armed {
            self.midi_output_armed = false;
            self.events.push_back(DspEvent::MidiOutput { at_ns, value });
            return Ok(true);
        }
        if let Some(pending) = self.pending_command {
            self.parameters.push(value);
            if self.parameters.len() >= pending.expected as usize {
                self.pending_command = None;
                let parameters = std::mem::take(&mut self.parameters);
                self.execute_command(pending.command, &parameters, at_ns);
            }
            return Ok(true);
        }

        let expected = self.command_parameter_count(value);
        if expected == 0 {
            self.execute_command(value, &[], at_ns);
        } else {
            self.pending_command = Some(PendingCommand {
                command: value,
                expected,
            });
            self.parameters.clear();
        }
        Ok(true)
    }

    fn command_parameter_count(&self, command: u8) -> u8 {
        match command {
            0x10 | 0x40 => 1,
            0x14 | 0x24 | 0x16 | 0x17 | 0x74..=0x77 | 0x80 => 2,
            0x41 | 0x42 if self.version.major >= 4 => 2,
            0x48 => 2,
            0xC0..=0xCF | 0xB0..=0xBF if self.version.major >= 4 => 3,
            _ => 0,
        }
    }

    fn execute_command(&mut self, command: u8, args: &[u8], _at_ns: u64) {
        match command {
            0x10 => self.direct_sample = args.first().copied(),
            0x20 => self.direct_input_pending = true,
            0x14 | 0x24 => {
                if let Some(count) = word(args) {
                    self.start_transfer(
                        command == 0x14,
                        TransferWidth::EightBit,
                        false,
                        false,
                        false,
                        false,
                        None,
                        Some(count),
                        Some(count),
                        false,
                    );
                }
            }
            0x1C | 0x2C if self.version.at_least(2, 0) => {
                let output = command == 0x1C;
                let stereo = if output {
                    self.legacy_output_stereo
                } else {
                    self.legacy_input_stereo
                };
                self.start_transfer(
                    output,
                    TransferWidth::EightBit,
                    stereo,
                    false,
                    true,
                    false,
                    None,
                    None,
                    None,
                    false,
                );
            }
            0x74 | 0x75 | 0x76 | 0x77 | 0x16 | 0x17 => {
                let count = word(args).unwrap_or(1);
                let bits = adpcm_width(command);
                let has_reference = matches!(command, 0x75 | 0x77 | 0x17);
                self.start_transfer(
                    true,
                    TransferWidth::EightBit,
                    false,
                    false,
                    false,
                    false,
                    Some(bits),
                    Some(count),
                    Some(count),
                    has_reference,
                );
                if has_reference {
                    self.direct_input_pending = false;
                }
            }
            0x1F | 0x7D | 0x7F if self.version.at_least(2, 0) => {
                self.start_transfer(
                    true,
                    TransferWidth::EightBit,
                    false,
                    false,
                    true,
                    false,
                    Some(adpcm_width(command)),
                    None,
                    None,
                    true,
                );
            }
            0x48 => self.block_size = word(args).unwrap_or(1),
            0x40 => {
                self.time_constant = args[0];
                self.rate_phase = 0;
            }
            0x41 if self.version.major >= 4 => {
                self.output_rate_hz = rate_word(args).unwrap_or(0).max(1) as u32;
                self.rate_phase = 0;
            }
            0x42 if self.version.major >= 4 => {
                self.input_rate_hz = rate_word(args).unwrap_or(0).max(1) as u32;
                self.rate_phase = 0;
            }
            0x80 => self.pause_samples = word(args).unwrap_or(1),
            0x90 | 0x91 | 0x98 | 0x99 if self.version.at_least(2, 1) && self.version.major < 4 => {
                let output = matches!(command, 0x90 | 0x91);
                let auto_init = matches!(command, 0x90 | 0x98);
                let count = self.block_size.max(1);
                let stereo = if output {
                    self.legacy_output_stereo
                } else {
                    self.legacy_input_stereo
                };
                self.start_transfer(
                    output,
                    TransferWidth::EightBit,
                    stereo,
                    false,
                    auto_init,
                    true,
                    None,
                    Some(count),
                    Some(count),
                    false,
                );
            }
            0xA0 if self.version.major == 3 => self.legacy_input_stereo = false,
            0xA8 if self.version.major == 3 => self.legacy_input_stereo = true,
            0xB0..=0xBF | 0xC0..=0xCF if self.version.major >= 4 => {
                let width = if command & 0xF0 == 0xB0 {
                    TransferWidth::SixteenBit
                } else {
                    TransferWidth::EightBit
                };
                let mode = args.first().copied().unwrap_or(0);
                let count = word(&args[1..]).unwrap_or(1);
                let input = command & 0x02 != 0;
                let auto_init = command & 0x04 != 0;
                self.start_transfer(
                    !input,
                    width,
                    mode & 0x20 != 0,
                    mode & 0x10 != 0,
                    auto_init,
                    false,
                    None,
                    Some(count),
                    Some(count),
                    false,
                );
            }
            0xD0 => self.pause_width(TransferWidth::EightBit, true),
            0xD4 => self.pause_width(TransferWidth::EightBit, false),
            0xD5 if self.version.major >= 4 => self.pause_width(TransferWidth::SixteenBit, true),
            0xD6 if self.version.major >= 4 => self.pause_width(TransferWidth::SixteenBit, false),
            0xD1 => self.speaker_on = true,
            0xD3 => self.speaker_on = false,
            0xD8 if self.version.at_least(2, 0) => {
                self.response
                    .push_back(if self.speaker_on { 0xFF } else { 0x00 })
            }
            0xD9 if self.version.major >= 4 => self.request_exit(TransferWidth::SixteenBit),
            0xDA => self.request_exit(TransferWidth::EightBit),
            0xE1 => {
                self.response.push_back(self.version.major);
                self.response.push_back(self.version.minor);
            }
            0x30 => self.midi_mode = MidiMode::NormalPolling,
            0x31 => self.midi_mode = MidiMode::NormalInterrupt,
            0x34 if self.version.at_least(2, 0) => self.midi_mode = MidiMode::UartPolling,
            0x35 if self.version.at_least(2, 0) => self.midi_mode = MidiMode::UartInterrupt,
            0x36 if self.version.at_least(2, 0) => self.midi_mode = MidiMode::UartTimestampPolling,
            0x37 if self.version.at_least(2, 0) => {
                self.midi_mode = MidiMode::UartTimestampInterrupt
            }
            0x38 => self.midi_output_armed = true,
            _ => {}
        }
        if command == 0x30
            || command == 0x31
            || command == 0x34
            || command == 0x35
            || command == 0x36
            || command == 0x37
        {
            self.midi_irq = false;
        }
    }

    fn start_transfer(
        &mut self,
        output: bool,
        width: TransferWidth,
        stereo: bool,
        signed: bool,
        auto_init: bool,
        high_speed: bool,
        adpcm_bits: Option<u8>,
        count: Option<u32>,
        reload: Option<u32>,
        adpcm_reference_pending: bool,
    ) {
        self.pending_dma.clear();
        let count = count.map(|v| v.max(1));
        self.active_transfer = Some(ActiveTransfer {
            direction: if output {
                DmaDirection::MemoryToCard
            } else {
                DmaDirection::CardToMemory
            },
            width,
            stereo,
            signed,
            auto_init,
            high_speed,
            adpcm_bits,
            adpcm_reference_pending,
            remaining: if high_speed { None } else { count },
            reload: reload.map(|v| v.max(1)),
            block_remaining: if high_speed {
                count.or(Some(self.block_size.max(1)))
            } else {
                None
            },
            exit_at_block: false,
            paused: false,
        });
        self.rate_phase = 0;
        self.stereo_next_right = false;
    }

    fn pause_width(&mut self, width: TransferWidth, paused: bool) {
        if let Some(mut transfer) = self.active_transfer {
            if transfer.width == width {
                transfer.paused = paused;
                self.active_transfer = Some(transfer);
            }
        }
    }

    fn request_exit(&mut self, width: TransferWidth) {
        if let Some(mut transfer) = self.active_transfer {
            if transfer.width == width && transfer.auto_init {
                transfer.exit_at_block = true;
                self.active_transfer = Some(transfer);
            }
        }
    }
}

fn word(bytes: &[u8]) -> Option<u32> {
    Some(raw_word(bytes)? as u32 + 1)
}

fn raw_word(bytes: &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes([*bytes.first()?, *bytes.get(1)?]))
}

fn rate_word(bytes: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]))
}

fn adpcm_width(command: u8) -> u8 {
    match command {
        0x16 | 0x17 | 0x1F => 2,
        0x76 | 0x77 | 0x7F => 3,
        _ => 4,
    }
}
