#[derive(Debug)]
pub struct LoopIterator {
    start: f64,
    iters: f64,
    end: f64,
    step_by: f64,
}

impl LoopIterator {
    pub fn new(offset: f64, count_by: f64, num: Option<f64>, items: &[String]) -> LoopIterator {
        let end = if let Some(num) = num {
            num
        } else if !items.is_empty() {
            items.len() as f64
        } else {
            std::f64::INFINITY
        };
        LoopIterator {
            start: offset - count_by,
            iters: 0.0,
            end,
            step_by: count_by,
        }
    }
}

impl Iterator for LoopIterator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.start += self.step_by;
        self.iters += 1.0;
        if self.iters <= self.end {
            Some(self.start)
        } else {
            None
        }
    }
}
