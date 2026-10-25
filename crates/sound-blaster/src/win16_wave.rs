//! Logical model of the bounded Win16 SB16 wave-output open and queue paths.
//!
//! This module does not implement the Win16 ABI, parse a complete wave-format structure, or call
//! the lower sound-driver service. Callers supply the service verdicts that the contract does not
//! determine. Unknown and overlapping outcomes stay explicit. Header completion and callback
//! timing, pause/restart/reset behavior, and exact lower-service dispatch timing are not modeled.

pub const WAVE_OPEN_QUERY: u16 = 0x0001;
pub const WAVE_HEADER_DONE: u16 = 0x0001;
pub const WAVE_HEADER_PREPARED: u16 = 0x0002;
pub const WAVE_HEADER_IN_QUEUE: u16 = 0x0010;

pub const WAVE_STATUS_SUCCESS: u16 = 0;
pub const WAVE_STATUS_UNSUPPORTED_FORMAT: u16 = 0x20;
pub const WAVE_STATUS_HEADER_ALREADY_QUEUED: u16 = 0x21;
pub const WAVE_STATUS_HEADER_NOT_PREPARED: u16 = 0x22;
pub const WAVE_STATUS_OPEN_UNMAPPED_3: u16 = 3;
pub const WAVE_STATUS_OPEN_CONTEXT_OR_ALLOCATION: u16 = 4;
pub const WAVE_STATUS_OPEN_UNMAPPED_7: u16 = 7;

/// Result supplied for the query-only format check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatCheckResult {
    Accepted,
    Rejected,
    Unsupported,
    Unknown,
}

/// State relevant to the non-query open path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceContextState {
    NotAlreadyOpen,
    AlreadyOpen,
    Unknown,
}

/// Result of attempting to allocate the per-stream state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamAllocationResult {
    Succeeded,
    Failed,
    Unknown,
}

/// Result supplied for the lower service's non-query open operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerOpenResult {
    Opened,
    FormatRejected,
    Unsupported,
    /// The status was observed, but its public failure cause is unmapped.
    UnmappedStatus3,
    /// The status was observed, but its public failure cause is unmapped.
    UnmappedStatus7,
    Unknown,
}

/// Inputs to the logical open operation.
///
/// `format_tag` is the only format field interpreted here. Format parameters and malformed
/// structure handling are not established by the contract. The other result fields matter only
/// on the path selected by `flags`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveOpenRequest {
    pub flags: u16,
    pub format_tag: u16,
    pub query_check: FormatCheckResult,
    pub context: DeviceContextState,
    pub allocation: StreamAllocationResult,
    pub lower_open: LowerOpenResult,
}

impl WaveOpenRequest {
    /// Build a query request. Query mode does not use context or allocation state.
    pub const fn query(format_tag: u16, check: FormatCheckResult) -> Self {
        Self {
            flags: WAVE_OPEN_QUERY,
            format_tag,
            query_check: check,
            context: DeviceContextState::Unknown,
            allocation: StreamAllocationResult::Unknown,
            lower_open: LowerOpenResult::Unknown,
        }
    }

    /// Build a non-query open request from explicit logical outcomes.
    pub const fn stream(
        format_tag: u16,
        context: DeviceContextState,
        allocation: StreamAllocationResult,
        lower_open: LowerOpenResult,
    ) -> Self {
        Self {
            flags: 0,
            format_tag,
            query_check: FormatCheckResult::Unknown,
            context,
            allocation,
            lower_open,
        }
    }
}

/// Definite result of the query-only open path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryOpenReply {
    Accepted,
    UnsupportedFormat,
    /// The supplied format-check result does not establish acceptance or rejection.
    Indeterminate,
}

impl QueryOpenReply {
    pub const fn status_code(self) -> Option<u16> {
        match self {
            Self::Accepted => Some(WAVE_STATUS_SUCCESS),
            Self::UnsupportedFormat => Some(WAVE_STATUS_UNSUPPORTED_FORMAT),
            Self::Indeterminate => None,
        }
    }
}

/// Failure statuses returned by the modeled open paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveOpenFailure {
    /// Status 3 is retained without assigning a failure cause.
    UnmappedStatus3,
    /// Status 4 was observed for an already-open context or failed stream allocation.
    ContextOrAllocationStatus4,
    /// Status 7 is retained without assigning a failure cause.
    UnmappedStatus7,
    UnsupportedFormat,
}

