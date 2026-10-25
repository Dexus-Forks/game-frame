//! Logical model of the generic Win16 SB16 mixer control-details read and write paths.
//!
//! Callers supply descriptor metadata, current per-control values, dispatcher context, and
//! configured mixer clients. Control identifiers, categories, shapes, value limits, and apply
//! actions are opaque caller inputs. This module does not identify a master-volume control,
//! implement the Win16 ABI, choose channel ordering, clamp values, or model audible gain or the
//! lower audio service. Ambiguous validation outcomes are returned without changing state.

use std::collections::BTreeSet;

pub const MIXER_DETAILS_CALLBACK_VALUE: u16 = 0x03D1;

pub const MIXER_DETAILS_STATUS_SUCCESS: u16 = 0;
pub const MIXER_DETAILS_STATUS_DEVICE_CONTEXT_UNRESOLVED: u16 = 2;
pub const MIXER_DETAILS_STATUS_LOWER_AUDIO_UNAVAILABLE: u16 = 3;
pub const MIXER_DETAILS_STATUS_INVALID_DETAILS_CONTEXT: u16 = 5;
pub const MIXER_DETAILS_STATUS_UNSUPPORTED_OPERATION: u16 = 8;
pub const MIXER_DETAILS_STATUS_INVALID_SHAPE_OR_CATEGORY: u16 = 0x0400;
pub const MIXER_DETAILS_STATUS_INVALID_CONTROL_ID: u16 = 0x0401;
pub const MIXER_DETAILS_STATUS_DESCRIPTOR_MISMATCH: u16 = 0x0402;
/// Observed on one details-shape failure path; its public meaning is unresolved.
pub const MIXER_DETAILS_STATUS_UNMAPPED_0B: u16 = 0x000B;

/// Whether the dispatcher resolved the selected device context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceContextState {
    Resolved,
    Unresolved,
    Unknown,
}

/// Whether lower audio state is available to the mixer path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerAudioState {
    Available,
    Unavailable,
    Unknown,
}

/// Whether the details helper received a valid context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailsContextState {
    Valid,
    Invalid,
    Unknown,
}

/// Context facts supplied for one logical mixer request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixerDetailsContext {
    /// Raw selector. Only 0 and 1 are covered by the observed paths.
    pub selector: u16,
    pub device_context: DeviceContextState,
    pub lower_audio: LowerAudioState,
    pub details_context: DetailsContextState,
}

/// Operation requested from the generic details path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixerDetailsOperation {
    Read,
    Write,
    /// A known mode or operation outside the modeled read and write paths.
    Unsupported,
    /// The caller cannot establish which operation was requested.
    Unknown,
}

/// Descriptor-supplied request and storage shape.
///
/// `value_count` is the expected count of logical unsigned values. It is not derived from the
/// channel or item counts, and it makes no claim about channel ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixerControlDetailsShape {
    pub channels: u32,
    pub items: u32,
    pub detail_size: u32,
    pub value_count: usize,
}

/// Whether caller-supplied descriptor metadata supports the modeled details operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixerDetailsCategory {
    Supported,
    Unsupported,
}

/// Opaque identity for the apply routine selected by a control descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorApplyAction(pub u64);

/// Caller-supplied metadata for one control in one selector's descriptor table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixerControlDescriptor {
    pub control_id: u32,
    pub category: MixerDetailsCategory,
    pub shape: MixerControlDetailsShape,
    /// Inclusive numeric limits supplied with this descriptor.
    pub minimum: u32,
    pub maximum: u32,
    pub apply_action: DescriptorApplyAction,
}

/// Current logical values for one descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MixerControlState {
    /// Raw selector whose descriptor table contains this record.
    pub selector: u16,
    pub descriptor: MixerControlDescriptor,
    /// Stored values in caller-defined order. This is not a channel-order claim.
    pub current_values: Vec<u32>,
}

impl MixerControlState {
    pub fn new(
        selector: u16,
        descriptor: MixerControlDescriptor,
        current_values: impl Into<Vec<u32>>,
    ) -> Self {
        Self {
            selector,
            descriptor,
            current_values: current_values.into(),
        }
    }
}

/// Opaque logical identity for one configured mixer client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MixerClientId(pub u64);

/// One notification with the raw callback value observed after a successful write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixerClientNotification {
    pub client: MixerClientId,
    /// Raw observed callback value; no Win16 callback ABI is modeled.
    pub callback_value: u16,
}

/// Inputs to one logical control-details read or write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixerDetailsRequest<'a> {
    pub context: MixerDetailsContext,
    pub operation: MixerDetailsOperation,
    pub control_id: u32,
    pub channels: u32,
    pub items: u32,
    pub detail_size: u32,
    /// For writes, values to validate and store. For reads, only the slice length is used as the
    /// logical caller-buffer capacity; its contents are ignored.
    pub details: &'a [u32],
}

