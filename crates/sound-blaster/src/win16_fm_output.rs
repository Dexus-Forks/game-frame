//! Logical model of the Win16 FM synth driver's open and packed short-message paths.
//!
//! The model returns the wrapper's transformed packed value. It does not interpret note data,
//! create a lower-service wire event, or model OPL conversion, voices, registers, timing, or sound.

pub const WIN16_FM_OUTPUT_CALLBACK_VALUE_OPEN: u16 = 0x03C7;
pub const WIN16_FM_OUTPUT_STATUS_SUCCESS: u16 = 0;
pub const WIN16_FM_OUTPUT_STATUS_INVALID_SELECTOR_OR_CONTEXT: u16 = 2;
pub const WIN16_FM_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE: u16 = 3;
pub const WIN16_FM_OUTPUT_STATUS_INITIALIZATION_FAILED_OR_ALREADY_OPEN: u16 = 4;
pub const WIN16_FM_OUTPUT_STATUS_STATE_ALLOCATION_FAILED: u16 = 7;
pub const WIN16_FM_OUTPUT_STATUS_REENTRANCY_REJECTED: u16 = 0x43;

/// The driver-global retained status byte. Its initial value is not established by the contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedStatusState {
    Unknown,
    Known(u8),
}

/// Caller-supplied outcome for the open path after selector, context, and service checks pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenAttempt {
    Ready,
    StateAllocationFailed,
    InitializationFailed,
    AlreadyOpen,
}

/// Inputs that the caller supplies for one open attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenRequest {
    pub selector: u16,
    pub device_context_present: bool,
    pub lower_service_available: bool,
    pub attempt: OpenAttempt,
    pub callback_configured: bool,
}

/// Result of one logical open attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenReply {
    pub status: u16,
    /// The raw callback value sent after a successful open, when a callback is configured.
    pub callback_value: Option<u16>,
    /// True when this attempt reached the path that marks its instance open.
    pub instance_marked_open: bool,
}

/// Caller-supplied inputs for one short-message call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortMessageRequest {
    pub selector: u16,
    pub device_context_present: bool,
    pub lower_service_available: bool,
    /// Current dispatch depth for this selector. Any nonzero value is treated as nested.
    pub current_dispatch_depth: u16,
    pub packed_message: u32,
}

/// Wrapper-level transformation of a packed short-message value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackedMessageOutput {
    /// The low byte contained a status byte, so the packed value passes through unchanged.
    ExplicitStatus {
        packed_message: u32,
        status_byte: u8,
    },
    /// The input was shifted and the retained global status byte was inserted in its low byte.
    DataOnly {
        shifted_message: u32,
        inserted_status: RetainedStatusState,
    },
}

impl PackedMessageOutput {
    /// Return the fully known transformed packed value, if its inserted status byte is known.
    pub const fn packed_value(self) -> Option<u32> {
        match self {
            Self::ExplicitStatus { packed_message, .. } => Some(packed_message),
            Self::DataOnly {
                shifted_message,
                inserted_status: RetainedStatusState::Known(status),
            } => Some(shifted_message | status as u32),
            Self::DataOnly {
                inserted_status: RetainedStatusState::Unknown,
                ..
            } => None,
        }
    }
}

/// Result of one short-message call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortMessageReply {
    pub status: u16,
    /// Present only when validation and the dispatch-depth guard permit transformation.
    pub output: Option<PackedMessageOutput>,
}

/// State shared by both accepted selectors in the Win16 FM output wrapper model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Win16FmOutput {
    retained_status: RetainedStatusState,
}

impl Default for Win16FmOutput {
    fn default() -> Self {
        Self::new(RetainedStatusState::Unknown)
    }
}

impl Win16FmOutput {
    /// Start with an explicit retained-status state. The observed initial byte is unknown.
    pub const fn new(retained_status: RetainedStatusState) -> Self {
        Self { retained_status }
    }

    pub const fn retained_status(&self) -> RetainedStatusState {
        self.retained_status
    }

    /// Resolve an open attempt from explicit caller-supplied outcomes.
    ///
    /// Opening does not reset the driver-global retained status. `OpenAttempt` represents one
    /// terminal outcome, so callers do not need to guess precedence between simultaneous failures.
    pub fn open(&self, request: OpenRequest) -> OpenReply {
        if !valid_selector(request.selector) || !request.device_context_present {
            return OpenReply {
                status: WIN16_FM_OUTPUT_STATUS_INVALID_SELECTOR_OR_CONTEXT,
                callback_value: None,
                instance_marked_open: false,
            };
        }
        if !request.lower_service_available {
            return OpenReply {
                status: WIN16_FM_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE,
                callback_value: None,
                instance_marked_open: false,
            };
        }

        let status = match request.attempt {
            OpenAttempt::Ready => WIN16_FM_OUTPUT_STATUS_SUCCESS,
            OpenAttempt::StateAllocationFailed => WIN16_FM_OUTPUT_STATUS_STATE_ALLOCATION_FAILED,
            OpenAttempt::InitializationFailed | OpenAttempt::AlreadyOpen => {
                WIN16_FM_OUTPUT_STATUS_INITIALIZATION_FAILED_OR_ALREADY_OPEN
            }
        };
        let succeeded = status == WIN16_FM_OUTPUT_STATUS_SUCCESS;
        OpenReply {
            status,
            callback_value: if succeeded && request.callback_configured {
                Some(WIN16_FM_OUTPUT_CALLBACK_VALUE_OPEN)
            } else {
                None
            },
            instance_marked_open: succeeded,
        }
    }

    /// Validate, apply the one-active-call guard, then transform the packed wrapper argument.
    pub fn short_message(&mut self, request: ShortMessageRequest) -> ShortMessageReply {
        if !valid_selector(request.selector) || !request.device_context_present {
            return ShortMessageReply {
                status: WIN16_FM_OUTPUT_STATUS_INVALID_SELECTOR_OR_CONTEXT,
                output: None,
            };
        }
        if !request.lower_service_available {
            return ShortMessageReply {
                status: WIN16_FM_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE,
                output: None,
            };
        }
        if request.current_dispatch_depth != 0 {
            return ShortMessageReply {
                status: WIN16_FM_OUTPUT_STATUS_REENTRANCY_REJECTED,
                output: None,
            };
        }

        let status_byte = request.packed_message as u8;
        let output = if status_byte & 0x80 != 0 {
            self.retained_status = RetainedStatusState::Known(status_byte);
            PackedMessageOutput::ExplicitStatus {
                packed_message: request.packed_message,
                status_byte,
            }
        } else {
            PackedMessageOutput::DataOnly {
                shifted_message: request.packed_message.wrapping_shl(8),
                inserted_status: self.retained_status,
            }
        };

        ShortMessageReply {
            status: WIN16_FM_OUTPUT_STATUS_SUCCESS,
            output: Some(output),
        }
    }
}

const fn valid_selector(selector: u16) -> bool {
    selector <= 1
}
