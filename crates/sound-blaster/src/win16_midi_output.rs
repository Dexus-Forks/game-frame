//! Logical model of the bounded Win16 SB16 MIDI-output open and short-message paths.
//!
//! This module does not implement the Win16 ABI, decode packed short-message fields, or model
//! the lower service or physical MIDI output. Callers supply lower-service availability,
//! initialization and allocation outcomes, the current per-port active-call count, an ordered
//! byte sequence, and an opaque lower-service event. The packed input layout, data-count table,
//! incomplete-message handling, callback timing, long messages, close/reset behavior, and hardware
//! output remain unknown.

pub const MIDI_OUTPUT_CALLBACK_VALUE_OPEN: u16 = 0x03C7;

pub const MIDI_OUTPUT_STATUS_SUCCESS: u16 = 0;
pub const MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT: u16 = 2;
pub const MIDI_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE: u16 = 3;
pub const MIDI_OUTPUT_STATUS_INITIALIZATION_FAILED: u16 = 4;
pub const MIDI_OUTPUT_STATUS_STATE_ALLOCATION_FAILED: u16 = 7;
pub const MIDI_OUTPUT_STATUS_REENTRANCY_REJECTED: u16 = 0x43;

const MIDI_PORT_COUNT: usize = 2;

/// State of the lower audio service, supplied by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerAudioState {
    Available,
    Unavailable,
    Unknown,
}

/// Whether the open descriptor or device context is known to be valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceContextState {
    Valid,
    Invalid,
    Unknown,
}

/// Result of the local per-port state allocation attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortStateAllocation {
    Allocated,
    Failed,
    Unknown,
}

/// Result of the lower service's port initialization call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortInitialization {
    Initialized,
    Failed(FailedInitializationContext),
    Unknown,
}

/// Whether a port context remains usable after initialization fails.
///
/// The contract records status 4 but does not establish cleanup behavior on that path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailedInitializationContext {
    Retained,
    Released,
    Unknown,
}

/// Opaque logical callback target information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackTargetState {
    Unknown,
    Absent,
    /// An opaque target identity, not a Win16 callback handle or pointer.
    Present(u64),
}

/// Callback information copied into per-port state during open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackInfo {
    pub target: CallbackTargetState,
    /// Opaque caller token. Its Win16 representation and callback arguments are not modeled.
    pub data: u64,
}

/// The result of a successful open's raw callback notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenCallback {
    Sent {
        target_id: u64,
        /// Raw observed callback value; no symbolic ABI name is asserted.
        callback_value: u16,
    },
    SuppressedNoTarget,
    TargetUnknown,
}

/// Inputs to opening one of the two observed MIDI-port selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiOutputOpenRequest {
    /// Raw selector. Only values 0 and 1 are covered by the observed path.
    pub selector: u16,
    pub device_context: DeviceContextState,
    pub lower_audio: LowerAudioState,
    pub allocation: PortStateAllocation,
    pub initialization: PortInitialization,
    pub callback: CallbackInfo,
}

/// Result of a MIDI-port open attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiOutputOpenReply {
    Opened {
        callback: OpenCallback,
    },
    Failed(u16),
    /// Available inputs do not establish one outcome or its effect on port state.
    Indeterminate,
}

impl MidiOutputOpenReply {
    pub const fn status_code(self) -> Option<u16> {
        match self {
            Self::Opened { .. } => Some(MIDI_OUTPUT_STATUS_SUCCESS),
            Self::Failed(status) => Some(status),
            Self::Indeterminate => None,
        }
    }
}

/// Current status-byte state for one open port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusByteState {
    /// The retained state is zero, so no status byte is active.
    Empty,
    /// A status byte in 0x80..=0xEF is retained.
    Known(u8),
    /// The contract or caller input does not establish the retained status byte.
    Unknown,
}

/// Caller-supplied current active-call count for the selected port.
///
/// The caller reports the per-port counter before this send enters its guard. A count of zero
/// admits the call. Any nonzero count means the new invocation would raise it above one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentActiveCallCount {
    Known(u32),
    Unknown,
}

/// What the trace establishes about expected data-byte count for the supplied byte sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpectedDataCount {
    /// No data byte reached the count lookup, or parsing did not begin.
    NotObserved,
    /// A data byte was seen while no status was retained; the expected count is zero.
    Zero,
    /// A data byte was seen with retained status, but the status-to-count table is unresolved.
    Unresolved,
}

