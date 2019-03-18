pub struct LoopIterator {
    start: f64,
    index: f64,
    end: f64,
    step_by: f64,
    items: Vec<String>,
}

impl LoopIterator {
    pub fn new(offset: f64, count_by: f64, num: Option<f64>, items: Vec<String>) -> LoopIterator {
        let end = if let Some(num) = num {
            num
        } else if !items.is_empty() {
            items.len() as f64
        } else {
            std::f64::INFINITY
        };
        LoopIterator {
            start: offset - count_by,
            index: 0.0,
            end,
            step_by: count_by,
            items,
        }
    }
}

pub struct LoopResult {
    pub item: Option<String>,
    pub actual_count: f64,
    pub index: f64,
}

impl Iterator for LoopIterator {
    type Item = LoopResult;

    fn next(&mut self) -> Option<LoopResult> {
        let current_index = self.index;
        self.start += self.step_by;
        self.index += 1.0;

        if self.index <= self.end {
            let item = self.items.get(self.index as usize).map(ToString::to_string);

            let res = LoopResult {
                item,
                actual_count: self.start,
                index: current_index,
            };

            Some(res)
        } else {
            None
        }
    }
}
