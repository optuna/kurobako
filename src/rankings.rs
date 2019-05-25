use kurobako_core::Result;
use std::cmp::Ordering;
use std::iter;

pub type Score = usize;

#[derive(Debug)]
pub struct Borda<T> {
    items: Vec<(T, Score)>,
}
impl<T: Ord> Borda<T> {
    pub fn new<I>(items: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        Self {
            items: items.zip(iter::repeat(0)).collect(),
        }
    }

    pub fn try_compete<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(&T, &T) -> Result<Ordering>,
    {
        for i in 0..self.items.len() {
            for j in (0..self.items.len()).filter(|&j| j != i) {
                if track!(f(&self.items[i].0, &self.items[j].0))? == Ordering::Less {
                    self.items[i].1 += 1;
                }
            }
        }
        Ok(())
    }

    pub fn scores<'a>(&'a self) -> impl 'a + Iterator<Item = Score> {
        self.items.iter().map(|t| t.1)
    }
}

#[derive(Debug)]
pub struct Firsts<T> {
    items: Vec<(T, Score)>,
}
impl<T: Ord> Firsts<T> {
    pub fn new<I>(items: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        Self {
            items: items.zip(iter::repeat(0)).collect(),
        }
    }

    pub fn try_compete<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(&T, &T) -> Result<Ordering>,
    {
        for i in 0..self.items.len() {
            let mut is_first = true;
            for j in (0..self.items.len()).filter(|&j| j != i) {
                if track!(f(&self.items[i].0, &self.items[j].0))? == Ordering::Greater {
                    is_first = false;
                    break;
                }
            }
            if is_first {
                self.items[i].1 += 1;
            }
        }
        Ok(())
    }

    pub fn scores<'a>(&'a self) -> impl 'a + Iterator<Item = Score> {
        self.items.iter().map(|t| t.1)
    }
}
