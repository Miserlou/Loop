pub struct LoopIterator {
    start: f64,
    index: f64,
    end: f64,
    step_by: f64,
    items: Vec<String>,
}

impl LoopIterator {
    pub fn new(
        offset: f64,
        count_by: f64,
        num: Option<f64>,
        items: Vec<String>,
    ) -> LoopIterator {
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

    fn is_last_iterm(&self, current_index: f64) -> bool {
        // end is loop length or --num flag value
        ((self.end - 1_f64) - current_index).abs() < 0.01
    }
}

pub struct Item {
    /// value of $ITEM
    pub item: Option<String>,
    /// index $COUNT of the loop which can be affected by loop-rs flags
    pub count: f64,
    /// the position of the index named $ACTUALCOUNT in loop-rs
    pub actual_count: f64,
    /// true if it is the last item of the iterator
    pub is_last: bool,
}

impl Iterator for LoopIterator {
    type Item = Item;

    fn next(&mut self) -> Option<Item> {
        let current_index = self.index;
        self.start += self.step_by;
        self.index += 1.0;

        if self.index <= self.end {
            let item =
                self.items.get(current_index as usize).map(ToString::to_string);

            let res = Item {
                item,
                count: self.start,
                actual_count: current_index,
                is_last: self.is_last_iterm(current_index),
            };

            Some(res)
        } else {
            None
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn loop_iterator__okay() {
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
}

#[test]
#[allow(non_snake_case)]
fn loop_iterator__count_by_2() {
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
}

#[test]
#[allow(non_snake_case)]
fn loop_iterator__num_1() {
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
