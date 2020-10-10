use std::ops::{Range};
use std::collections::{HashMap};

use crate::game::{Card};

#[derive(Debug, Clone)]
pub struct CardMatrix<T> {
    row_len: usize,
    rows: HashMap<Card, usize>,
    values: Vec<T>,
}

impl<T: Clone> CardMatrix<T> {
    pub fn new<C>(cards: C, row_len: usize, value: T) -> CardMatrix<T>
        where C: IntoIterator<Item = Card>
    {
        let mut rows = HashMap::new();
        for (row_i, card) in cards.into_iter().enumerate() {
            rows.insert(card, row_i);
        }
        let values = vec![value; rows.len() * row_len];
        CardMatrix { row_len, rows, values }
    }

    /*
    pub fn cards(&self) -> impl Iterator<Item = &Card> {
        self.rows.keys()
    }
    */

    pub fn elem(&self, card: Card, idx: usize) -> &T {
        &self.values[self.elem_idx(card, idx)]
    }
    pub fn elem_mut(&mut self, card: Card, idx: usize) -> &mut T {
        let elem_idx = self.elem_idx(card, idx);
        &mut self.values[elem_idx]
    }
    fn elem_idx(&self, card: Card, idx: usize) -> usize {
        let row_i = self.rows.get(&card).unwrap();
        row_i * self.row_len + idx
    }

    /*
    pub fn row(&self, card: Card) -> &[T] {
        &self.values[self.row_range(card)]
    }
    pub fn row_mut(&mut self, card: Card) -> &mut [T] {
        let row_range = self.row_range(card);
        &mut self.values[row_range]
    }
    fn row_range(&self, card: Card) -> Range<usize> {
        self.row_i_range(*self.rows.get(&card).unwrap())
    }
    */
    fn row_i_range(&self, row_i: usize) -> Range<usize> {
        (row_i * self.row_len)..((row_i + 1) * self.row_len)
    }

    pub fn col(&self, idx: usize) -> Vec<T> {
        (0..self.rows.len()).into_iter()
            .map(|row_i| self.values[row_i * self.row_len + idx].clone())
            .collect()
    }

    pub fn for_each_row<F>(&mut self, mut f: F) where F: FnMut(&mut [T]) {
        for row_i in 0..self.rows.len() {
            let row_range = self.row_i_range(row_i);
            f(&mut self.values[row_range]);
        }
    }

    /*
    pub fn map<F, U>(self, f: F) -> CardMatrix<U> where F: FnMut(T) -> U {
        CardMatrix {
            row_len: self.row_len,
            rows: self.rows,
            values: self.values.into_iter().map(f).collect(),
        }
    }
    */
}