/// Result of a modeled read, write, failure, or ambiguous request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MixerDetailsReply {
    Read {
        /// Values copied from the selected control's stored state.
        details: Vec<u32>,
    },
    Written {
        /// Opaque descriptor-selected action for the caller to dispatch.
        apply_action: DescriptorApplyAction,
        /// Notifications for all configured clients, in the order supplied to the model.
        notifications: Vec<MixerClientNotification>,
    },
    Failed {
        status: u16,
    },
    /// Several outcomes remain possible. No read or write effects are performed by this model.
    /// `failure_statuses` contains possible non-success statuses; success is reported separately.
    Indeterminate {
        failure_statuses: Vec<u16>,
        success_possible: bool,
    },
}

impl MixerDetailsReply {
    /// Return the status only when the result has one definite status.
    pub const fn status_code(&self) -> Option<u16> {
        match self {
            Self::Read { .. } | Self::Written { .. } => Some(MIXER_DETAILS_STATUS_SUCCESS),
            Self::Failed { status } => Some(*status),
            Self::Indeterminate { .. } => None,
        }
    }
}

/// State for the bounded generic mixer-details paths.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Win16MixerDetails {
    controls: Vec<MixerControlState>,
    clients: Vec<MixerClientId>,
}

impl Win16MixerDetails {
    pub fn new(
        controls: impl Into<Vec<MixerControlState>>,
        clients: impl Into<Vec<MixerClientId>>,
    ) -> Self {
        Self {
            controls: controls.into(),
            clients: clients.into(),
        }
    }

    /// Read stored logical details or write values through the generic details path.
    ///
    /// The contract does not establish a universal validation precedence. Independent checks are
    /// therefore combined into a set of possible statuses. A write mutates state and emits its
    /// apply action and client notifications only after every required check is definite and
    /// successful. Unknown context, duplicate descriptor IDs, or overlapping failures return an
    /// indeterminate result without applying effects.
    pub fn control_details(&mut self, request: MixerDetailsRequest<'_>) -> MixerDetailsReply {
        let mut outcome = ValidationOutcome::default();

        validate_context(request.context, &mut outcome);
        match request.operation {
            MixerDetailsOperation::Unsupported => {
                outcome.definite_failure(MIXER_DETAILS_STATUS_UNSUPPORTED_OPERATION);
            }
            MixerDetailsOperation::Unknown => {
                outcome.possible_failure(MIXER_DETAILS_STATUS_UNSUPPORTED_OPERATION);
                outcome.success_possible = true;
            }
            MixerDetailsOperation::Read | MixerDetailsOperation::Write => {
                self.validate_details_request(request, &mut outcome);
            }
        }

        if let Some(reply) = outcome.into_reply_if_unresolved() {
            return reply;
        }

        // The only remaining outcomes are a definite, fully validated read or write.
        let Some(index) = self.unique_control_index(request.context.selector, request.control_id)
        else {
            // Descriptor lookup was already validated. Keep this defensive branch explicit if
            // caller-owned state is changed concurrently through a future API.
            return MixerDetailsReply::Indeterminate {
                failure_statuses: vec![MIXER_DETAILS_STATUS_INVALID_CONTROL_ID],
                success_possible: false,
            };
        };

        let record = &mut self.controls[index];
        match request.operation {
            MixerDetailsOperation::Read => MixerDetailsReply::Read {
                details: record.current_values.clone(),
            },
            MixerDetailsOperation::Write => {
                record.current_values.clear();
                record.current_values.extend_from_slice(request.details);
                let apply_action = record.descriptor.apply_action;
                let notifications = self
                    .clients
                    .iter()
                    .copied()
                    .map(|client| MixerClientNotification {
                        client,
                        callback_value: MIXER_DETAILS_CALLBACK_VALUE,
                    })
                    .collect();
                MixerDetailsReply::Written {
                    apply_action,
                    notifications,
                }
            }
            MixerDetailsOperation::Unsupported | MixerDetailsOperation::Unknown => {
                unreachable!("non-read/write operations return before applying details")
            }
        }
    }

    /// Inspect the model's logical values without exposing descriptor internals through a
    /// Win16-shaped interface.
    pub fn stored_values(&self, selector: u16, control_id: u32) -> Option<&[u32]> {
        let index = self.unique_control_index(selector, control_id)?;
        Some(&self.controls[index].current_values)
    }

