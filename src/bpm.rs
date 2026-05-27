use std::collections::VecDeque;
use std::time::Instant;

pub struct BpmTracker {
    taps: VecDeque<Instant>,
    offset:f64,
    multipliers:[f64;5],
    multiplier_index:usize,
    blood_mode:bool,
}

impl BpmTracker {
    pub fn new() -> Self {
        Self {
            taps: VecDeque::with_capacity(5), // Keep the last 5 taps for a smooth average
            offset: 0.0,
            multipliers:[0.5,0.75,1.0,1.25,1.5],
            multiplier_index:2,
            blood_mode:false,
        }
    }

    pub fn cycle_multiplier(&mut self){
        self.multiplier_index = (self.multiplier_index +1)%self.multipliers.len();
    }
    pub fn toggle_blood_moon(&mut self){
        self.blood_mode=!self.blood_mode;
    }
    pub fn get_speed_multiplier(&self) -> u32 {
        (self.multipliers[self.multiplier_index]*100.0) as u32
    }
    pub fn is_blood_mode(&self) -> bool {
        self.blood_mode
    }
    pub fn tap(&mut self) {
        self.taps.push_back(Instant::now());
        // Keep only the most recent 5 taps
        if self.taps.len() > 5 {
            self.taps.pop_front();
        }
    }

    pub fn tick(&mut self) {
        // If more than 2 seconds have passed since the last tap, assume we stopped walking
        //if let Some(&last_tap) = self.taps.back() {
        //    if last_tap.elapsed().as_secs_f64() > 2.0 {
        //        self.taps.clear();
        //    }
        //}
    }

    /// Returns a tuple of (BPM, Meters per Second)
    pub fn calculate(&self) -> (f64, f64) {
        if self.taps.len() < 2 {
            return (0.0, 0.0);
        }

        let first = self.taps.front().unwrap();
        let last = self.taps.back().unwrap();

        let duration_secs = last.duration_since(*first).as_secs_f64();
        if duration_secs <= 0.0 {
            return (0.0, 0.0);
        }

        let steps = (self.taps.len() - 1) as f64;
        let steps_per_sec = steps / duration_secs;


        let bpm = steps_per_sec * 60.0;
        let bpm_adj = bpm/1.0+(self.offset/100.0);
        //let speed_ms = steps_per_sec * self.stride_meters;
        let bm=if self.blood_mode {1.15} else {1.0};
        let speed_ms=bpm/(self.multipliers[self.multiplier_index]*bm*(60.0+bpm_adj*0.075));
        (bpm, speed_ms)
    }
}