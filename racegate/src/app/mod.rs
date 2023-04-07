use crate::hal::button::ButtonState;
use crate::hal::gate::GateState;
use crate::hal::rgb_led::RgbLed;
use crate::hal::rgb_led::RgbLedColor;
use crate::hal::Platform;

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct SystemState {
    pub gate_state: GateState,
}

impl From<&AppState> for SystemState {
    fn from(value: &AppState) -> Self {
        match value {
            AppState::Init(x) => SystemState {
                gate_state: x.gate_state,
            },
        }
    }
}

struct Services<'a> {
    led_controller: LedController<'a>,
    platform: &'a dyn Platform,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
struct InitAppState {
    is_wifi_connected: bool,
    gate_state: GateState,
    button_state: ButtonState,
}

impl InitAppState {
    pub fn update(&mut self, services: &Services) -> AppState {
        let is_wifi_connected = services.platform.wifi().is_connected();
        let gate_state = services.platform.gate().state();
        let button_state = services.platform.button().state();

        AppState::Init(InitAppState {
            is_wifi_connected,
            gate_state,
            button_state,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum AppState {
    Init(InitAppState),
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Init(InitAppState::default())
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

        let services = Services {
            led_controller,
            platform,
        };

        let state = AppState::default();

        Self { services, state }
    }

    pub fn update(&mut self) {
        let new_state = match self.state {
            AppState::Init(mut state) => state.update(&self.services),
        };

        if new_state != self.state {
            log::info!("{:?}", &new_state);
            self.state = new_state;
        }

        self.services.led_controller.update(&self.state);

        self.services
            .platform
            .http_server()
            .set_system_state((&self.state).into());
    }
}

struct LedController<'a> {
    led: &'a dyn RgbLed,
}

impl<'a> LedController<'a> {
    pub fn update(&mut self, app_state: &AppState) {
        let color = match app_state {
            AppState::Init(state) => {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
