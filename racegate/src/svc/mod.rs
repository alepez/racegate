use crate::app::SystemState;

pub trait HttpServer {
    fn set_system_state(&self, status: &SystemState);
}

pub trait RaceNode {
    fn set_system_state(&self, status: &SystemState);
}