impl WaveOpenFailure {
    pub const fn status_code(self) -> u16 {
        match self {
            Self::UnmappedStatus3 => WAVE_STATUS_OPEN_UNMAPPED_3,
            Self::ContextOrAllocationStatus4 => WAVE_STATUS_OPEN_CONTEXT_OR_ALLOCATION,
            Self::UnmappedStatus7 => WAVE_STATUS_OPEN_UNMAPPED_7,
            Self::UnsupportedFormat => WAVE_STATUS_UNSUPPORTED_FORMAT,
        }
    }
}

/// One possible result when the available inputs do not establish a single open outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenPossibility {
    Opened,
    Failed(WaveOpenFailure),
}

/// Result of a query or non-query open request.
#[derive(Debug, PartialEq, Eq)]
pub enum WaveOpenReply {
    Query(QueryOpenReply),
    Opened(WaveOutputStream),
    Failed(WaveOpenFailure),
    /// Possible outcomes are listed without choosing an unverified failure precedence.
    Indeterminate(Vec<OpenPossibility>),
}

impl WaveOpenReply {
    /// Return a status only when this result has one definite public status.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Query(reply) => reply.status_code(),
            Self::Opened(_) => Some(WAVE_STATUS_SUCCESS),
            Self::Failed(failure) => Some(failure.status_code()),
            Self::Indeterminate(_) => None,
        }
    }
}

/// Evaluate the bounded wave-output open behavior.
///
/// Query-only mode is selected by bit 0 of `flags`; it checks the format and never creates a
/// stream. Only tag 1 is confirmed as PCM. For non-query opens, candidates are combined without
/// imposing precedence when multiple failure conditions are supplied.
pub fn open_wave_output(request: WaveOpenRequest) -> WaveOpenReply {
    if request.flags & WAVE_OPEN_QUERY != 0 {
        return WaveOpenReply::Query(query_open(request.format_tag, request.query_check));
    }

    let mut possibilities = Vec::new();
    let format_can_open = request.format_tag == 1;
    if !format_can_open {
        push_unmapped_format_failure(&mut possibilities);
    }

    match request.context {
        DeviceContextState::AlreadyOpen | DeviceContextState::Unknown => {
            push_failure(
                &mut possibilities,
                WaveOpenFailure::ContextOrAllocationStatus4,
            );
        }
        DeviceContextState::NotAlreadyOpen => {}
    }
    match request.allocation {
        StreamAllocationResult::Failed | StreamAllocationResult::Unknown => {
            push_failure(
                &mut possibilities,
                WaveOpenFailure::ContextOrAllocationStatus4,
            );
        }
        StreamAllocationResult::Succeeded => {}
    }

    let service_can_open = matches!(
        request.lower_open,
        LowerOpenResult::Opened | LowerOpenResult::Unknown
    );
    match request.lower_open {
        LowerOpenResult::Opened => {}
        LowerOpenResult::FormatRejected | LowerOpenResult::Unsupported => {
            push_unmapped_format_failure(&mut possibilities);
        }
        LowerOpenResult::UnmappedStatus3 => {
            push_failure(&mut possibilities, WaveOpenFailure::UnmappedStatus3);
        }
        LowerOpenResult::UnmappedStatus7 => {
            push_failure(&mut possibilities, WaveOpenFailure::UnmappedStatus7);
        }
        LowerOpenResult::Unknown => {
            push_unmapped_format_failure(&mut possibilities);
        }
    }

    let context_can_be_open = request.context != DeviceContextState::AlreadyOpen;
    let allocation_can_succeed = request.allocation != StreamAllocationResult::Failed;
    if format_can_open && context_can_be_open && allocation_can_succeed && service_can_open {
        push_possibility(&mut possibilities, OpenPossibility::Opened);
    }

    match possibilities.as_slice() {
        [OpenPossibility::Opened] => WaveOpenReply::Opened(WaveOutputStream::new()),
        [OpenPossibility::Failed(failure)] => WaveOpenReply::Failed(*failure),
        _ => WaveOpenReply::Indeterminate(possibilities),
    }
}

fn query_open(format_tag: u16, check: FormatCheckResult) -> QueryOpenReply {
    if format_tag != 1 {
        return QueryOpenReply::UnsupportedFormat;
    }
    match check {
        FormatCheckResult::Accepted => QueryOpenReply::Accepted,
        FormatCheckResult::Rejected | FormatCheckResult::Unsupported => {
            QueryOpenReply::UnsupportedFormat
        }
        FormatCheckResult::Unknown => QueryOpenReply::Indeterminate,
    }
}

