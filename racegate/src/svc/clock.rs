#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Instant(i32);

impl Instant {
    pub fn from_millis(ms: i32) -> Self {
        Self(ms)
    }

    pub fn as_millis(&self) -> i32 {
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

        // milliseconds, 32 bits, max 24 days (enough!)
        if t_ms < (i32::MAX as u128) {
            Some(Instant(t_ms as i32))
        } else {
            None
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct AdjustedInstant(i32);

impl AdjustedInstant {
    pub fn from_millis(ms: i32) -> Self {
        Self(ms)
    }

    pub fn as_millis(&self) -> i32 {
        self.0
    }
}

pub struct AdjustedClock<'a> {
    clock: &'a Clock,
    offset: ClockOffset,
}

impl<'a> AdjustedClock<'a> {
    pub fn new(clock: &'a Clock, offset: ClockOffset) -> Self {
        Self { clock, offset }
    }

    pub fn now(&self) -> AdjustedInstant {
        let t = self.clock.now().expect("Cannot get time");
        AdjustedInstant::from_millis(t.as_millis() + self.offset.as_millis())
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockOffset(i32);

impl ClockOffset {
    pub fn from_millis(ms: i32) -> Self {
        Self(ms)
    }

    pub fn as_millis(&self) -> i32 {
        self.0
    }
}

pub fn calculate_clock_offset(coordinator_time: Instant, local_time: Instant) -> ClockOffset {
    let c = coordinator_time.as_millis();
    let l = local_time.as_millis();
    ClockOffset::from_millis(c - l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_clock_offset_when_coordinator_started_before_gate() {
        let coord_time = Instant::from_millis(60_000);
        let local_time = Instant::from_millis(10_000);
        let offset = calculate_clock_offset(coord_time, local_time);
        assert_eq!(offset, ClockOffset::from_millis(50_000));
    }

    #[test]
    fn test_calculate_clock_offset_when_coordinator_started_after_gate() {
        let coord_time = Instant::from_millis(60_000);
        let local_time = Instant::from_millis(110_000);
        let offset = calculate_clock_offset(coord_time, local_time);
        assert_eq!(offset, ClockOffset::from_millis(-50_000));
    }
}
