use crate::hal::button::ButtonState;
use crate::hal::gate::GateState;
use crate::hal::rgb_led::RgbLed;
use crate::hal::rgb_led::RgbLedColor;
use crate::hal::Platform;
use crate::svc::race_node::*;
use crate::svc::{
    calculate_clock_offset, AdjustedClock, AdjustedInstant, Clock, ClockOffset, Instant,
};

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct SystemState {
    pub gate_state: GateState,
    pub time: Instant,
}

impl From<&AppState> for SystemState {
    fn from(value: &AppState) -> Self {
        match value {
            AppState::Init(x) => SystemState {
                gate_state: GateState::Inactive,
                time: x.time,
            },
            AppState::CoordinatorReady(x) => SystemState {
                gate_state: GateState::Inactive,
                time: x.time,
            },
            AppState::GateStartup(x) => SystemState {
                gate_state: GateState::Inactive,
                time: x.time,
            },
            AppState::GateReady(x) => SystemState {
                gate_state: x.gate_state,
                time: x.time,
            },
        }
    }
}

struct Services<'a> {
    led_controller: LedController<'a>,
    platform: &'a dyn Platform,
    local_clock: Clock,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

        let race_clock = Clock::default();

        let services = Services {
            led_controller,
            platform,
            local_clock: race_clock,
        };

        let state = AppState::default();

        Self { services, state }
    }

    pub fn update(&mut self) {
        let new_state = match self.state {
            AppState::Init(mut state) => state.update(&self.services),
            AppState::CoordinatorReady(mut state) => state.update(&self.services),
            AppState::GateStartup(mut state) => state.update(&self.services),
            AppState::GateReady(mut state) => state.update(&self.services),
        };

        if new_state != self.state {
            // log::info!("{:?}", &new_state);
            self.state = new_state;
        }

        self.services.led_controller.update(&self.state);

        let system_state = (&self.state).into();

        self.services
            .platform
            .http_server()
            .set_system_state(&system_state);

        self.services
            .platform
            .race_node()
            .set_system_state(&system_state);
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
    time: Instant,
}

impl InitState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_connected();
        let gate_state = services.platform.gate().state();
        let button_state = services.platform.button().state();
        let time = services.local_clock.now().expect("Cannot get time");

        let startup_as_gate = is_wifi_connected
            && (button_state != ButtonState::Pressed)
            && gate_state != GateState::Active;

        /* Coordinator gate is always active at startup */
        let startup_as_coordinator = is_wifi_connected
            && (button_state != ButtonState::Pressed)
            && gate_state == GateState::Active;

        if startup_as_gate {
            log::info!("This is a gate");
            AppState::GateStartup(GateStartupState {
                is_wifi_connected,
                time,
            })
        } else if startup_as_coordinator {
            log::info!("This is a coordinator");
            AppState::CoordinatorReady(CoordinatorReadyState {
                is_wifi_connected,
                time,
            })
        } else {
            AppState::Init(*self)
        }
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
struct CoordinatorReadyState {
    is_wifi_connected: bool,
    time: Instant,
}

impl CoordinatorReadyState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_connected();
        let time = services.local_clock.now().expect("Cannot get time");

        let beacon = CoordinatorBeacon { time };

        if let Err(e) = services.platform.race_node().publish(beacon.into()) {
            log::error!("{e}");
        }

        AppState::CoordinatorReady(CoordinatorReadyState {
            is_wifi_connected,
            time,
        })
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
struct GateStartupState {
    is_wifi_connected: bool,
    time: Instant,
}

impl GateStartupState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_connected();
        let time = services.local_clock.now().expect("Cannot get time");
        let gate_state = services.platform.gate().state();

        let clock_offset = services
            .platform
            .race_node()
            .coordinator()
            .map(|coord| calculate_clock_offset(coord.time, time));

        if let Some(clock_offset) = clock_offset {
            log::info!("Gate is ready, offset: {}ms", clock_offset.as_millis());
            let clock = AdjustedClock::new(&services.local_clock, clock_offset);
            let adjusted_time = clock.now();
            AppState::GateReady(GateReadyState {
                is_wifi_connected,
                time,
                gate_state,
                clock_offset,
                adjusted_time,
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
    time: Instant,
    clock_offset: ClockOffset,
    adjusted_time: AdjustedInstant,
}

impl GateReadyState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_connected();
        let gate_state = services.platform.gate().state();
        let time = services.local_clock.now().expect("Cannot get time");
        let clock_offset = self.clock_offset;
        let clock = AdjustedClock::new(&services.local_clock, clock_offset);
        let adjusted_time = clock.now();

        AppState::GateReady(GateReadyState {
            is_wifi_connected,
            gate_state,
            time,
            clock_offset,
            adjusted_time,
        })
    }
}