/// Inputs for one short-message submission.
#[derive(Debug, PartialEq, Eq)]
pub struct ShortMessageRequest<'a, E> {
    /// Raw selector. Only values 0 and 1 are covered by the observed path.
    pub selector: u16,
    pub lower_audio: LowerAudioState,
    pub active_call_count: CurrentActiveCallCount,
    /// Caller-supplied byte sequence in the order to model. This is not the packed Win16 layout.
    pub message_bytes: &'a [u8],
    /// Caller-created opaque lower-service event. Its format is not modeled.
    pub lower_service_event: E,
}

/// Result of one short-message submission.
#[derive(Debug, PartialEq, Eq)]
pub enum ShortMessageReply<E> {
    /// One lower-service event is ready for the caller to submit.
    Submitted {
        status: u16,
        status_before: StatusByteState,
        status_after: StatusByteState,
        expected_data_count: ExpectedDataCount,
        lower_service_event: E,
    },
    Failed(u16),
    /// Available inputs do not establish a submission outcome; no lower-service event is returned.
    Indeterminate {
        expected_data_count: ExpectedDataCount,
    },
}

impl<E> ShortMessageReply<E> {
    pub const fn status_code(&self) -> Option<u16> {
        match self {
            Self::Submitted { status, .. } | Self::Failed(status) => Some(*status),
            Self::Indeterminate { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActivePort {
    selector: u16,
    callback: CallbackInfo,
    status_byte: StatusByteState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PortContext {
    Closed,
    Active(ActivePort),
    Unknown,
}

/// Logical state for the observed Win16 SB16 MIDI-output port paths.
///
/// This type tracks separate state for selectors 0 and 1. It leaves duplicate-open behavior
/// unresolved and does not expose per-message callbacks, because none are visible on the observed
/// short-message path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Win16MidiOutput {
    ports: [PortContext; MIDI_PORT_COUNT],
}

impl Default for Win16MidiOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Win16MidiOutput {
    pub const fn new() -> Self {
        Self {
            ports: [PortContext::Closed; MIDI_PORT_COUNT],
        }
    }

    /// Attempt to open a MIDI port using caller-supplied lower-service outcomes.
    pub fn open(&mut self, request: MidiOutputOpenRequest) -> MidiOutputOpenReply {
        let Some(index) = port_index(request.selector) else {
            return MidiOutputOpenReply::Failed(MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT);
        };

        match request.device_context {
            DeviceContextState::Invalid => {
                return MidiOutputOpenReply::Failed(MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT)
            }
            DeviceContextState::Unknown => {
                self.ports[index] = PortContext::Unknown;
                return MidiOutputOpenReply::Indeterminate;
            }
            DeviceContextState::Valid => {}
        }

        if matches!(self.ports[index], PortContext::Active(_)) {
            // Duplicate-open handling and its effects are not established.
            return MidiOutputOpenReply::Indeterminate;
        }

        match request.lower_audio {
            LowerAudioState::Unavailable => {
                return MidiOutputOpenReply::Failed(MIDI_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE)
            }
            LowerAudioState::Unknown => {
                self.ports[index] = PortContext::Unknown;
                return MidiOutputOpenReply::Indeterminate;
            }
            LowerAudioState::Available => {}
        }

        match request.allocation {
            PortStateAllocation::Failed => {
                return MidiOutputOpenReply::Failed(MIDI_OUTPUT_STATUS_STATE_ALLOCATION_FAILED)
            }
            PortStateAllocation::Unknown => {
                self.ports[index] = PortContext::Unknown;
                return MidiOutputOpenReply::Indeterminate;
            }
            PortStateAllocation::Allocated => {}
        }

        let active = ActivePort {
            selector: request.selector,
            callback: request.callback,
            status_byte: StatusByteState::Empty,
        };

        match request.initialization {
            PortInitialization::Initialized => {
                self.ports[index] = PortContext::Active(active);
                MidiOutputOpenReply::Opened {
                    callback: open_callback(request.callback.target),
                }
            }
            PortInitialization::Failed(retention) => {
                self.ports[index] = match retention {
                    FailedInitializationContext::Retained => PortContext::Active(active),
                    FailedInitializationContext::Released => PortContext::Closed,
                    FailedInitializationContext::Unknown => PortContext::Unknown,
                };
                MidiOutputOpenReply::Failed(MIDI_OUTPUT_STATUS_INITIALIZATION_FAILED)
            }
            PortInitialization::Unknown => {
                self.ports[index] = PortContext::Unknown;
                MidiOutputOpenReply::Indeterminate
            }
        }
    }

    /// Submit one caller-interpreted short message and return its opaque lower-service event.
    pub fn submit_short_message<E>(
        &mut self,
        request: ShortMessageRequest<'_, E>,
    ) -> ShortMessageReply<E> {
        let Some(index) = port_index(request.selector) else {
            return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT);
        };

        let active = match self.ports[index] {
            PortContext::Closed => {
                return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT)
            }
            PortContext::Unknown => {
                return ShortMessageReply::Indeterminate {
                    expected_data_count: ExpectedDataCount::NotObserved,
                }
            }
            PortContext::Active(active) => active,
        };

        match request.lower_audio {
            LowerAudioState::Unavailable => {
                return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE)
            }
            LowerAudioState::Unknown => {
                return ShortMessageReply::Indeterminate {
                    expected_data_count: ExpectedDataCount::NotObserved,
                }
            }
            LowerAudioState::Available => {}
        }

        match request.active_call_count {
            CurrentActiveCallCount::Known(1..) => {
                return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_REENTRANCY_REJECTED)
            }
            CurrentActiveCallCount::Unknown => {
                return ShortMessageReply::Indeterminate {
                    expected_data_count: ExpectedDataCount::NotObserved,
                }
            }
            CurrentActiveCallCount::Known(0) => {}
        }

        let status_before = active.status_byte;
        let (status_after, expected_data_count) =
            apply_message_bytes(status_before, request.message_bytes);
        self.ports[index] = PortContext::Active(ActivePort {
            status_byte: status_after,
            ..active
        });

        if request.message_bytes.is_empty() || expected_data_count != ExpectedDataCount::NotObserved
        {
            return ShortMessageReply::Indeterminate {
                expected_data_count,
            };
        }

        ShortMessageReply::Submitted {
            status: MIDI_OUTPUT_STATUS_SUCCESS,
            status_before,
            status_after,
            expected_data_count,
            lower_service_event: request.lower_service_event,
        }
    }

