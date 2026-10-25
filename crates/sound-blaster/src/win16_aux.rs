//! Logical model of the Win16 auxiliary volume query and update paths.
//!
//! This module does not implement the Win16 ABI or the lower sound-driver service. Volume values
//! stay raw because their scale, defaults, clamping, and quantization are unknown.

pub const AUX_VOLUME_QUERY_MESSAGE: u16 = 5;
pub const AUX_VOLUME_UPDATE_MESSAGE: u16 = 6;
pub const AUXILIARY_UNIT_COUNT: u16 = 7;

/// Raw 16-bit left and right values passed to or returned from the logical API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StereoVolume {
    pub left: u16,
    pub right: u16,
}

impl StereoVolume {
    pub const fn new(left: u16, right: u16) -> Self {
        Self { left, right }
    }
}

/// Query-visible state. Initial values are unknown; callers can inject known state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeState {
    Unknown,
    Known(StereoVolume),
}

/// A unit ID covered by the observed setter cases, 0 through 6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuxiliaryUnitId(u8);

impl AuxiliaryUnitId {
    pub const fn new(value: u16) -> Option<Self> {
        if value < AUXILIARY_UNIT_COUNT {
            Some(Self(value as u8))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Target resolution supplied to a query or update request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuxiliaryUnitTarget {
    Id(u16),
    UnresolvedHandle,
    OwnershipCheckFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerServiceState {
    Available,
    Absent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveUnitState {
    Unknown,
    None,
    Unit(AuxiliaryUnitId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImmediateApply {
    Indicated,
    NotIndicated,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuxiliaryVolumeOperation {
    Query,
    Update,
}

/// Numeric statuses observed on the bounded auxiliary dispatcher paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuxiliaryStatus {
    Success,
    InvalidUnitOrHandle,
    LowerServiceAbsent,
    OwnershipCheckFailed,
    UnhandledOrDisabledConflict,
}

impl AuxiliaryStatus {
    /// Logical status value; this does not specify the original ABI return width.
    pub const fn code(self) -> u16 {
        match self {
            Self::Success => 0,
            Self::InvalidUnitOrHandle => 2,
            Self::LowerServiceAbsent => 3,
            Self::OwnershipCheckFailed => 4,
            Self::UnhandledOrDisabledConflict => 8,
        }
    }
}

/// Update action to forward to the lower sound-driver service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LowerServiceVolumeUpdate {
    pub unit: AuxiliaryUnitId,
    pub volume: StereoVolume,
    pub immediate_apply: ImmediateApply,
}

/// Result of a query or update. Selector 7 and a pending lower-service action have no status code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuxiliaryReply {
    Volume(VolumeState),
    /// Action to forward; the lower-service result and final status are unknown.
    ForwardUpdate(LowerServiceVolumeUpdate),
    Status(AuxiliaryStatus),
    SelectorSevenUnresolved(AuxiliaryVolumeOperation),
}

impl AuxiliaryReply {
    pub const fn status_code(self) -> Option<u16> {
        match self {
            Self::Volume(_) => Some(AuxiliaryStatus::Success.code()),
            Self::ForwardUpdate(_) => None,
            Self::Status(status) => Some(status.code()),
            Self::SelectorSevenUnresolved(_) => None,
        }
    }
}

/// State for the bounded Win16 auxiliary volume surface.
///
/// Updates are returned as actions and do not rewrite query-visible state. The contract does not
/// establish whether an update changes that state; callers can inject it with
/// [`Self::set_query_volume_state`].
#[derive(Clone, Debug)]
pub struct Win16Aux {
    lower_service: LowerServiceState,
    volumes: [VolumeState; AUXILIARY_UNIT_COUNT as usize],
    active_unit: ActiveUnitState,
}

impl Win16Aux {
    pub fn new(lower_service: LowerServiceState) -> Self {
        Self {
            lower_service,
            volumes: [VolumeState::Unknown; AUXILIARY_UNIT_COUNT as usize],
            active_unit: ActiveUnitState::Unknown,
        }
    }

    /// Return seven units when lower state is available, otherwise zero.
    pub fn unit_count(&self) -> u16 {
        match self.lower_service {
            LowerServiceState::Available => AUXILIARY_UNIT_COUNT,
            LowerServiceState::Absent => 0,
        }
    }

    /// Inject query-visible state without assuming a driver default or update readback rule.
    pub fn set_query_volume_state(&mut self, unit: AuxiliaryUnitId, state: VolumeState) {
        self.volumes[unit.index()] = state;
    }

    pub fn set_active_unit_state(&mut self, state: ActiveUnitState) {
        self.active_unit = state;
    }

    /// Query the state for a resolved target. Selector 7 remains explicitly unresolved.
    pub fn query_volume(&self, target: AuxiliaryUnitTarget) -> AuxiliaryReply {
        let unit = match resolve_unit(target, AuxiliaryVolumeOperation::Query) {
            Ok(unit) => unit,
            Err(reply) => return reply,
        };
        if self.lower_service == LowerServiceState::Absent {
            return AuxiliaryReply::Status(AuxiliaryStatus::LowerServiceAbsent);
        }
        AuxiliaryReply::Volume(self.volumes[unit.index()])
    }

    /// Return the lower-service action for a volume update without modeling hardware behavior.
    pub fn update_volume(
        &self,
        target: AuxiliaryUnitTarget,
        volume: StereoVolume,
    ) -> AuxiliaryReply {
        let unit = match resolve_unit(target, AuxiliaryVolumeOperation::Update) {
            Ok(unit) => unit,
            Err(reply) => return reply,
        };
        if self.lower_service == LowerServiceState::Absent {
            return AuxiliaryReply::Status(AuxiliaryStatus::LowerServiceAbsent);
        }
        let immediate_apply = match self.active_unit {
            ActiveUnitState::Unknown => ImmediateApply::Unknown,
            ActiveUnitState::None => ImmediateApply::NotIndicated,
            ActiveUnitState::Unit(active) if active == unit => ImmediateApply::Indicated,
            ActiveUnitState::Unit(_) => ImmediateApply::NotIndicated,
        };
        AuxiliaryReply::ForwardUpdate(LowerServiceVolumeUpdate {
            unit,
            volume,
            immediate_apply,
        })
    }
}

fn resolve_unit(
    target: AuxiliaryUnitTarget,
    operation: AuxiliaryVolumeOperation,
) -> Result<AuxiliaryUnitId, AuxiliaryReply> {
    match target {
        AuxiliaryUnitTarget::UnresolvedHandle => {
            Err(AuxiliaryReply::Status(AuxiliaryStatus::InvalidUnitOrHandle))
        }
        AuxiliaryUnitTarget::OwnershipCheckFailed => Err(AuxiliaryReply::Status(
            AuxiliaryStatus::OwnershipCheckFailed,
        )),
        AuxiliaryUnitTarget::Id(value) if value < AUXILIARY_UNIT_COUNT => {
            Ok(AuxiliaryUnitId(value as u8))
        }
        AuxiliaryUnitTarget::Id(7) => Err(AuxiliaryReply::SelectorSevenUnresolved(operation)),
        AuxiliaryUnitTarget::Id(_) => {
            Err(AuxiliaryReply::Status(AuxiliaryStatus::InvalidUnitOrHandle))
        }
    }
}
