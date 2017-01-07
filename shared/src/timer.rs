// External Dependencies ------------------------------------------------------
use clock_ticks;


// Timer Abstraction ----------------------------------------------------------
pub struct Timer<A, B, C, D> {
    callbacks: Vec<(u64, Box<FnMut(&mut A, &mut B, &mut C, &D)>)>
}

impl<A, B, C, D> Timer<A, B, C, D> {

    pub fn new() -> Timer<A, B, C, D> {
        Timer {
            callbacks: Vec::new()
        }
    }

    pub fn schedule<F: FnMut(&mut A, &mut B, &mut C, &D) + 'static>(&mut self, callback: F, delay: u64) {
        let time = clock_ticks::precise_time_ms() + delay;
        match self.callbacks.binary_search_by(|probe| time.cmp(&probe.0)) {
            Ok(index) => {
                self.callbacks.insert(index, (time, Box::new(callback)));
            },
            Err(index) => {
                self.callbacks.insert(index, (time, Box::new(callback)));
            }
        }
    }

    pub fn run(&mut self, a: &mut A, b: &mut B, c: &mut C, d: &D) {

        let now = clock_ticks::precise_time_ms();
        while {
            if let Some(next) = self.callbacks.last() {
                now >= next.0

            } else {
                false
            }
        } {
            if let Some((_, mut callback)) = self.callbacks.pop() {
                callback(a, b, c, d);
            }
        }

    }

}

