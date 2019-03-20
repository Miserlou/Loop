pub struct LoopIterator {
    start: f64,
    index: f64,
    end: f64,
    step_by: f64,
    items: Vec<String>,
}

impl LoopIterator {
    pub fn new(offset: f64, count_by: f64, num: Option<f64>, items: Vec<String>) -> LoopIterator {
        let end = num.unwrap_or_else(|| {
            if items.is_empty() {
                std::f64::INFINITY
            } else {
                items.len() as f64
            }
        });
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
    /// value of $ITEM
    pub item: Option<String>,
    /// index $COUNT of the loop which can be affected by loop-rs flags
    pub count: f64,
    /// the position of the index named $ACTUALCOUNT in loop-rs
    pub actual_count: f64,
}

impl Iterator for LoopIterator {
    type Item = LoopResult;

    fn next(&mut self) -> Option<LoopResult> {
        let current_index = self.index;
        self.start += self.step_by;
        self.index += 1.0;

        if self.index <= self.end {
            let item = self
                .items
                .get(current_index as usize)
                .map(ToString::to_string);

            let res = LoopResult {
                item,
                count: self.start,
                actual_count: current_index,
            };

            Some(res)
        } else {
            None
        }
    }
}

#[test]
fn test_loop_iterator() {
    // okay
    let mut iter = {
        let count_by = 1_f64;
        let offset = 0_f64;
        LoopIterator::new(
            offset,
            count_by,
            None,
            vec!["a", "b"].into_iter().map(str::to_owned).collect(),
        )
        .into_iter()
    };
    let it = iter.next().unwrap();
    assert_eq!(it.item, Some("a".to_owned()));
    assert_eq!(it.count, 0_f64);
    assert_eq!(it.actual_count, 0_f64);

    let it = iter.next().unwrap();
    assert_eq!(it.item, Some("b".to_owned()));
    assert_eq!(it.count, 1_f64);
    assert_eq!(it.actual_count, 1_f64);

    // --count-by 2
    let mut iter = {
        let count_by = 2_f64;
        let offset = 0_f64;
        LoopIterator::new(
            offset,
            count_by,
            None,
            vec!["a", "b"].into_iter().map(str::to_owned).collect(),
        )
        .into_iter()
    };
    let it = iter.next().unwrap();
    assert_eq!(it.item, Some("a".to_owned()));
    assert_eq!(it.count, 0_f64);
    assert_eq!(it.actual_count, 0_f64);

    let it = iter.next().unwrap();
    assert_eq!(it.item, Some("b".to_owned()));
    assert_eq!(it.count, 2_f64);
    assert_eq!(it.actual_count, 1_f64);

    // --num 1
    let mut iter = {
        let count_by = 1_f64;
        let offset = 0_f64;
        LoopIterator::new(
            offset,
            count_by,
            Some(1_f64),
            vec!["a", "b"].into_iter().map(str::to_owned).collect(),
        )
        .into_iter()
    };
    let it = iter.next().unwrap();
    assert_eq!(it.item, Some("a".to_owned()));
    assert_eq!(it.count, 0_f64);
    assert_eq!(it.actual_count, 0_f64);
    assert!(iter.next().is_none());
}
