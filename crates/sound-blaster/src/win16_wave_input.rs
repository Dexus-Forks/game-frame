//! Logical model of the bounded Win16 SB16 wave-input paths.
//!
//! Callers supply lower-service outcomes and capture byte counts. This module does not implement
//! the Win16 ABI, parse complete structures, or model captured samples or hardware. It models one
//! stream's queue at a time. The PCM matrix, malformed formats, statuses 4 and 7 causes, capture
//! start and hardware behavior, callback timing, stream multiplicity, failure precedence,
//! overruns, and pause/restart/stop/reset effects remain unknown. Callback values are raw values.

use std::collections::VecDeque;

pub const WAVE_INPUT_OPEN_QUERY: u16 = 0x0001;
pub const WAVE_INPUT_HEADER_DONE: u16 = 0x0001;
pub const WAVE_INPUT_HEADER_PREPARED: u16 = 0x0002;
pub const WAVE_INPUT_HEADER_IN_QUEUE: u16 = 0x0010;

/// Raw callback message value observed on successful input open.
pub const WAVE_INPUT_CALLBACK_VALUE_OPEN: u16 = 0x03BE;
/// Raw callback message value observed after an input header completes.
pub const WAVE_INPUT_CALLBACK_VALUE_BUFFER_DONE: u16 = 0x03C0;

pub const WAVE_INPUT_STATUS_SUCCESS: u16 = 0;
pub const WAVE_INPUT_STATUS_STREAM_MISMATCH: u16 = 5;
pub const WAVE_INPUT_STATUS_UNSUPPORTED_FORMAT: u16 = 0x20;
pub const WAVE_INPUT_STATUS_HEADER_ALREADY_QUEUED: u16 = 0x21;
pub const WAVE_INPUT_STATUS_HEADER_NOT_PREPARED: u16 = 0x22;
pub const WAVE_INPUT_STATUS_OPEN_4: u16 = 4;
pub const WAVE_INPUT_STATUS_OPEN_7: u16 = 7;

/// Result supplied for the lower service's query-only format check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatCheckResult {
    Accepted,
    Rejected,
    Unsupported,
    Unknown,
}

/// Result supplied for the lower service's non-query open attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerOpenResult {
    Opened,
    FormatRejected,
    Unsupported,
    /// Status 4 was observed, but its public failure cause is not established.
    UnmappedStatus4,
    /// Status 7 was observed, but its public failure cause is not established.
    UnmappedStatus7,
    Unknown,
}

/// Callback-target knowledge without modeling a Win16 callback ABI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackTargetState {
    Unknown,
    Absent,
    /// Opaque logical target identity, not a Win16 callback handle or pointer.
    Present(u64),
}

/// Inputs to the logical open operation.
///
/// The format tag is the only format field interpreted here. Tag 1 is the only confirmed PCM
/// family. The caller supplies the lower-service result because the accepted PCM matrix and the
/// lower-service protocol are unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveInputOpenRequest {
    pub flags: u16,
    pub format_tag: u16,
    pub query_check: FormatCheckResult,
    pub lower_open: LowerOpenResult,
    pub stream_handle: u64,
    pub callback_target: CallbackTargetState,
}

impl WaveInputOpenRequest {
    /// Build a query-only request. Stream and callback inputs are unused on this path.
    pub const fn query(format_tag: u16, check: FormatCheckResult) -> Self {
        Self {
            flags: WAVE_INPUT_OPEN_QUERY,
            format_tag,
            query_check: check,
            lower_open: LowerOpenResult::Unknown,
            stream_handle: 0,
            callback_target: CallbackTargetState::Unknown,
        }
    }

    /// Build a non-query open request from the caller-supplied lower-service result.
    pub const fn stream(
        format_tag: u16,
        lower_open: LowerOpenResult,
        stream_handle: u64,
        callback_target: CallbackTargetState,
    ) -> Self {
        Self {
            flags: 0,
            format_tag,
            query_check: FormatCheckResult::Unknown,
            lower_open,
            stream_handle,
            callback_target,
        }
    }
}

/// Definite result of a query-only open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryOpenReply {
    Accepted,
    UnsupportedFormat,
    /// The supplied evidence does not establish acceptance or rejection.
    Indeterminate,
}

