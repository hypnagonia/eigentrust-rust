use std::cmp::Ordering;
use std::ops::Add;

pub fn nil_if_empty<T>(slice: Vec<T>) -> Option<Vec<T>> {
    if slice.is_empty() {
        None
    } else {
        Some(slice)
    }
}

pub fn filter<T, F>(slice: Vec<T>, pred: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    slice.into_iter().filter(|x| pred(x)).collect()
}

// KBNSummer is the Kahan-Babushka-Neumaier compensated summation algorithm.
pub struct KBNSummer {
    sum: f64,
    compensation: f64,
}

impl KBNSummer {
    pub fn new() -> Self {
        Self {
            sum: 0.0,
            compensation: 0.0,
        }
    }

    pub fn add(&mut self, value: f64) {
        let mut more_sig = self.sum;
        let mut less_sig = value;

        if more_sig.abs() < less_sig.abs() {
            std::mem::swap(&mut more_sig, &mut less_sig);
        }

        self.sum += value;

        // Recover truncated less_sig used in the addition.
        let truncated_less_sig = self.sum - more_sig;

        // Calculate and add the compensation.
        self.compensation += less_sig - truncated_less_sig;
    }

    pub fn sum(&self) -> f64 {
        self.sum + self.compensation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nil_if_empty() {
        let not_empty_slice = vec![3, 4];
        let empty_slice: Vec<i32> = vec![];
        let nil_slice: Vec<i32> = vec![];

        assert_eq!(nil_if_empty(not_empty_slice.clone()), Some(not_empty_slice));
        assert_eq!(nil_if_empty(empty_slice.clone()), None);
        assert_eq!(nil_if_empty(nil_slice.clone()), None);
    }

    #[test]
    fn test_filter() {
        let slice = vec![3, 1, -4, -1, 5, 9, -2, -6, 5, 3, -5, -8, 9, 7];

        let positive = |&x: &i32| x > 0;
        let negative = |&x: &i32| x < 0;
        let zero = |&x: &i32| x == 0;

        assert_eq!(
            filter(slice.clone(), positive),
            vec![3, 1, 5, 9, 5, 3, 9, 7]
        );
        assert_eq!(
            filter(slice.clone(), negative),
            vec![-4, -1, -2, -6, -5, -8]
        );
        assert_eq!(filter(slice.clone(), zero), vec![] as Vec<i32>);
    }
}
