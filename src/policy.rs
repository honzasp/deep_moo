use crate::game::{Card, Rules, Table};
use crate::utils;

/// Calculates a basic first-order policy that a "reasonable" actor may play in the given table
/// situation and with the given hand. Returns a normalized pdf where values correspond to actions
/// (cards) from the hand.
pub fn policy_1(rules: &Rules, table: &Table, player_count: usize, hand: &[Card]) -> Vec<f32> {
    let costs = hand.iter().map(|&card| policy_1_q(rules, table, player_count, hand.len(), card)).collect();
    costs_to_policy(costs)
}

/// Converts action costs to a "reasonable" policy.
pub fn costs_to_policy(mut xs: Vec<f32>) -> Vec<f32> {
    xs.iter_mut().for_each(|x| *x = 1. / (0.02 + *x));
    utils::normalize_pdf(&mut xs);
    xs
}

/// Estimates the action value of the given card at the given table situation.
/// Our basic assumption is that other players play completely at random. While this is
/// not in general true in practice for human players, it still provides a basic for a
/// reasonable strategy.
fn policy_1_q(rules: &Rules, table: &Table, player_count: usize, hand_len: usize, card: Card) -> f32 {
    let free_card_count = rules.card_count() - table.card_count() - hand_len;

    if let Some(row_i) = table.match_row(card) {
        // Case 1: this card will be added to row_i (we ignore the possibility that other
        // player may "under-eat" this row with her small card).
        let last_idx = table.row_last(row_i).idx();

        // `slack` is the number of cards that can be added to this row before it
        // overflows and `gap` is the number of cards between the last card in the row and
        // our card, which other players can add to the row before our card.
        let slack = rules.max_row_len - table.row_len(row_i);
        let gap = card.idx() - last_idx - 1;
        if slack > gap || slack >= player_count {
            // There is surely enough room for our card, so we cannot eat this row.
            return 0.
        }

        let row_cost = table.row_cost(row_i) as f32;
        if slack == 0 && gap == 0 {
            // Our card will surely eat this row.
            return row_cost;
        }

        // Estimate the cost of this row once it is full
        let cost = row_cost + slack as f32 * Card::mean_cost(last_idx + 1, card.idx());
        // Estimate the probability that a card randomly played by other player hits the
        // "gap" between the last card in the row and our card.
        let gap_prob = gap as f32 / free_card_count as f32;
        // Calculate the probability that exactly `slack` cards from other players fall
        // into the "gap", forcing us to eat this row.
        let hit_prob = utils::binom_pdf(player_count - 1, slack, gap_prob);

        hit_prob * cost
    } else {
        // Case 2: this card is under all rows, so we may "under-eat" some row.

        // The cost of "under-eating" is the smallest cost among all rows (because we can
        // pick the cheapest row).
        let cost = (0..rules.row_count)
            .map(|row_i| table.row_cost(row_i))
            .min().unwrap() as f32;

        let gap = card.idx() - rules.min_card_idx;
        // Estimate the probability that a player may hit the "gap", playing even smaller
        // card than us and saving us from "under-eating".
        let gap_prob = gap as f32 / free_card_count as f32;
        // Estimate the probability that no player hits the "gap", so we will be forced to
        // "under-eat".
        let hit_prob = (1. - gap_prob).powi(player_count as i32 - 1);

        hit_prob * cost
    }
}
