//! A simple RAII-based timer for benchmarking function runtimes

use instant::Instant;

pub struct Timer {
    start_t: Instant,
    is_running: bool,
    msg: &'static str,
}

impl Timer {
    pub fn new(msg: &'static str) -> Self {
        Timer {
            start_t: Instant::now(),
            is_running: true,
            msg,
        }
    }

    pub fn _lap(&mut self) {
        self.summary("currently", self.start_t.elapsed().as_nanos());
    }

    pub fn stop(&mut self) -> u128 {
        if !self.is_running {
            return 0;
        }
        let elapsed = self.start_t.elapsed().as_nanos();
        self.summary("finished", elapsed);
        self.is_running = false;
        elapsed
    }

    pub fn stop_silent(&mut self) -> u128 {
        let elapsed = self.start_t.elapsed().as_nanos();
        self.is_running = false;
        elapsed
    }

    fn summary(&self, verb: &str, mut elapsed: u128) {
        let nanos = elapsed % 1000;
        elapsed /= 1000;
        let micros = elapsed % 1000;
        elapsed /= 1000;
        let millis = elapsed % 1000;
        elapsed /= 1000;
        let secs = elapsed % 1000;
        info!(
            "{} {} in {}:{}:{}:{}",
            self.msg, verb, secs, millis, micros, nanos
        );
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if self.is_running {
            self.stop();
        }
    }
}

pub fn time_from(mut t: u128) -> String {
    let nanos = t % 1000;
    t /= 1000;
    let micros = t % 1000;
    t /= 1000;
    let millis = t % 1000;
    t /= 1000;
    let secs = t % 1000;
    format!("{}:{}:{}:{}", secs, millis, micros, nanos)
}