    /// Return the retained status-byte state for an active or unresolved selector.
    pub fn status_byte_state(&self, selector: u16) -> Option<StatusByteState> {
        let index = port_index(selector)?;
        match self.ports[index] {
            PortContext::Closed => None,
            PortContext::Active(active) => Some(active.status_byte),
            PortContext::Unknown => Some(StatusByteState::Unknown),
        }
    }

    /// Return the opaque callback information retained for a known active port.
    pub fn callback_info(&self, selector: u16) -> Option<CallbackInfo> {
        let index = port_index(selector)?;
        match self.ports[index] {
            PortContext::Active(active) => Some(active.callback),
            PortContext::Closed | PortContext::Unknown => None,
        }
    }

    /// Return the selector retained in a known active context.
    pub fn active_selector(&self, selector: u16) -> Option<u16> {
        let index = port_index(selector)?;
        match self.ports[index] {
            PortContext::Active(active) => Some(active.selector),
            PortContext::Closed | PortContext::Unknown => None,
        }
    }
}

fn port_index(selector: u16) -> Option<usize> {
    match selector {
        0 => Some(0),
        1 => Some(1),
        _ => None,
    }
}

fn open_callback(target: CallbackTargetState) -> OpenCallback {
    match target {
        CallbackTargetState::Unknown => OpenCallback::TargetUnknown,
        CallbackTargetState::Absent => OpenCallback::SuppressedNoTarget,
        CallbackTargetState::Present(target_id) => OpenCallback::Sent {
            target_id,
            callback_value: MIDI_OUTPUT_CALLBACK_VALUE_OPEN,
        },
    }
}

fn apply_message_bytes(
    current: StatusByteState,
    message_bytes: &[u8],
) -> (StatusByteState, ExpectedDataCount) {
    let mut status = current;
    let mut expected_data_count = ExpectedDataCount::NotObserved;

    for &byte in message_bytes {
        match byte {
            0x80..=0xEF => status = StatusByteState::Known(byte),
            0xF0..=0xF7 => status = StatusByteState::Empty,
            0xF8..=0xFF => {}
            0x00..=0x7F => {
                let observation = match status {
                    StatusByteState::Empty => ExpectedDataCount::Zero,
                    StatusByteState::Known(_) | StatusByteState::Unknown => {
                        ExpectedDataCount::Unresolved
                    }
                };
                if observation == ExpectedDataCount::Unresolved {
                    expected_data_count = observation;
                } else if expected_data_count == ExpectedDataCount::NotObserved {
                    expected_data_count = observation;
                }
            }
        }
    }

    (status, expected_data_count)
}
