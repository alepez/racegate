#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Instant(u16);

impl Instant {
    pub fn from_millis(ms: u16) -> Self {
        Self(ms)
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

        // milliseconds, 16 bits, max 18 hours
        if t_ms < (u16::MAX as u128) {
            Some(Instant(t_ms as u16))
        } else {
            None
        }
    }
}