impl QueryOpenReply {
    pub const fn status_code(self) -> Option<u16> {
        match self {
            Self::Accepted => Some(WAVE_INPUT_STATUS_SUCCESS),
            Self::UnsupportedFormat => Some(WAVE_INPUT_STATUS_UNSUPPORTED_FORMAT),
            Self::Indeterminate => None,
        }
    }
}

/// Open failures retained without guessing their public causes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveInputOpenFailure {
    UnmappedStatus4,
    UnmappedStatus7,
    UnsupportedFormat,
}

impl WaveInputOpenFailure {
    pub const fn status_code(self) -> u16 {
        match self {
            Self::UnmappedStatus4 => WAVE_INPUT_STATUS_OPEN_4,
            Self::UnmappedStatus7 => WAVE_INPUT_STATUS_OPEN_7,
            Self::UnsupportedFormat => WAVE_INPUT_STATUS_UNSUPPORTED_FORMAT,
        }
    }
}

/// Result of a query or non-query open request.
#[derive(Debug, PartialEq, Eq)]
pub enum WaveInputOpenReply {
    Query(QueryOpenReply),
    Opened {
        stream: WaveInputStream,
        callback: CallbackDelivery,
    },
    Failed(WaveInputOpenFailure),
    /// Available inputs do not establish one result.
    Indeterminate,
}

impl WaveInputOpenReply {
    /// Return a status only when the result has one definite public status.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Query(reply) => reply.status_code(),
            Self::Opened { .. } => Some(WAVE_INPUT_STATUS_SUCCESS),
            Self::Failed(failure) => Some(failure.status_code()),
            Self::Indeterminate => None,
        }
    }
}

/// Evaluate the bounded wave-input open behavior.
///
/// Query-only mode is selected by bit 0 of `flags` and never creates stream state. For a
/// non-query open, only tag 1 is confirmed as PCM. A non-PCM tag paired with lower-service success
/// remains indeterminate because rejection is only confirmed when the format helper is available.
pub fn open_wave_input(request: WaveInputOpenRequest) -> WaveInputOpenReply {
    if request.flags & WAVE_INPUT_OPEN_QUERY != 0 {
        return WaveInputOpenReply::Query(query_open(request.format_tag, request.query_check));
    }

    match request.lower_open {
        LowerOpenResult::Opened if request.format_tag == 1 => opened_reply(request),
        LowerOpenResult::Opened => WaveInputOpenReply::Indeterminate,
        LowerOpenResult::FormatRejected | LowerOpenResult::Unsupported => {
            WaveInputOpenReply::Failed(WaveInputOpenFailure::UnsupportedFormat)
        }
        LowerOpenResult::UnmappedStatus4 if request.format_tag == 1 => {
            WaveInputOpenReply::Failed(WaveInputOpenFailure::UnmappedStatus4)
        }
        LowerOpenResult::UnmappedStatus7 if request.format_tag == 1 => {
            WaveInputOpenReply::Failed(WaveInputOpenFailure::UnmappedStatus7)
        }
        LowerOpenResult::UnmappedStatus4 | LowerOpenResult::UnmappedStatus7 => {
            // The non-PCM rejection and raw open failure may overlap; precedence is unknown.
            WaveInputOpenReply::Indeterminate
        }
        LowerOpenResult::Unknown => WaveInputOpenReply::Indeterminate,
    }
}

fn opened_reply(request: WaveInputOpenRequest) -> WaveInputOpenReply {
    WaveInputOpenReply::Opened {
        stream: WaveInputStream::new(request.stream_handle, request.callback_target),
        callback: callback_for_target(
            request.callback_target,
            WAVE_INPUT_CALLBACK_VALUE_OPEN,
            None,
        ),
    }
}

fn query_open(format_tag: u16, check: FormatCheckResult) -> QueryOpenReply {
    if format_tag != 1 {
        return match check {
            FormatCheckResult::Unknown => QueryOpenReply::Indeterminate,
            FormatCheckResult::Accepted
            | FormatCheckResult::Rejected
            | FormatCheckResult::Unsupported => QueryOpenReply::UnsupportedFormat,
        };
    }
    match check {
        FormatCheckResult::Accepted => QueryOpenReply::Accepted,
        FormatCheckResult::Rejected | FormatCheckResult::Unsupported => {
            QueryOpenReply::UnsupportedFormat
        }
        FormatCheckResult::Unknown => QueryOpenReply::Indeterminate,
    }
}