fn push_failure(possibilities: &mut Vec<OpenPossibility>, failure: WaveOpenFailure) {
    push_possibility(possibilities, OpenPossibility::Failed(failure));
}

fn push_unmapped_format_failure(possibilities: &mut Vec<OpenPossibility>) {
    push_failure(possibilities, WaveOpenFailure::UnsupportedFormat);
    push_failure(possibilities, WaveOpenFailure::UnmappedStatus3);
    push_failure(possibilities, WaveOpenFailure::UnmappedStatus7);
}

fn push_possibility(possibilities: &mut Vec<OpenPossibility>, possibility: OpenPossibility) {
    if !possibilities.contains(&possibility) {
        possibilities.push(possibility);
    }
}

/// State for a stream returned only by a definite successful non-query open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WaveOutputStream {
    queued_header_ids: Vec<u64>,
}

impl WaveOutputStream {
    fn new() -> Self {
        Self::default()
    }

    /// Submit a prepared header and append its logical ID in submission order.
    pub fn submit(&mut self, header: &mut WaveHeader) -> WaveSubmissionReply {
        let prepared = header.flags & WAVE_HEADER_PREPARED != 0;
        let in_queue = header.flags & WAVE_HEADER_IN_QUEUE != 0;
        match (prepared, in_queue) {
            (false, true) => WaveSubmissionReply::IndeterminateFailurePrecedence,
            (false, false) => WaveSubmissionReply::Failed(WaveSubmissionFailure::NotPrepared),
            (true, true) => WaveSubmissionReply::Failed(WaveSubmissionFailure::AlreadyQueued),
            (true, false) => {
                header.flags = (header.flags | WAVE_HEADER_IN_QUEUE) & !WAVE_HEADER_DONE;
                header.link_fields = HeaderLinkState::ResetByAcceptedSubmission;
                self.queued_header_ids.push(header.id);
                WaveSubmissionReply::Accepted
            }
        }
    }

    /// Return accepted header IDs in observed submission order.
    pub fn queued_header_ids(&self) -> &[u64] {
        &self.queued_header_ids
    }
}

/// Logical standard wave-header identity and flags used by the queue model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveHeader {
    id: u64,
    flags: u16,
    link_fields: HeaderLinkState,
}

impl WaveHeader {
    pub const fn new(id: u64, flags: u16) -> Self {
        Self {
            id,
            flags,
            link_fields: HeaderLinkState::Unmodeled,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn flags(&self) -> u16 {
        self.flags
    }

    /// Replace flags to model caller-side preparation or another observed flag change.
    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags;
    }

    pub const fn link_fields(&self) -> HeaderLinkState {
        self.link_fields
    }
}

/// Abstract marker for the observed link-field reset; no ABI layout or pointer values are modeled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderLinkState {
    Unmodeled,
    ResetByAcceptedSubmission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveSubmissionFailure {
    AlreadyQueued,
    NotPrepared,
}

impl WaveSubmissionFailure {
    pub const fn status_code(self) -> u16 {
        match self {
            Self::AlreadyQueued => WAVE_STATUS_HEADER_ALREADY_QUEUED,
            Self::NotPrepared => WAVE_STATUS_HEADER_NOT_PREPARED,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveSubmissionReply {
    Accepted,
    Failed(WaveSubmissionFailure),
    /// Both rejection conditions are present; the contract does not establish which status wins.
    IndeterminateFailurePrecedence,
}

impl WaveSubmissionReply {
    pub const fn status_code(self) -> Option<u16> {
        match self {
            Self::Accepted => Some(WAVE_STATUS_SUCCESS),
            Self::Failed(failure) => Some(failure.status_code()),
            Self::IndeterminateFailurePrecedence => None,
        }
    }

    /// Describe the limited dispatch behavior recorded for an accepted submission.
    pub const fn lower_service_dispatch(self) -> Option<LowerServiceDispatch> {
        match self {
            Self::Accepted => Some(LowerServiceDispatch::MayBeCalledIfNotPaused),
            Self::Failed(_) | Self::IndeterminateFailurePrecedence => None,
        }
    }
}

/// The contract allows a queued header to be passed to the lower service when output is not
/// paused, but does not establish whether or when that happens for every accepted header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerServiceDispatch {
    MayBeCalledIfNotPaused,
}
