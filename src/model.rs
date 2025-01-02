use embassy_time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum CurrentState {
    STARTED,
    STOPPED
} 

#[derive(Clone)]
pub struct SharedState {
    pub state: CurrentState,
    pub target_minutes: u64,
    pub millis_left: u64,
    pub last: Instant,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            state: CurrentState::STARTED,
            target_minutes: 20,
            millis_left: 20 * 60 * 1000,
            last: Instant::now(),
        }
    }

    pub fn target_down(&mut self) {
        if self.state == CurrentState::STARTED {
            return;
        }

        if self.target_minutes == 5 {
            return;
        }

        self.target_minutes = self.target_minutes - 5;
        self.millis_left = self.target_minutes * 60 * 1000;
    }

    pub fn target_up(&mut self) {
        if self.state == CurrentState::STARTED {
            return;
        }

        if self.target_minutes == 30 {
            return;
        }

        self.target_minutes = self.target_minutes + 5;
        self.millis_left = self.target_minutes * 60 * 1000;
    }

    pub fn toggle(&mut self) {
        match self.state {
            CurrentState::STARTED => self.stop(),
            CurrentState::STOPPED => self.start()
        }
    }

    fn stop(&mut self) {
        self.state = CurrentState::STOPPED;
        self.millis_left = self.target_minutes * 60 * 1000;
        self.last = Instant::now();
    }

    fn start(&mut self) {
        self.state = CurrentState::STARTED;
        self.millis_left = self.target_minutes * 60 * 1000;
        self.last = Instant::now();
    }

    pub fn tick(&mut self) -> (bool, bool) {
        if self.state == CurrentState::STOPPED {
            // keep on keeping on
            self.last = Instant::now();
            return (false, false);
        }

        let old_millis = self.millis_left;

        // calculate millis left
        let next = Instant::now();
        let elapsed = next.duration_since(self.last).as_millis();

        if elapsed >= old_millis {
            self.stop();
            return (true, true);
        }

        self.millis_left = old_millis - elapsed;
        self.last = next;

        // update the display if the seconds have changed
        if  to_seconds(old_millis) != to_seconds(self.millis_left) {
            return (true, false);
        }

        (false, false)
    }
}

pub fn to_seconds(millis: u64) -> u64 {
    Duration::from_millis(millis).as_secs()
}