/// Possible callback delivery without assigning callback timing or ABI arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackDelivery {
    Sent(WaveInputCallbackEvent),
    SuppressedNoTarget,
    TargetUnknown,
}

/// One observed raw callback value and its logical payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveInputCallbackEvent {
    pub target_id: u64,
    /// Raw observed value; no symbolic Win16 ABI name is asserted.
    pub callback_value: u16,
    pub header: Option<WaveInputHeaderInfo>,
}

fn callback_for_target(
    target: CallbackTargetState,
    callback_value: u16,
    header: Option<WaveInputHeaderInfo>,
) -> CallbackDelivery {
    match target {
        CallbackTargetState::Unknown => CallbackDelivery::TargetUnknown,
        CallbackTargetState::Absent => CallbackDelivery::SuppressedNoTarget,
        CallbackTargetState::Present(target_id) => CallbackDelivery::Sent(WaveInputCallbackEvent {
            target_id,
            callback_value,
            header,
        }),
    }
}

/// State for one successfully opened input stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveInputStream {
    handle: u64,
    callback_target: CallbackTargetState,
    queued_headers: VecDeque<WaveInputHeader>,
}

impl WaveInputStream {
    fn new(handle: u64, callback_target: CallbackTargetState) -> Self {
        Self {
            handle,
            callback_target,
            queued_headers: VecDeque::new(),
        }
    }

    pub const fn handle(&self) -> u64 {
        self.handle
    }

    /// Submit a header and append it in capture order when all observed acceptance conditions
    /// hold. A rejected or indeterminate submission returns ownership of the header to the caller.
    pub fn submit(
        &mut self,
        supplied_handle: u64,
        mut header: WaveInputHeader,
    ) -> WaveInputSubmissionReply {
        let mut failures = Vec::new();
        if supplied_handle != self.handle {
            push_status(&mut failures, WAVE_INPUT_STATUS_STREAM_MISMATCH);
        }
        if header.flags & WAVE_INPUT_HEADER_PREPARED == 0 {
            push_status(&mut failures, WAVE_INPUT_STATUS_HEADER_NOT_PREPARED);
        }
        if header.flags & WAVE_INPUT_HEADER_IN_QUEUE != 0 {
            push_status(&mut failures, WAVE_INPUT_STATUS_HEADER_ALREADY_QUEUED);
        }

        match failures.as_slice() {
            [] => {
                header.flags =
                    (header.flags | WAVE_INPUT_HEADER_IN_QUEUE) & !WAVE_INPUT_HEADER_DONE;
                header.link_fields = HeaderLinkState::ResetByAcceptedSubmission;
                self.queued_headers.push_back(header);
                WaveInputSubmissionReply::Accepted
            }
            [status] => WaveInputSubmissionReply::Failed {
                status: *status,
                header,
            },
            _ => WaveInputSubmissionReply::IndeterminateFailurePrecedence {
                statuses: failures,
                header,
            },
        }
    }

    /// Return queued header IDs in submission order.
    pub fn queued_header_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.queued_headers.iter().map(WaveInputHeader::id)
    }

    /// Inspect a queued header by its logical ID.
    pub fn queued_header(&self, id: u64) -> Option<&WaveInputHeader> {
        self.queued_headers.iter().find(|header| header.id == id)
    }

    /// Apply caller-supplied capture byte counts to queued headers.
    ///
    /// Only byte counts are modeled; this does not create sample data or model capture hardware.
    /// Filled headers are removed in submission order. Their flags and links are updated before
    /// a completion event is returned. Bytes supplied after the queue empties are reported as
    /// unassigned; the contract does not establish overrun effects.
    pub fn capture_bytes(&mut self, byte_count: u32) -> CaptureProgressReply {
        let mut remaining = byte_count;
        let mut completions = Vec::new();

        while remaining > 0 {
            let Some(header) = self.queued_headers.front_mut() else {
                break;
            };
            if header.bytes_recorded >= header.buffer_length_bytes {
                return CaptureProgressReply {
                    supplied_bytes: byte_count,
                    copied_bytes: byte_count - remaining,
                    unassigned_bytes: remaining,
                    boundary: CaptureBoundary::HeaderAlreadyFull(header.id),
                    completions,
                };
            }

            let room = header.buffer_length_bytes - header.bytes_recorded;
            let copied = remaining.min(room);
            header.bytes_recorded += copied;
            remaining -= copied;

            if header.bytes_recorded == header.buffer_length_bytes {
                let mut completed = self.queued_headers.pop_front().expect("front was present");
                completed.flags =
                    (completed.flags | WAVE_INPUT_HEADER_DONE) & !WAVE_INPUT_HEADER_IN_QUEUE;
                completed.link_fields = HeaderLinkState::ClearedByCompletion;
                let info = completed.info();
                let callback = callback_for_target(
                    self.callback_target,
                    WAVE_INPUT_CALLBACK_VALUE_BUFFER_DONE,
                    Some(info),
                );
                completions.push(WaveInputCompletion {
                    header: info,
                    callback,
                });
            } else {
                break;
            }
        }

        let copied_bytes = byte_count - remaining;
        CaptureProgressReply {
            supplied_bytes: byte_count,
            copied_bytes,
            unassigned_bytes: remaining,
            boundary: if byte_count == 0 {
                CaptureBoundary::NoBytesSupplied
            } else if remaining > 0 {
                CaptureBoundary::QueueEmpty
            } else {
                CaptureBoundary::Progressed
            },
            completions,
        }
    }

    pub const fn callback_target(&self) -> CallbackTargetState {
        self.callback_target
    }
}

