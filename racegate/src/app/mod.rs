use crate::app::gates::Gates;
use crate::app::race::Race;
use crate::hal::button::ButtonState;
use crate::hal::gate::GateState;
use crate::hal::rgb_led::RgbLed;
use crate::hal::rgb_led::RgbLedColor;
use crate::hal::Platform;
use crate::svc::race_node::*;
use crate::svc::{
    calculate_clock_offset, CoordinatedClock, CoordinatedInstant, LocalClock, LocalInstant,
    LocalOffset,
};

pub mod gates;
mod race;

#[derive(Debug, Default, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SystemState {
    time: CoordinatedInstant,
    gates: Gates,
    race: Race,
}

struct Services<'a> {
    led_controller: LedController<'a>,
    platform: &'a dyn Platform,
    local_clock: LocalClock,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum AppState {
    Init(InitState),
    CoordinatorReady(CoordinatorReadyState),
    GateStartup(GateStartupState),
    GateReady(GateReadyState),
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Init(InitState::default())
    }
}

pub struct App<'a> {
    services: Services<'a>,
    state: AppState,
}

impl<'a> App<'a> {
    pub fn new(platform: &'a mut (dyn Platform)) -> Self {
        let led_controller = LedController {
            led: platform.rgb_led(),
        };

        let race_clock = LocalClock::default();

        let services = Services {
            led_controller,
            platform,
            local_clock: race_clock,
        };

        let state = AppState::default();

        Self { services, state }
    }

    pub fn update(&mut self) {
        let new_state = match &mut self.state {
            AppState::Init(state) => state.update(&self.services),
            AppState::CoordinatorReady(state) => state.update(&self.services),
            AppState::GateStartup(state) => state.update(&self.services),
            AppState::GateReady(state) => state.update(&self.services),
        };

        if new_state != self.state {
            // log::info!("{:?}", &new_state);
            self.state = new_state;
        }

        self.services.led_controller.update(&self.state);
    }
}

struct LedController<'a> {
    led: &'a dyn RgbLed,
}

impl<'a> LedController<'a> {
    pub fn update(&mut self, app_state: &AppState) {
        let color = match app_state {
            AppState::Init(_) => 0xFF0000,
            AppState::CoordinatorReady(_) => 0xFFFFFF,
            AppState::GateStartup(_) => 0xFFFF00,
            AppState::GateReady(state) => {
                if state.gate_state == GateState::Active {
                    0x008080
                } else if state.is_wifi_connected {
                    0x008000
                } else {
                    0x800000
                }
            }
        };

        self.led.set_color(RgbLedColor::from(color));
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
struct InitState {
    is_wifi_connected: bool,
    gate_state: GateState,
    button_state: ButtonState,
    time: LocalInstant,
}

impl InitState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_up();
        let gate_state = services.platform.gate().state();
        let button_state = services.platform.button().state();
        let local_time = services.local_clock.now().expect("Cannot get time");
        let address = address(services);

        log::info!("address: {:?}", address);

        let startup_as_gate = address.is_gate()
            && is_wifi_connected
            && (button_state != ButtonState::Pressed)
            && gate_state != GateState::Active;

        /* Coordinator is selected by dip switch */
        let startup_as_coordinator = address.is_coordinator()
            && is_wifi_connected
            && (button_state != ButtonState::Pressed)
            && gate_state != GateState::Active;

        if startup_as_gate {
            log::info!("This is a gate");
            AppState::GateStartup(GateStartupState {
                is_wifi_connected,
                time: local_time,
            })
        } else if startup_as_coordinator {
            log::info!("This is a coordinator");
            // On coordinator, local time is the coordinated time, without any offset
            AppState::CoordinatorReady(CoordinatorReadyState {
                is_wifi_connected,
                time: CoordinatedInstant::from_millis(local_time.as_millis()),
                system_state: SystemState::default(),
            })
        } else {
            AppState::Init(*self)
        }
    }
}

#[derive(Default, Clone, Eq, PartialEq, Debug)]
struct CoordinatorReadyState {
    is_wifi_connected: bool,
    time: CoordinatedInstant,
    system_state: SystemState,
}

impl CoordinatorReadyState {
    pub fn update(&self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_up();
        let local_time = services.local_clock.now().expect("Cannot get time");

        // On coordinator, local time is the coordinated time, without any offset
        let time = CoordinatedInstant::from_millis(local_time.as_millis());

        let beacon = CoordinatorBeacon { time };

        if let Err(e) = services.platform.race_node().publish(beacon.into()) {
            log::error!("{e}");
        }

        services.platform.race_node().set_coordinator_time(time);

        let gates = services.platform.race_node().gates();

        let mut race = self.system_state.race.clone();

        race.set_gates(&gates);

        let system_state = SystemState { time, gates, race };

        services
            .platform
            .http_server()
            .set_system_state(&self.system_state);

        AppState::CoordinatorReady(CoordinatorReadyState {
            is_wifi_connected,
            time,
            system_state,
        })
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
struct GateStartupState {
    is_wifi_connected: bool,
    time: LocalInstant,
}

impl GateStartupState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_up();
        let time = services.local_clock.now().expect("Cannot get time");
        let gate_state = services.platform.gate().state();

        let clock_offset = services
            .platform
            .race_node()
            .coordinator_time()
            .map(|coord_time| calculate_clock_offset(coord_time, time));

        if let Some(clock_offset) = clock_offset {
            log::info!("Gate is ready, offset: {}ms", clock_offset.as_millis());
            let clock = CoordinatedClock::new(&services.local_clock, clock_offset);
            let coordinated_time = clock.now();
            AppState::GateReady(GateReadyState {
                is_wifi_connected,
                gate_state,
                clock_offset,
                coordinated_time,
                last_activation_time: None,
            })
        } else {
            AppState::GateStartup(GateStartupState {
                is_wifi_connected,
                time,
            })
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct GateReadyState {
    is_wifi_connected: bool,
    gate_state: GateState,
    clock_offset: LocalOffset,
    coordinated_time: CoordinatedInstant,
    last_activation_time: Option<CoordinatedInstant>,
}

impl GateReadyState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_up();
        let gate_state = services.platform.gate().state();
        let clock_offset = self.clock_offset;
        let clock = CoordinatedClock::new(&services.local_clock, clock_offset);
        let coordinated_time = clock.now();
        let addr = address(services);

        let last_activation_time = if gate_state == GateState::Active {
            Some(coordinated_time)
        } else {
            self.last_activation_time
        };

        let beacon = GateBeacon {
            addr,
            state: gate_state,
            last_activation_time,
        };

        if let Err(e) = services.platform.race_node().publish(beacon.into()) {
            log::error!("{e}");
        }

        AppState::GateReady(GateReadyState {
            is_wifi_connected,
            gate_state,
            clock_offset,
            coordinated_time,
            last_activation_time,
        })
    }
}

fn address(services: &Services) -> NodeAddress {
    address_from_env_var().unwrap_or_else(|| services.platform.dip_switch().address())
}

fn address_from_env_var() -> Option<NodeAddress> {
    option_env!("RACEGATE_NODE_ADDRESS")?
        .parse::<u8>()
        .ok()
        .map(NodeAddress::from)
}
