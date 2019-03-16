use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NonNanF64(f64);
impl NonNanF64 {
    pub fn new(x: f64) -> Self {
        assert!(!x.is_nan()); // TODO(?): Convert NaN to Infinity
        Self(x)
    }

    // pub fn get(self) -> f64 {
    //     self.0
    // }
}
impl Eq for NonNanF64 {}
impl Ord for NonNanF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).expect("never fails")
    }
}
