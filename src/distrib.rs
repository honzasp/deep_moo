use std::collections::{HashSet};
use rand::{RngCore, seq::SliceRandom};

use crate::{policy, utils};
use crate::game::{Card, GameState, Rules};
use crate::card_matrix::{CardMatrix};

#[derive(Debug)]
pub struct HandsDistrib {
    card_probs: CardMatrix<f32>,
    mean_owner_probs: Vec<f32>,
    cards: Vec<Card>,
    hand_len: usize,
    player_count: usize,
}

impl HandsDistrib {
    /// Estimates the distribution from previous state of the game, assuming that other
    /// players follow policy_1().
    pub fn estimate(rng: &mut dyn RngCore, rules: &Rules, state: &GameState)
        -> HandsDistrib
    {
        // `known_hands` lists the cards that are known to be in hand of a given owner.
        // `unknown_cards` is a set of cards with unknown owner.
        let mut known_hands = vec![Vec::new(); state.player_count];
        let mut unknown_cards = rules.cards().collect::<HashSet<Card>>();
        for round in state.past_rounds.iter() {
            for player_i in 1..state.player_count {
                let card = round.actions[player_i];
                known_hands[player_i].push(card);
                unknown_cards.remove(&card);
            }
        }
        state.past_rounds.iter()
            .map(|r| r.table.cards()).flatten()
            .chain(state.table.cards())
            .chain(state.my_hand.iter())
            .for_each(|card| { unknown_cards.remove(card); });

        // convert the `unknown_cards` set into a vec and sort it to get a deterministic
        // result (without sorting the order depends on the HashSet randomization).
        let mut unknown_cards = unknown_cards.into_iter().collect::<Vec<Card>>();
        unknown_cards.sort_unstable_by_key(|&card| card.idx());

        // estimate the probabilities of owners for each unknown card
        let card_probs = estimate_probs(rng, rules, state,
            &known_hands, &mut unknown_cards);

        // calculate mean probabilities per owner
        let mut mean_owner_probs = (0..state.player_count)
            .map(|owner_i| card_probs.col(owner_i).iter().sum())
            .collect::<Vec<_>>();
        utils::normalize_pdf(&mut mean_owner_probs);

        // sort the cards so that the cards that are most likely to be in the deck are
        // first
        unknown_cards.sort_unstable_by(|&c1, &c2|
            utils::compare_f32(*card_probs.elem(c2, 0), *card_probs.elem(c1, 0)));

        HandsDistrib {
            card_probs: card_probs,
            mean_owner_probs: mean_owner_probs,
            cards: unknown_cards,
            hand_len: rules.hand_len - state.past_rounds.len(),
            player_count: state.player_count,
        }
    }

    /// Samples hands of owners from this distribution. Returns the hands and a weight of
    /// this sample.
    pub fn sample(&self, rng: &mut dyn RngCore, _rules: &Rules)
        -> (Vec<Vec<Card>>, f32)
    {
        let mut sample_hands = vec![Vec::new(); self.player_count];
        if self.hand_len == 0 {
            return (sample_hands, 1.);
        }

        let deck_len = self.cards.len() - self.hand_len * (self.player_count - 1);
        let mut sample_weight = 1.;
        let mut owner_is = (0..self.player_count).collect::<Vec<_>>();
        let mut sample_probs = vec![0.; self.player_count];
        for &card in self.cards.iter() {
            assert!(!owner_is.is_empty());
            for (i, &owner_i) in owner_is.iter().enumerate() {
                let prob = *self.card_probs.elem(card, owner_i);
                let mean_owner_prob = self.mean_owner_probs[owner_i];
                let hand_len = sample_hands[owner_i].len();
                let max_hand_len = if owner_i == 0 { deck_len } else { self.hand_len };
                sample_probs[i] = prob * (max_hand_len - hand_len) as f32
                    / mean_owner_prob;
            }
            utils::normalize_pdf(&mut sample_probs[..owner_is.len()]);

            let i = utils::sample_pdf(rng, &sample_probs[..owner_is.len()]);
            let owner_i = owner_is[i];
            let sample_prob = sample_probs[i];
            sample_hands[owner_i].push(card);
            sample_weight *= self.card_probs.elem(card, owner_i) / sample_prob;

            let hand_len = sample_hands[owner_i].len();
            let max_hand_len = if owner_i == 0 { deck_len } else { self.hand_len };
            if hand_len >= max_hand_len {
                owner_is.swap_remove(i);
            }
        }
        assert!(owner_is.is_empty());

        (sample_hands, sample_weight)
    }
}

/// Estimates the probabilities Pr(owner_i owns card | state) for every unknown card,
/// assuming that players follow the policy_1(). owner_i = 0 is "the deck", i.e. the
/// probability that a card is in the deck and not in the hand of any player.
fn estimate_probs(
    rng: &mut dyn RngCore, rules: &Rules, state: &GameState,
    known_hands: &[Vec<Card>], unknown_cards: &mut [Card],
) -> CardMatrix<f32>
{
    let mut log_probs = CardMatrix::new(unknown_cards.iter().cloned(),
        state.player_count, -f32::INFINITY);
    for _ in 0..10000 {
        let hands = sample_hands_uniform(rng, rules, known_hands, unknown_cards);
        let hands_log_prob = calc_hands_log_prob(rules, state, &hands);
        for owner_i in 0..state.player_count {
            let unknown_begin = 
                if owner_i == 0 { 0 }
                else { known_hands[owner_i].len() };
            for &card in hands[owner_i][unknown_begin..].iter() {
                let log_prob = log_probs.elem_mut(card, owner_i);
                *log_prob = utils::log_add(*log_prob, hands_log_prob);
            }
        }
    }

    let mut probs = log_probs;
    probs.for_each_row(utils::exp_normalize_log_pdf);
    probs
}

/// Uniformly samples full hands for all players (except us).
fn sample_hands_uniform(
    rng: &mut dyn RngCore, rules: &Rules,
    known_hands: &[Vec<Card>], unknown_cards: &mut [Card]
) -> Vec<Vec<Card>>
{
    unknown_cards.shuffle(rng);
    let mut card_i = 0;
    let mut hands = known_hands.to_vec();
    for player_i in 1..hands.len() {
        while hands[player_i].len() < rules.hand_len {
            hands[player_i].push(unknown_cards[card_i]);
            card_i += 1;
        }
    }
    while card_i < unknown_cards.len() {
        hands[0].push(unknown_cards[card_i]);
        card_i += 1;
    }
    hands
}

/// Calculates the log-probability log Pr(state | hands, policy_1()) that the players
/// would play as they did if they had the given hands and followed the policy_1(). 
fn calc_hands_log_prob(rules: &Rules, state: &GameState, hands: &[Vec<Card>]) -> f32 {
    let mut log_prob = 0.;
    for (round_i, round) in state.past_rounds.iter().enumerate() {
        for player_i in 1..hands.len() {
            assert_eq!(hands[player_i][round_i], round.actions[player_i]);
            let hand = &hands[player_i][round_i..];
            let policy = policy::policy_1(rules, &round.table, state.player_count, hand);
            log_prob += policy[0].ln();
        }
    }
    log_prob
}

