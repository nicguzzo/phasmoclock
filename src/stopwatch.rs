use std::time::Instant;
//use std::io::{self, Write};

pub struct Stopwatch {
    pub second: u16,
    pub minute: u16,
    pub hour: u16,
    pub start_time: Instant,
    pub current_time: Instant,
    pub seconds: u64,
    pub milliseconds: u32,
    pub last_seconds: u64,
    pub last_milliseconds: u32,
    pub is_running: bool,
}

impl Stopwatch {
    pub fn new() -> Self {
        let start_time = Instant::now();
        let current_time = Instant::now();
        Self {
            second: 0,
            minute: 0,
            hour: 0,
            start_time,
            current_time,
            seconds: 0,
            milliseconds: 0,
            last_seconds: 0,
            last_milliseconds: 0,
            is_running: false,
        }
    }

    pub fn reset(&mut self) {
        println!("reset!!!!");
        if self.is_running {
            self.last_seconds = self.seconds;
            self.last_milliseconds = self.milliseconds;
        }
        self.is_running = !self.is_running;
        let now = Instant::now();
        self.start_time = now;
        self.current_time = now;
        self.second = 0;
        self.hour = 0;
        self.minute = 0;

        self.seconds = 0;
        self.milliseconds = 0;
    }

    pub fn tick(&mut self) {
        if !self.is_running {
            // If paused, keep pushing start_time forward so it doesn't accumulate elapsed time
            let now = Instant::now();
            let paused_duration = now - self.current_time;
            self.start_time += paused_duration;
            self.current_time = now;
            return;
        }
        let elapsed = self.current_time - self.start_time;
        self.seconds = elapsed.as_secs();
        self.milliseconds = elapsed.subsec_millis();
        let minutes = self.seconds / 60;
        let hours = minutes / 60;
        self.second = (self.seconds as u16) % 60;
        self.minute = (minutes % 60) as u16;
        self.hour = (hours as u16) % 24;
        self.current_time = Instant::now();
        //println!("ticking!! {}", self.seconds);
    }
    //pub fn format(&self)->String{
    //    //format!("{:0width$}:{:0width$}:{:0width$} - {}", self.hour, self.minute, self.second, self.seconds, width = 2)
    //    format!("{:.2}",  self.seconds)
    //}
    // pub fn print(&self){
    //     let time=self.format();
    //     print!("\r\x1b {}",time);
    //     let _ = io::stdout().flush();
    // }
}
