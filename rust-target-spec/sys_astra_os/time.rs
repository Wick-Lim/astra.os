// Time stub for ASTRA.OS
// This file goes in: rust/library/std/src/sys/astra_os/time.rs

use crate::fmt;
use crate::time::Duration;

// PIT (Programmable Interval Timer) tick counter
// The kernel's timer interrupt should increment this
static mut TICKS: u64 = 0;

// Called by kernel timer interrupt handler
#[no_mangle]
pub unsafe extern "C" fn astra_os_timer_tick() {
    TICKS += 1;
}

// Get current tick count (incremented by timer interrupt)
fn get_ticks() -> u64 {
    unsafe { TICKS }
}

// PIT frequency: 1000 Hz (1ms per tick)
const PIT_FREQUENCY: u64 = 1000;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    pub fn now() -> Instant {
        Instant {
            ticks: get_ticks(),
        }
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        if self.ticks >= other.ticks {
            let ticks_diff = self.ticks - other.ticks;
            Some(Duration::from_millis(ticks_diff * 1000 / PIT_FREQUENCY))
        } else {
            None
        }
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
        let millis = other.as_millis() as u64;
        let ticks_to_add = millis * PIT_FREQUENCY / 1000;
        Some(Instant {
            ticks: self.ticks.checked_add(ticks_to_add)?,
        })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        let millis = other.as_millis() as u64;
        let ticks_to_sub = millis * PIT_FREQUENCY / 1000;
        Some(Instant {
            ticks: self.ticks.checked_sub(ticks_to_sub)?,
        })
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
}

impl core::ops::Sub for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.checked_sub_instant(&other).unwrap_or(Duration::ZERO)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        self.checked_add_duration(&other).expect("overflow when adding duration to instant")
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        self.checked_sub_duration(&other).expect("overflow when subtracting duration from instant")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    seconds: u64,
}

impl SystemTime {
    // UNIX epoch: 1970-01-01 00:00:00
    pub const UNIX_EPOCH: SystemTime = SystemTime { seconds: 0 };

    pub fn now() -> SystemTime {
        // For now, return a fixed time (2026-01-01 00:00:00)
        // This is approximately 56 years after UNIX epoch
        // TODO: Integrate with RTC (Real-Time Clock) to get actual time
        SystemTime {
            seconds: 56 * 365 * 24 * 60 * 60, // ~2026
        }
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, ()> {
        if self.seconds >= other.seconds {
            Ok(Duration::from_secs(self.seconds - other.seconds))
        } else {
            Err(())
        }
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime {
            seconds: self.seconds.checked_add(other.as_secs())?,
        })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime {
            seconds: self.seconds.checked_sub(other.as_secs())?,
        })
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemTime")
            .field("seconds_since_epoch", &self.seconds)
            .finish()
    }
}