    fn validate_details_request(
        &self,
        request: MixerDetailsRequest<'_>,
        outcome: &mut ValidationOutcome,
    ) {
        if !matches!(request.context.selector, 0 | 1) {
            outcome.definite_failure(MIXER_DETAILS_STATUS_DEVICE_CONTEXT_UNRESOLVED);
            return;
        }

        let matches: Vec<usize> = self
            .controls
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                (state.selector == request.context.selector
                    && state.descriptor.control_id == request.control_id)
                    .then_some(index)
            })
            .collect();

        let index = match matches.as_slice() {
            [] => {
                outcome.definite_failure(MIXER_DETAILS_STATUS_INVALID_CONTROL_ID);
                return;
            }
            [index] => *index,
            _ => {
                // The behavior of a malformed descriptor table with duplicate IDs is unknown.
                outcome.possible_failure(MIXER_DETAILS_STATUS_INVALID_CONTROL_ID);
                outcome.success_possible = true;
                return;
            }
        };

        let descriptor = &self.controls[index].descriptor;
        if descriptor.category == MixerDetailsCategory::Unsupported {
            outcome.definite_failure(MIXER_DETAILS_STATUS_INVALID_SHAPE_OR_CATEGORY);
        }

        let shape_matches = request.channels == descriptor.shape.channels
            && request.items == descriptor.shape.items
            && request.detail_size == descriptor.shape.detail_size
            && request.details.len() == descriptor.shape.value_count;

        if !shape_matches {
            // The contract records several failure values for details-shape failures and does
            // not establish which one wins for each mismatched field.
            outcome.possible_failure(MIXER_DETAILS_STATUS_INVALID_SHAPE_OR_CATEGORY);
            outcome.possible_failure(MIXER_DETAILS_STATUS_DESCRIPTOR_MISMATCH);
            outcome.possible_failure(MIXER_DETAILS_STATUS_UNMAPPED_0B);
            outcome.success_possible = false;
            return;
        }

        if request.operation == MixerDetailsOperation::Write
            && request
                .details
                .iter()
                .any(|value| *value < descriptor.minimum || *value > descriptor.maximum)
        {
            outcome.definite_failure(MIXER_DETAILS_STATUS_DESCRIPTOR_MISMATCH);
        }
    }

    fn unique_control_index(&self, selector: u16, control_id: u32) -> Option<usize> {
        let mut matches = self
            .controls
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                (state.selector == selector && state.descriptor.control_id == control_id)
                    .then_some(index)
            });
        let index = matches.next()?;
        matches.next().is_none().then_some(index)
    }
}

fn validate_context(context: MixerDetailsContext, outcome: &mut ValidationOutcome) {
    if !matches!(context.selector, 0 | 1) {
        outcome.definite_failure(MIXER_DETAILS_STATUS_DEVICE_CONTEXT_UNRESOLVED);
    }

    match context.device_context {
        DeviceContextState::Resolved => {}
        DeviceContextState::Unresolved => {
            outcome.definite_failure(MIXER_DETAILS_STATUS_DEVICE_CONTEXT_UNRESOLVED)
        }
        DeviceContextState::Unknown => {
            outcome.possible_failure(MIXER_DETAILS_STATUS_DEVICE_CONTEXT_UNRESOLVED);
            outcome.success_possible = true;
        }
    }

    match context.lower_audio {
        LowerAudioState::Available => {}
        LowerAudioState::Unavailable => {
            outcome.definite_failure(MIXER_DETAILS_STATUS_LOWER_AUDIO_UNAVAILABLE)
        }
        LowerAudioState::Unknown => {
            outcome.possible_failure(MIXER_DETAILS_STATUS_LOWER_AUDIO_UNAVAILABLE);
            outcome.success_possible = true;
        }
    }

    match context.details_context {
        DetailsContextState::Valid => {}
        DetailsContextState::Invalid => {
            outcome.definite_failure(MIXER_DETAILS_STATUS_INVALID_DETAILS_CONTEXT)
        }
        DetailsContextState::Unknown => {
            outcome.possible_failure(MIXER_DETAILS_STATUS_INVALID_DETAILS_CONTEXT);
            outcome.success_possible = true;
        }
    }
}

#[derive(Default)]
struct ValidationOutcome {
    failure_statuses: BTreeSet<u16>,
    success_possible: bool,
}

impl ValidationOutcome {
    fn definite_failure(&mut self, status: u16) {
        self.failure_statuses.insert(status);
        self.success_possible = false;
    }

    fn possible_failure(&mut self, status: u16) {
        self.failure_statuses.insert(status);
    }

    fn into_reply_if_unresolved(self) -> Option<MixerDetailsReply> {
        if !self.failure_statuses.is_empty() {
            if !self.success_possible && self.failure_statuses.len() == 1 {
                return Some(MixerDetailsReply::Failed {
                    status: *self.failure_statuses.iter().next().unwrap(),
                });
            }
            return Some(MixerDetailsReply::Indeterminate {
                failure_statuses: self.failure_statuses.into_iter().collect(),
                success_possible: self.success_possible,
            });
        }

        None
    }
}
