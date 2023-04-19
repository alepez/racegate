#![allow(non_snake_case)]

use dioxus_desktop::Config;
use racegate::app::SystemState;
use racegate_ui::app::{Dashboard, DashboardProps};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system_state = Box::new(SystemState::default());
    let system_state = Box::leak(system_state);
    let config = Config::default();
    let props = DashboardProps { system_state };
    dioxus_desktop::launch_with_props(Dashboard, props, config);
    Ok(())
}
