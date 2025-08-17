use chrono::Local;

pub trait Clock {
    fn now_formatted(&self, pattern: &str) -> String;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_formatted(&self, pattern: &str) -> String {
        Local::now().format(pattern).to_string()
    }
}

#[cfg(test)]
pub struct FixedClock {
    pub formatted: String,
}

#[cfg(test)]
impl Clock for FixedClock {
    fn now_formatted(&self, _pattern: &str) -> String {
        self.formatted.clone()
    }
}
