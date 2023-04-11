#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Instant(u32);

impl Instant {
    pub fn from_millis(ms: u32) -> Self {
        Self(ms)
    }

    pub fn to_millis(&self) -> u32 {
        self.0
    }
}

pub struct Clock {
    start: std::time::Instant,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Clock {
    pub fn now(&self) -> Option<Instant> {
        let t = std::time::Instant::now().checked_duration_since(self.start)?;
        let t_ms = t.as_millis();

        // milliseconds, 32 bits, max 49 days
        if t_ms < (u32::MAX as u128) {
            Some(Instant(t_ms as u32))
        } else {
            None
        }
    }
}
