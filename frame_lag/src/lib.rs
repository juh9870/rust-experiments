use std::time::Duration;

#[cfg(test)]
mod monitor_simulator;

/// Targeting 60 FPS
const TARGET_FRAME_RATE: u64 = 60;

/// Delta time is fixed
const DELTA_TIME: f64 = 1.0 / (TARGET_FRAME_RATE as f64);

/// Minimal elapsed time is 1/1000 of a frame
///
/// This does mean that we would get speed ups when running over 60k FPS, but
/// no one cares at this point, can always increase this later if need to arises
const PARTS_PER_FRAME: u64 = 1000;

/// Error margin for accumulated frame to still be considered a frame
const FRAME_PARTS_ERROR_MARGIN: u64 = 17;

/// Amount of frame parts in one second
const PARTS_IN_SECOND: u64 = TARGET_FRAME_RATE * PARTS_PER_FRAME;

/// Some arbitrary margin for frametime snapping
const SNAP_MARGIN: u64 = (0.002 * PARTS_IN_SECOND as f64) as u64;

pub struct FrameData {
    accumulator: FrameParts,
}

#[inline(always)]
fn snap(num: &mut u64, to: u64) {
    if num.abs_diff(to) <= SNAP_MARGIN {
        *num = to;
    }
}

impl Default for FrameData {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameData {
    pub fn new() -> FrameData {
        FrameData {
            accumulator: FrameParts(0),
        }
    }

    /// Notifies frame data about elapsed time in seconds
    pub fn elapsed(&mut self, time: impl Into<FrameParts>) {
        let mut elapsed = time.into().0;
        // Half of the frame
        snap(&mut elapsed, PARTS_PER_FRAME / 2);
        snap(&mut elapsed, PARTS_PER_FRAME);
        snap(&mut elapsed, PARTS_PER_FRAME * 2);
        self.accumulator.0 += elapsed;
        // println!("{} {}, accumulated: {}", "[ELAPSED]", elapsed, self.accumulator.0);
    }

    /// Asks frame data if frame needs to be executed
    ///
    /// This function automatically updates internal timers, and so you should
    /// ensure that this is called only once per frame drawn
    ///
    /// # Examples
    /// ```no_run
    /// while(frame_data.frame()) {
    ///     do_frame()
    /// }
    /// ```
    pub fn frame(&mut self) -> bool {
        if self.accumulator.0 < PARTS_PER_FRAME - FRAME_PARTS_ERROR_MARGIN {
            // println!("{} {}", "[NO_FRAME]".bright_black(), self.accumulator.0);
            false
        } else {
            // println!("{} {}", "[FRAME]", self.accumulator.0);
            self.accumulator.deduct_frame();
            if (self.accumulator.0 >= PARTS_PER_FRAME - FRAME_PARTS_ERROR_MARGIN) {}
            true
        }
    }
}

#[cfg(test)]
impl monitor_simulator::FrameCounter for FrameData {
    fn elapsed(&mut self, seconds: f64) {
        self.elapsed(seconds);
    }
    fn frame(&mut self) -> bool {
        self.frame()
    }
}

pub struct FrameParts(u64);

impl FrameParts {
    fn full_frames(&self) -> u64 {
        self.0 / PARTS_PER_FRAME
    }

    fn has_frame(&self) -> bool {
        self.0 >= PARTS_PER_FRAME
    }

    fn deduct_frame(&mut self) {
        if !self.has_frame() {
            self.0 = 0;
        } else {
            self.0 -= PARTS_PER_FRAME;
        }
    }

    fn try_deduct_frame(&mut self) -> bool {
        if !self.has_frame() {
            return false;
        }
        self.0 -= PARTS_PER_FRAME;
        return true;
    }
}

impl From<f64> for FrameParts {
    fn from(value: f64) -> Self {
        FrameParts((value * PARTS_IN_SECOND as f64).ceil() as u64)
    }
}

impl From<f32> for FrameParts {
    fn from(value: f32) -> Self {
        FrameParts((value * PARTS_IN_SECOND as f32).ceil() as u64)
    }
}

impl From<&Duration> for FrameParts {
    fn from(value: &Duration) -> Self {
        FrameParts((value.as_secs_f64() * PARTS_IN_SECOND as f64).ceil() as u64)
    }
}

#[cfg(test)]
mod tests {
    use colored::Colorize;

    use super::monitor_simulator::MonitorSimulator;
    use super::FrameData;

    #[test]
    fn run_simulation() {
        let mut monitor = MonitorSimulator::new(
            144.0,
            false,
            &|x| match x {
                0 => print!("{}|", format!("{x}").bright_black()),
                1 => print!("{}|", format!("{x}").green()),
                2.. => print!("{}|", format!("{x}").red()),
            },
            (),
        );
        let mut frames = FrameData::new();
        monitor.run(10000, &mut frames);
        println!("{}", monitor.report());
    }
}
