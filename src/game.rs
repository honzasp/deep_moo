use std::{cmp, fmt};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    idx_u8: u8,
    cost_u8: u8,
}

impl Card {
    pub fn new(idx: usize) -> Card {
        let mut cost = 0;
        if idx % 5 == 0 {
            cost += 2;
            if idx % 2 == 0 {
                cost += 1;
            }
        }
        if idx % 11 == 0 {
            cost += 5;
        }
        Card { idx_u8: idx as u8, cost_u8: cmp::max(cost, 1) }
    }

    pub fn idx(&self) -> usize { self.idx_u8 as usize }
    pub fn cost(&self) -> usize { self.cost_u8 as usize }

    pub fn mean_cost(begin_idx: usize, end_idx: usize) -> f32 {
        let count = (end_idx - begin_idx) as f32;
        count * (2./5. + 1./10. + 5./11. + 1.)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Card").field(&self.idx_u8).field(&self.cost_u8).finish()
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    rows: Vec<Vec<Card>>
}

impl Table {
    pub fn new(rows: Vec<Vec<Card>>) -> Table {
        Table { rows }
    }

    pub fn row(&self, row_i: usize) -> &[Card] {
        &self.rows[row_i]
    }
    pub fn row_len(&self, row_i: usize) -> usize {
        self.rows[row_i].len()
    }
    pub fn row_cost(&self, row_i: usize) -> usize {
        self.row(row_i).iter().map(|&card| card.cost()).sum()
    }
    pub fn row_last(&self, row_i: usize) -> Card {
        *self.rows[row_i].last().unwrap()
    }

    pub fn card_count(&self) -> usize {
        self.rows.iter().map(|row| row.len()).sum()
    }
    pub fn cards(&self) -> impl Iterator<Item = &Card> {
        self.rows.iter().flatten()
    }

    pub fn match_row(&self, card: Card) -> Option<usize> {
        let mut match_row_i = None;
        let mut match_idx = 0;
        for (row_i, row) in self.rows.iter().enumerate() {
            let last = row.last().unwrap();
            if last.idx() < card.idx() && match_idx < card.idx() {
                match_row_i = Some(row_i);
                match_idx = last.idx();
            }
        }
        match_row_i
    }

    pub fn replace_row(&mut self, row_i: usize, card: Card) {
        self.rows[row_i].clear();
        self.rows[row_i].push(card);
    }
    pub fn push_to_row(&mut self, row_i: usize, card: Card) {
        self.rows[row_i].push(card);
    }
}

#[derive(Debug, Clone)]
pub struct Round {
    pub table: Table,
    pub actions: Vec<Card>,
}

#[derive(Debug, Clone)]
pub struct Rules {
    pub min_card_idx: usize,
    pub max_card_idx: usize,
    pub max_row_len: usize,
    pub hand_len: usize,
    pub row_count: usize,
}

impl Rules {
    pub fn card_count(&self) -> usize {
        self.max_card_idx - self.min_card_idx + 1
    }

    pub fn cards(&self) -> impl Iterator<Item = Card> {
        (self.min_card_idx..=self.max_card_idx).map(Card::new)
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub my_hand: Vec<Card>,
    pub past_rounds: Vec<Round>,
    pub table: Table,
    pub player_count: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_card_costs() {
        assert_eq!(Card::new(2).cost(), 1);
        assert_eq!(Card::new(5).cost(), 2);
        assert_eq!(Card::new(15).cost(), 2);
        assert_eq!(Card::new(25).cost(), 2);
        assert_eq!(Card::new(10).cost(), 3);
        assert_eq!(Card::new(20).cost(), 3);
        assert_eq!(Card::new(30).cost(), 3);
        assert_eq!(Card::new(11).cost(), 5);
        assert_eq!(Card::new(22).cost(), 5);
        assert_eq!(Card::new(33).cost(), 5);
        assert_eq!(Card::new(55).cost(), 7);
    }
}
