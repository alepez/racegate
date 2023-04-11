#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct RaceInstant(u16);

impl RaceInstant {
    pub fn from_millis(ms: u16) -> Self {
        Self(ms)
    }
}

pub struct RaceClock {
    start: std::time::Instant,
}

impl RaceClock {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn now(&self) -> Option<RaceInstant> {
        let t = std::time::Instant::now().checked_duration_since(self.start)?;
        let t_ms = t.as_millis();

        // milliseconds, 16 bits, max 18 hours
        if t_ms < (u16::MAX as u128) {
            Some(RaceInstant(t_ms as u16))
        } else {
            None
        }
    }
}
