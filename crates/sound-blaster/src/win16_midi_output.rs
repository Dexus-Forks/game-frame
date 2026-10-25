//! Logical model of the bounded Win16 SB16 MIDI-output open and short-message paths.
//!
//! This module does not implement the Win16 ABI, decode packed short-message fields, or model
//! the lower service or physical MIDI output. Callers supply lower-service availability,
//! initialization and allocation outcomes, the reentrancy decision, status-byte effects, and an
//! opaque lower-service event. The accepted reentrancy depth, system-message parsing, callback
//! timing, long messages, close/reset behavior, and hardware output remain unknown.

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
    /// No status byte is currently retained.
    Empty,
    /// A caller-classified status byte is retained.
    Known(u8),
    /// The contract or caller input does not establish the retained status byte.
    Unknown,
}

/// Caller-supplied interpretation of the status-byte effect of a short message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortMessageStatusEffect {
    /// A caller-classified channel status byte replaces the retained status byte.
    ChannelStatus(u8),
    /// A system status byte's class-specific effect, as resolved by the caller.
    SystemStatus {
        status_byte: u8,
        effect: SystemStatusEffect,
    },
    /// The message is data-only, so it uses the retained state without changing it.
    DataOnly,
    /// Parsing or the effect on retained state is not known.
    Unknown,
}

/// Caller-supplied effect for a system status byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemStatusEffect {
    Clear,
    Replace(u8),
    Unknown,
}

/// Result of the observed reentrancy counter check.
///
/// The accepted depth is unknown, so the caller supplies the result of the guard check instead
/// of this model choosing a numeric threshold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReentrancyDecision {
    Admitted,
    Rejected,
    Unknown,
}

/// Inputs for one short-message submission.
#[derive(Debug, PartialEq, Eq)]
pub struct ShortMessageRequest<E> {
    /// Raw selector. Only values 0 and 1 are covered by the observed path.
    pub selector: u16,
    pub lower_audio: LowerAudioState,
    pub reentrancy: ReentrancyDecision,
    pub status_effect: ShortMessageStatusEffect,
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
        lower_service_event: E,
    },
    Failed(u16),
    /// Available inputs do not establish one outcome.
    Indeterminate,
}

impl<E> ShortMessageReply<E> {
    pub const fn status_code(&self) -> Option<u16> {
        match self {
            Self::Submitted { status, .. } | Self::Failed(status) => Some(*status),
            Self::Indeterminate => None,
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
        request: ShortMessageRequest<E>,
    ) -> ShortMessageReply<E> {
        let Some(index) = port_index(request.selector) else {
            return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT);
        };

        let active = match self.ports[index] {
            PortContext::Closed => {
                return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT)
            }
            PortContext::Unknown => return ShortMessageReply::Indeterminate,
            PortContext::Active(active) => active,
        };

        match request.lower_audio {
            LowerAudioState::Unavailable => {
                return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE)
            }
            LowerAudioState::Unknown => return ShortMessageReply::Indeterminate,
            LowerAudioState::Available => {}
        }

        match request.reentrancy {
            ReentrancyDecision::Rejected => {
                return ShortMessageReply::Failed(MIDI_OUTPUT_STATUS_REENTRANCY_REJECTED)
            }
            ReentrancyDecision::Unknown => return ShortMessageReply::Indeterminate,
            ReentrancyDecision::Admitted => {}
        }

        let status_before = active.status_byte;
        let status_after = apply_status_effect(status_before, request.status_effect);
        self.ports[index] = PortContext::Active(ActivePort {
            status_byte: status_after,
            ..active
        });

        ShortMessageReply::Submitted {
            status: MIDI_OUTPUT_STATUS_SUCCESS,
            status_before,
            status_after,
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

fn apply_status_effect(
    current: StatusByteState,
    effect: ShortMessageStatusEffect,
) -> StatusByteState {
    match effect {
        ShortMessageStatusEffect::ChannelStatus(status_byte) => StatusByteState::Known(status_byte),
        ShortMessageStatusEffect::SystemStatus {
            status_byte: _,
            effect,
        } => match effect {
            SystemStatusEffect::Clear => StatusByteState::Empty,
            SystemStatusEffect::Replace(replacement) => StatusByteState::Known(replacement),
            SystemStatusEffect::Unknown => StatusByteState::Unknown,
        },
        ShortMessageStatusEffect::DataOnly => current,
        ShortMessageStatusEffect::Unknown => StatusByteState::Unknown,
    }
}
