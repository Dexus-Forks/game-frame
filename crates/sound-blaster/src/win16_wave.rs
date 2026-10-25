//! Logical model of the bounded Win16 SB16 wave-output open and queue paths.
//!
//! This module does not implement the Win16 ABI, parse a complete wave-format structure, or call
//! the lower sound-driver service. Callers supply the service verdicts that the contract does not
//! determine. Unknown and overlapping outcomes stay explicit. Completion is a caller-driven
//! logical transition, not a timing or hardware model. The lower-service protocol and reset's
//! queued-header effects remain unresolved.

pub const WAVE_OPEN_QUERY: u16 = 0x0001;
pub const WAVE_HEADER_DONE: u16 = 0x0001;
pub const WAVE_HEADER_PREPARED: u16 = 0x0002;
pub const WAVE_HEADER_IN_QUEUE: u16 = 0x0010;
pub const WAVE_CALLBACK_BUFFER_DONE: u16 = 0x03BD;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveOutputStream {
    queued_header_ids: Vec<u64>,
    completion_ready_header_ids: Vec<u64>,
    queue_knowledge: QueueKnowledge,
    callback_target: CallbackTargetState,
    pause_state: OutputPauseState,
    playback_position: PlaybackPositionState,
    active_stream: ActiveStreamState,
}

