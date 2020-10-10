use rand::{RngCore};

use crate::{policy, utils};
use crate::game::{Card, Rules, Table};

pub fn estimate_policy_2_rel_costs(
    rng: &mut dyn RngCore, rules: &Rules,
    table: &Table, hands: &[Vec<Card>],
    round_count: usize,
) -> Vec<f32>
{
    (0..hands[0].len()).into_iter()
        .map(|my_first_action_i| {
            let action_fn = |rng: &mut dyn RngCore, player_i, round_i, table: &Table, hand: &[Card]| {
                if player_i == 0 && round_i == 0 {
                    my_first_action_i
                } else if hand.len() == 1 {
                    0
                } else {
                    let policy = policy::policy_1(rules, table, hands.len(), hand);
                    utils::sample_pdf(rng, &policy)
                }
            };

            let costs = simulate_playout(rng, rules,
                table.clone(), hands.to_vec(), round_count, action_fn);
            let other_cost_mean = costs[1..].iter().sum::<f32>() / (costs.len() - 1) as f32;
            costs[0] - other_cost_mean
        })
        .collect()
}

fn simulate_playout<F>(
    rng: &mut dyn RngCore, rules: &Rules,
    mut table: Table, mut hands: Vec<Vec<Card>>,
    round_count: usize, mut action_fn: F
) -> Vec<f32>
    where F: FnMut(&mut dyn RngCore, usize, usize, &Table, &[Card]) -> usize
{
    let mut costs = vec![0.; hands.len()];
    for round_i in 0..round_count {
        let mut actions = hands.iter_mut().enumerate().map(|(player_i, hand)| {
            let action_i = action_fn(rng, player_i, round_i, &table, hand);
            let action = hand.swap_remove(action_i);
            (player_i, action)
        }).collect::<Vec<_>>();

        actions.sort_unstable_by_key(|(_, card)| card.idx());

        for (player_i, card) in actions {
            costs[player_i] += simulate_action(rng, rules, &mut table, card);
        }
    }
    costs
}

fn simulate_action(
    rng: &mut dyn RngCore, rules: &Rules,
    table: &mut Table, card: Card,
) -> f32
{
    let eaten_row_i =
        if let Some(row_i) = table.match_row(card) {
            if table.row_len(row_i) < rules.max_row_len {
                table.push_to_row(row_i, card);
                return 0.
            }
            row_i
        } else {
            let row_costs = (0..rules.row_count)
                .map(|row_i| table.row_cost(row_i) as f32)
                .collect();
            let policy = policy::costs_to_policy(row_costs);
            utils::sample_pdf(rng, &policy)
        };

    let cost = table.row_cost(eaten_row_i);
    table.replace_row(eaten_row_i, card);
    cost as f32
}
