#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Clean Rust models of the Creative Sound Blaster DSP, mixer and SB16 MPU-401 UART ports.
//! The host supplies ISA DMA transfers and the machine's programmable interrupt controller.

mod adpcm;
mod card;
mod dsp;
mod fm;
mod mixer;
mod mpu;
pub mod win16_aux;
pub mod win16_midi_output;
pub mod win16_wave;
pub mod win16_wave_input;

pub use adpcm::{AdpcmDecoder, AdpcmMode, AdpcmSamples};
pub use card::{SoundBlaster, SoundBlasterConfig};
pub use dsp::{
    DmaDirection, DmaRequest, Dsp, DspEvent, DspVersion, InterruptSource, PcmChannel, PcmSample,
    TimeError, TransferWidth,
};
pub use fm::{FmChipKind, FmSample, FmSynth};
pub use mixer::{Mixer, MixerKind};
pub use mpu::{Mpu401, MpuEvent};
pub use win16_aux::{
    ActiveUnitState, AuxiliaryReply, AuxiliaryStatus, AuxiliaryUnitId, AuxiliaryUnitTarget,
    AuxiliaryVolumeOperation, ImmediateApply, LowerServiceState, LowerServiceVolumeUpdate,
    StereoVolume, VolumeState, Win16Aux, AUXILIARY_UNIT_COUNT, AUX_VOLUME_QUERY_MESSAGE,
    AUX_VOLUME_UPDATE_MESSAGE,
};
pub use win16_midi_output::{
    CallbackInfo as MidiOutputCallbackInfo, CallbackTargetState as MidiOutputCallbackTargetState,
    CurrentActiveCallCount as MidiOutputCurrentActiveCallCount,
    DeviceContextState as MidiOutputDeviceContextState,
    ExpectedDataCount as MidiOutputExpectedDataCount,
    FailedInitializationContext as MidiOutputFailedInitializationContext,
    LowerAudioState as MidiOutputLowerAudioState, MidiOutputOpenReply, MidiOutputOpenRequest,
    OpenCallback as MidiOutputOpenCallback, PortInitialization as MidiOutputPortInitialization,
    PortStateAllocation as MidiOutputPortStateAllocation, ShortMessageReply, ShortMessageRequest,
    StatusByteState, Win16MidiOutput, MIDI_OUTPUT_CALLBACK_VALUE_OPEN,
    MIDI_OUTPUT_STATUS_INITIALIZATION_FAILED, MIDI_OUTPUT_STATUS_INVALID_PORT_OR_CONTEXT,
    MIDI_OUTPUT_STATUS_LOWER_SERVICE_UNAVAILABLE, MIDI_OUTPUT_STATUS_REENTRANCY_REJECTED,
    MIDI_OUTPUT_STATUS_STATE_ALLOCATION_FAILED, MIDI_OUTPUT_STATUS_SUCCESS,
};
pub use win16_wave::{
    open_wave_output, ActiveStreamState, CallbackDelivery, CallbackTargetState,
    CompletionDrainOutcome, CompletionDrainReply, DeviceContextState, FormatCheckResult,
    HeaderLinkState, LowerOpenResult, LowerServiceControlRequest, LowerServiceDispatch,
    OpenPossibility, OutputPauseState, PlaybackPositionState, QueryOpenReply, QueueAdvanceReply,
    QueueKnowledge, ResetQueueEffect, StreamAllocationResult, WaveCallbackEvent, WaveHeader,
    WaveHeaderBufferInfo, WaveHeaderInfo, WaveOpenFailure, WaveOpenReply, WaveOpenRequest,
    WaveOutputStream, WaveResetReply, WaveSubmissionFailure, WaveSubmissionReply,
    WAVE_CALLBACK_BUFFER_DONE, WAVE_HEADER_DONE, WAVE_HEADER_IN_QUEUE, WAVE_HEADER_PREPARED,
    WAVE_OPEN_QUERY, WAVE_STATUS_HEADER_ALREADY_QUEUED, WAVE_STATUS_HEADER_NOT_PREPARED,
    WAVE_STATUS_OPEN_CONTEXT_OR_ALLOCATION, WAVE_STATUS_OPEN_UNMAPPED_3,
    WAVE_STATUS_OPEN_UNMAPPED_7, WAVE_STATUS_SUCCESS, WAVE_STATUS_UNSUPPORTED_FORMAT,
};
