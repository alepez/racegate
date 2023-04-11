use std::time::Duration;

use crate::app::gates::Gates;
use crate::svc::CoordinatedInstant;

#[derive(Debug, Default, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Race {
    start_time: Option<CoordinatedInstant>,
    finish_time: Option<CoordinatedInstant>,
    duration: Option<Duration>,
}

impl Race {
    pub fn set_gates(&mut self, gates: &Gates) {
        let start_time = gates.start_gate().last_activation_time;
        let finish_time = gates.finish_gate().last_activation_time;

        if let Some(start_time) = start_time {
            self.start_time = Some(start_time);
        }

        if let Some(finish_time) = finish_time {
            self.finish_time = Some(finish_time);
        }

        if let Some(start_time) = start_time {
            if let Some(finish_time) = finish_time {
                if start_time > finish_time {
                    self.finish_time = None;
                    self.duration = None;
                } else {
                    let duration_millis = finish_time.as_millis() - start_time.as_millis();
                    self.duration = Some(Duration::from_millis(duration_millis as u64));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::app::gates::Gate;

    use super::*;

    #[test]
    fn test_race_default() {
        let race = Race::default();
        assert_debug_snapshot!(race);
    }

    #[test]
    fn test_race_with_start_gate_active() {
        let mut race = Race::default();
        race.set_gates(&Gates::new([
            Gate {
                active: true,
                last_activation_time: Some(CoordinatedInstant::from_millis(10_000)),
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
        ]));
        assert_debug_snapshot!(race);
    }

    #[test]
    fn test_race_with_start_and_finish_gates_active() {
        let mut race = Race::default();
        race.set_gates(&Gates::new([
            Gate {
                active: true,
                last_activation_time: Some(CoordinatedInstant::from_millis(10_000)),
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
        ]));
        race.set_gates(&Gates::new([
            Gate {
                active: false,
                last_activation_time: Some(CoordinatedInstant::from_millis(10_000)),
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: true,
                last_activation_time: Some(CoordinatedInstant::from_millis(20_000)),
            },
        ]));
        assert_debug_snapshot!(race);
    }

    #[test]
    fn test_race_with_start_after_finish() {
        let mut race = Race::default();
        race.set_gates(&Gates::new([
            Gate {
                active: true,
                last_activation_time: Some(CoordinatedInstant::from_millis(10_000)),
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
        ]));
        race.set_gates(&Gates::new([
            Gate {
                active: false,
                last_activation_time: Some(CoordinatedInstant::from_millis(10_000)),
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: true,
                last_activation_time: Some(CoordinatedInstant::from_millis(20_000)),
            },
        ]));
        race.set_gates(&Gates::new([
            Gate {
                active: false,
                last_activation_time: Some(CoordinatedInstant::from_millis(30_000)),
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: false,
                last_activation_time: None,
            },
            Gate {
                active: true,
                last_activation_time: Some(CoordinatedInstant::from_millis(20_000)),
            },
        ]));
        assert_debug_snapshot!(race);
    }
}
