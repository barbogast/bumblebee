use std::time::{Duration, Instant};

pub struct Debounce<'a, Arg> {
    delay: Duration,
    last_run: Option<Instant>,
    func: &'a dyn Fn(Arg),
}

impl<'a, Arg> Debounce<'a, Arg> {
    pub fn new(delay: Duration, func: &'a dyn Fn(Arg)) -> Self {
        Self {
            delay,
            func,
            last_run: None,
        }
    }
    pub fn maybe_run(&mut self, arg: Arg) {
        if self.last_run.is_some() {
            let then = self.last_run.unwrap();
            let now = Instant::now();

            if now.duration_since(then) > self.delay {
                self.last_run = Some(Instant::now());

                (self.func)(arg);
            }
        } else {
            self.last_run = Some(Instant::now());
            (self.func)(arg);
        }
    }
}