impl WaveOutputStream {
    fn new() -> Self {
        Self {
            queued_header_ids: Vec::new(),
            completion_ready_header_ids: Vec::new(),
            queue_knowledge: QueueKnowledge::Known,
            callback_target: CallbackTargetState::Unknown,
            pause_state: OutputPauseState::Unknown,
            playback_position: PlaybackPositionState::Unknown,
            active_stream: ActiveStreamState::Unknown,
        }
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
    pub fn queued_header_ids(&self) -> Option<&[u64]> {
        match self.queue_knowledge {
            QueueKnowledge::Known => Some(&self.queued_header_ids),
            QueueKnowledge::UnresolvedAfterReset => None,
        }
    }

    /// Return known completion-ready IDs, or `None` after a reset made membership unresolved.
    pub fn completion_ready_header_ids(&self) -> Option<&[u64]> {
        match self.queue_knowledge {
            QueueKnowledge::Known => Some(&self.completion_ready_header_ids),
            QueueKnowledge::UnresolvedAfterReset => None,
        }
    }

    pub const fn queue_knowledge(&self) -> QueueKnowledge {
        self.queue_knowledge
    }

    /// Move the next queued header to the completion-ready list after the caller observes its
    /// data processing as complete. This does not model progress, hardware dispatch, or timing.
    pub fn mark_next_queued_header_completion_ready(&mut self) -> QueueAdvanceReply {
        if self.queue_knowledge == QueueKnowledge::UnresolvedAfterReset {
            return QueueAdvanceReply::UnresolvedAfterReset;
        }
        if self.queued_header_ids.is_empty() {
            return QueueAdvanceReply::NoQueuedHeader;
        }
        let header_id = self.queued_header_ids.remove(0);
        self.completion_ready_header_ids.push(header_id);
        QueueAdvanceReply::Ready(header_id)
    }

    /// Drain the next completion-ready header. Header flags and link state change before any
    /// callback event is returned. The caller supplies the logical header object by identity.
    pub fn drain_next_completion(&mut self, header: &mut WaveHeader) -> CompletionDrainReply {
        if self.queue_knowledge == QueueKnowledge::UnresolvedAfterReset {
            return CompletionDrainReply::UnresolvedAfterReset;
        }
        let Some(expected_id) = self.completion_ready_header_ids.first().copied() else {
            return CompletionDrainReply::NoCompletionReadyHeader;
        };
        if expected_id != header.id {
            return CompletionDrainReply::HeaderIdentityMismatch {
                expected_id,
                actual_id: header.id,
            };
        }

        self.completion_ready_header_ids.remove(0);
        header.flags = (header.flags | WAVE_HEADER_DONE) & !WAVE_HEADER_IN_QUEUE;
        header.link_fields = HeaderLinkState::ClearedByCompletion;
        let info = header.info();
        let callback = match self.callback_target {
            CallbackTargetState::Unknown => CallbackDelivery::TargetUnknown,
            CallbackTargetState::Absent => CallbackDelivery::SuppressedNoTarget,
            CallbackTargetState::Present(target_id) => CallbackDelivery::Sent(WaveCallbackEvent {
                target_id,
                callback_value: WAVE_CALLBACK_BUFFER_DONE,
                header: info,
            }),
        };
        CompletionDrainReply::Drained(CompletionDrainOutcome {
            header: info,
            callback,
        })
    }

    pub const fn callback_target(&self) -> CallbackTargetState {
        self.callback_target
    }

    /// Supply callback-target knowledge without modeling a Win16 callback ABI.
    pub fn set_callback_target(&mut self, target: CallbackTargetState) {
        self.callback_target = target;
    }

    pub const fn pause_state(&self) -> OutputPauseState {
        self.pause_state
    }

    /// Mark the stream paused and describe the request to forward to the lower service.
    pub fn pause(&mut self) -> LowerServiceControlRequest {
        self.pause_state = OutputPauseState::Paused;
        LowerServiceControlRequest::PauseOutput
    }

    /// Clear the logical paused state and describe the request to forward to the lower service.
    pub fn restart(&mut self) -> LowerServiceControlRequest {
        self.pause_state = OutputPauseState::Running;
        LowerServiceControlRequest::RestartOutput
    }

    pub const fn playback_position(&self) -> PlaybackPositionState {
        self.playback_position
    }

    pub const fn active_stream(&self) -> ActiveStreamState {
        self.active_stream
    }

    /// Inject opaque logical state for the fields cleared by reset. No position units are assumed.
    pub fn set_playback_state(
        &mut self,
        position: PlaybackPositionState,
        active_stream: ActiveStreamState,
    ) {
        self.playback_position = position;
        self.active_stream = active_stream;
    }

    /// Describe a stop request and clear known playback-position and active-stream state.
    ///
    /// The reset contract does not establish what happens to queued headers, their flags, or their
    /// callbacks. After reset, queue queries and transitions report that state as unresolved.
    pub fn reset(&mut self) -> WaveResetReply {
        self.playback_position = PlaybackPositionState::ClearedByReset;
        self.active_stream = ActiveStreamState::ClearedByReset;
        self.queue_knowledge = QueueKnowledge::UnresolvedAfterReset;
        WaveResetReply {
            lower_service_request: LowerServiceControlRequest::StopOutput,
            queued_header_effect: ResetQueueEffect::HeadersAndCallbacksUnresolved,
        }
    }
}

/// Knowledge about queue membership after the modeled transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueKnowledge {
    Known,
    UnresolvedAfterReset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueAdvanceReply {
    Ready(u64),
    NoQueuedHeader,
    UnresolvedAfterReset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackTargetState {
    Unknown,
    Absent,
    /// Opaque logical target identity; this is not a Win16 callback handle or pointer.
    Present(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackDelivery {
    Sent(WaveCallbackEvent),
    SuppressedNoTarget,
    TargetUnknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveCallbackEvent {
    pub target_id: u64,
    pub callback_value: u16,
    pub header: WaveHeaderInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletionDrainOutcome {
    pub header: WaveHeaderInfo,
    pub callback: CallbackDelivery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionDrainReply {
    Drained(CompletionDrainOutcome),
    NoCompletionReadyHeader,
    UnresolvedAfterReset,
    HeaderIdentityMismatch { expected_id: u64, actual_id: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputPauseState {
    Unknown,
    Paused,
    Running,
}

/// Playback-position value with unknown units, or the known reset effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackPositionState {
    Unknown,
    OpaqueValue(u64),
    ClearedByReset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveStreamState {
    Unknown,
    Inactive,
    Active(u64),
    ClearedByReset,
}

/// Descriptive control intent only; these variants do not assign lower-service codes or arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerServiceControlRequest {
    PauseOutput,
    RestartOutput,
    StopOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetQueueEffect {
    HeadersAndCallbacksUnresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveResetReply {
    pub lower_service_request: LowerServiceControlRequest,
    pub queued_header_effect: ResetQueueEffect,
}

/// Logical standard wave-header identity and flags used by the queue model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveHeader {
    id: u64,
    buffer: WaveHeaderBufferInfo,
    flags: u16,
    link_fields: HeaderLinkState,
}

impl WaveHeader {
    pub const fn new(id: u64, flags: u16) -> Self {
        Self {
            id,
            buffer: WaveHeaderBufferInfo::UNKNOWN,
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

    pub const fn buffer_info(&self) -> WaveHeaderBufferInfo {
        self.buffer
    }

    /// Supply opaque logical buffer identity and length for completion information.
    pub fn set_buffer_info(&mut self, buffer: WaveHeaderBufferInfo) {
        self.buffer = buffer;
    }

    /// Replace flags to model caller-side preparation or another observed flag change.
    pub fn set_flags(&mut self, flags: u16) {
        self.flags = flags;
    }

    pub const fn link_fields(&self) -> HeaderLinkState {
        self.link_fields
    }

    pub const fn info(&self) -> WaveHeaderInfo {
        WaveHeaderInfo {
            id: self.id,
            buffer: self.buffer,
            flags: self.flags,
            link_fields: self.link_fields,
        }
    }
}

/// Opaque logical data identity and byte length carried by a wave header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveHeaderBufferInfo {
    pub data_identity: Option<u64>,
    pub length_bytes: Option<u32>,
}

impl WaveHeaderBufferInfo {
    pub const UNKNOWN: Self = Self {
        data_identity: None,
        length_bytes: None,
    };

    pub const fn new(data_identity: Option<u64>, length_bytes: Option<u32>) -> Self {
        Self {
            data_identity,
            length_bytes,
        }
    }
}

/// Logical completed-header information returned with the callback event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveHeaderInfo {
    pub id: u64,
    pub buffer: WaveHeaderBufferInfo,
    pub flags: u16,
    pub link_fields: HeaderLinkState,
}

/// Abstract marker for link-field effects; no ABI layout or pointer values are modeled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderLinkState {
    Unmodeled,
    ResetByAcceptedSubmission,
    ClearedByCompletion,
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
