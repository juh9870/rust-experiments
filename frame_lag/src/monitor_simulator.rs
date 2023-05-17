use colored::Colorize;
use rand::prelude::*;

pub struct MonitorSimulator<'a> {
    // Config
    pub monitor_refresh_rate: f64,
    pub vsync: bool,

    pub game: Box<dyn MonitorGameSimulation + 'a>,
    pub reporter: &'a dyn Fn(u64),

    // simulation
    pub frame_updates: u64,
    pub total_updates: u64,
    pub first_vsync: u64,
    pub last_vsync: u64,
    pub missed_updates: u64,
    pub double_updates: u64,
    pub system_time: f64,
    pub timing_fuzziness: f64,
}

impl<'a> MonitorSimulator<'a> {
    pub fn new(
        monitor_refresh_rate: f64,
        vsync: bool,
        reporter: &'a impl Fn(u64),
        game: impl MonitorGameSimulation + 'a,
    ) -> MonitorSimulator<'a> {
        MonitorSimulator {
            monitor_refresh_rate,
            vsync,
            frame_updates: 0,
            total_updates: 0,
            first_vsync: 0,
            last_vsync: 0,
            missed_updates: 0,
            double_updates: 0,
            system_time: 0.0,
            timing_fuzziness: 1.0 / 60.0 * 0.005,
            reporter,
            game: Box::new(game),
        }
    }

    fn fuzzy(&self) -> f64 {
        rand::thread_rng().gen_range(-self.timing_fuzziness..=self.timing_fuzziness)
    }

    fn simulate_update(&mut self) {
        self.system_time += f64::max(0.0, self.game.game_update_time() + self.fuzzy() * 0.01);
        self.total_updates += 1;
        self.frame_updates += 1;
    }

    fn simulate_render(&mut self) {
        self.system_time += f64::max(0.0, self.game.game_render_time() + self.fuzzy() * 0.01);
    }

    fn simulate_display(&mut self) {
        if self.vsync {
            self.system_time += f64::max(
                0.0,
                (self.system_time * self.monitor_refresh_rate).ceil() / self.monitor_refresh_rate
                    - self.system_time
                    + self.fuzzy(),
            )
        } else {
            self.system_time += f64::max(0.0, self.game.game_display_time() + self.fuzzy());
        }

        let current_vsync = (self.system_time * self.monitor_refresh_rate).round() as u64;
        if self.last_vsync != current_vsync {
            for i in self.last_vsync..(current_vsync - 1) {
                (self.reporter)(0);
                self.missed_updates += 1;
            }
            (self.reporter)(self.frame_updates);
            if self.frame_updates > 1 {
                self.double_updates += 1;
            }
            self.last_vsync = current_vsync;
            self.frame_updates = 0;
        }
    }

    fn simulate_busy(&mut self) {
        self.system_time += f64::max(0.0, self.game.busy_time() + self.fuzzy() * 0.00001);
    }

    pub fn run(&mut self, iterations: u64, counter: &mut impl FrameCounter) {
        self.system_time = rand::thread_rng().gen::<f64>() * 10000.0;
        let mut prev_frame_time = self.system_time;
        self.last_vsync = (self.system_time * self.monitor_refresh_rate).round() as u64;
        self.first_vsync = self.last_vsync;
        while self.total_updates < iterations {
            let current_frame_time = self.system_time;
            let delta_frame_time = current_frame_time - prev_frame_time;
            counter.elapsed(delta_frame_time);
            prev_frame_time = current_frame_time;
            while counter.frame() || self.total_updates == 0 {
                self.simulate_update();
            }
            self.simulate_render();
            self.simulate_display();
            self.simulate_busy();
        }
    }

    pub fn report(&self) -> String {
        let MonitorSimulator {
            total_updates,
            double_updates,
            missed_updates,
            last_vsync,
            first_vsync,
            monitor_refresh_rate,
            ..
        } = self;
        let total_vsyncs = last_vsync - first_vsync;
        let total_updates_time = *total_updates as f64 * (1.0 / 60.0);
        let system_time = (last_vsync - first_vsync) as f64 / monitor_refresh_rate;
        format!("Total updates: {total_updates}\nTotal vsyncs: {total_vsyncs}\nTotal double updates: {double_updates}\nTotal skipped renders: {missed_updates}\nGame time: {total_updates_time}\nSystem time: {system_time}")
    }
}

pub trait FrameCounter {
    fn elapsed(&mut self, seconds: f64);
    fn frame(&mut self) -> bool;
}

pub trait MonitorGameSimulation {
    fn game_update_time(&mut self) -> f64 {
        0.00001
    }
    fn game_render_time(&mut self) -> f64 {
        0.005
    }
    fn game_display_time(&mut self) -> f64 {
        0.000001
    }
    fn busy_time(&mut self) -> f64 {
        0.000001
    }
}

impl MonitorGameSimulation for () {}
