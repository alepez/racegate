pub trait Gate {
    fn is_active(&self) -> bool;
    fn status(&self) -> GateStatus;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GateStatus {
    Inactive,
    Active,
}

impl Default for GateStatus {
    fn default() -> Self {
        GateStatus::Inactive
    }
}