fn push_status(statuses: &mut Vec<u16>, status: u16) {
    if !statuses.contains(&status) {
        statuses.push(status);
    }
}

/// Logical wave-input header information needed by the bounded queue model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveInputHeader {
    id: u64,
    buffer_length_bytes: u32,
    bytes_recorded: u32,
    flags: u16,
    link_fields: HeaderLinkState,
}

impl WaveInputHeader {
    pub const fn new(id: u64, buffer_length_bytes: u32, flags: u16) -> Self {
        Self {
            id,
            buffer_length_bytes,
            bytes_recorded: 0,
            flags,
            link_fields: HeaderLinkState::Unmodeled,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn buffer_length_bytes(&self) -> u32 {
        self.buffer_length_bytes
    }

    pub const fn bytes_recorded(&self) -> u32 {
        self.bytes_recorded
    }

    /// Supply the caller-side recorded-byte count. Accepted submission leaves it unchanged.
    pub fn set_bytes_recorded(&mut self, bytes_recorded: u32) {
        self.bytes_recorded = bytes_recorded;
    }

    pub const fn flags(&self) -> u16 {
        self.flags
    }

    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags;
    }

    pub const fn link_fields(&self) -> HeaderLinkState {
        self.link_fields
    }

    pub const fn info(&self) -> WaveInputHeaderInfo {
        WaveInputHeaderInfo {
            id: self.id,
            buffer_length_bytes: self.buffer_length_bytes,
            bytes_recorded: self.bytes_recorded,
            flags: self.flags,
            link_fields: self.link_fields,
        }
    }
}

/// Link-field effects without modeling the Win16 wave-header layout or pointer values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderLinkState {
    Unmodeled,
    ResetByAcceptedSubmission,
    ClearedByCompletion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveInputHeaderInfo {
    pub id: u64,
    pub buffer_length_bytes: u32,
    pub bytes_recorded: u32,
    pub flags: u16,
    pub link_fields: HeaderLinkState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaveInputSubmissionReply {
    Accepted,
    Failed {
        status: u16,
        header: WaveInputHeader,
    },
    /// Several rejection conditions were supplied; their status precedence is unknown.
    IndeterminateFailurePrecedence {
        statuses: Vec<u16>,
        header: WaveInputHeader,
    },
}

impl WaveInputSubmissionReply {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Accepted => Some(WAVE_INPUT_STATUS_SUCCESS),
            Self::Failed { status, .. } => Some(*status),
            Self::IndeterminateFailurePrecedence { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveInputCompletion {
    pub header: WaveInputHeaderInfo,
    pub callback: CallbackDelivery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureBoundary {
    Progressed,
    QueueEmpty,
    NoBytesSupplied,
    /// A caller-supplied header already had a full byte count when capture reached it.
    /// The contract does not establish how this prefilled state is handled.
    HeaderAlreadyFull(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureProgressReply {
    pub supplied_bytes: u32,
    pub copied_bytes: u32,
    pub unassigned_bytes: u32,
    pub boundary: CaptureBoundary,
    pub completions: Vec<WaveInputCompletion>,
}
