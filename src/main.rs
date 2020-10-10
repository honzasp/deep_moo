use std::{collections::{HashSet}, io, env, fs};
use rand_pcg::{Pcg64Mcg};

use crate::distrib::{HandsDistrib};
use crate::game::{Card, GameState, Round, Rules, Table};

mod card_matrix;
mod distrib;
mod game;
mod mc;
mod policy;
mod utils;

fn main() -> io::Result<()> {
    let rules = Rules {
        min_card_idx: 1,
        max_card_idx: 104,
        max_row_len: 5,
        hand_len: 10,
        row_count: 4,
    };

    let state = {
        let args = env::args_os().collect::<Vec<_>>();
        if args.len() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Use: deep_moo <game.txt>"));
        }

        let input_file = fs::File::open(&args[1])?;
        let mut input = io::BufReader::new(input_file);
        read_game_state(&mut input, &rules)?
    };

    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);
    let distrib = HandsDistrib::estimate(&mut rng, &rules, &state);

    let mut weight_sum = 0.;
    let mut rel_costs_sum = vec![0.; state.my_hand.len()];
    for _ in 0..10000 {
        let (mut hands, weight) = distrib.sample(&mut rng, &rules);
        hands[0] = state.my_hand.clone();

        let rel_costs = mc::estimate_policy_2_rel_costs(
            &mut rng, &rules, &state.table, &hands, state.my_hand.len());

        weight_sum += weight;
        for i in 0..state.my_hand.len() {
            rel_costs_sum[i] += rel_costs[i] * weight;
        }
    }

    let rel_costs = rel_costs_sum.iter().map(|x| x / weight_sum).collect::<Vec<_>>();
    let mut best_action_is = (0..state.my_hand.len()).collect::<Vec<_>>();
    best_action_is.sort_by(|&i, &j| utils::compare_f32(rel_costs[i], rel_costs[j]));

    for &action_i in best_action_is.iter() {
        println!("{:3} {:6.2}", state.my_hand[action_i].idx(), -rel_costs[action_i]);
    }

    Ok(())
}

fn read_game_state<I: io::BufRead>(input: I, rules: &Rules) -> io::Result<GameState> {
    let mut my_hand: Option<HashSet<Card>> = None;
    let mut player_names: Option<Vec<String>> = None;
    let mut past_rounds: Vec<Round> = Vec::new();
    let mut current_table: Vec<Vec<Card>> = Vec::new();

    for (line_i, line) in input.lines().enumerate() {
        let line = line?;

        let err = |reason| {
            let msg = format!("{}: {}", line_i + 1, reason);
            io::Error::new(io::ErrorKind::InvalidInput, msg)
        };

        let parse_card = |idx_str: &str| {
            match idx_str.parse::<usize>() {
                Ok(idx) if idx < rules.min_card_idx => Err(err("bad card (index too low)")),
                Ok(idx) if idx > rules.max_card_idx => Err(err("bad card (index too high)")),
                Err(_) => Err(err("bad card (could not parse integer)")),
                Ok(idx) => Ok(Card::new(idx)),
            }
        };

        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.len() == 0 || words[0] == "#" {
            continue
        } else if words[0] == "h" {
            if my_hand.is_some() { return Err(err("duplicated 'h' command")); }
            let hand = words[1..].iter().cloned().map(parse_card)
                .collect::<io::Result<HashSet<Card>>>()?;
            if hand.len() != rules.hand_len { return Err(err("bad hand length")); }
            my_hand = Some(hand);
        } else if words[0] == "p" {
            if player_names.is_some() { return Err(err("duplicated 'p' command")); }
            if words.len() < 3 { return Err(err("too few players")); }
            player_names = Some(words[1..].iter().map(|&n| n.to_string()).collect());
        } else if words[0] == "t" {
            if words.len() < 2 { return Err(err("row cannot be empty")); }
            current_table.push(words[1..].iter().cloned().map(parse_card)
                .collect::<io::Result<Vec<Card>>>()?);
        } else if words[0] == "a" {
            let my_hand = my_hand.as_mut().ok_or(err("missing 'h' command"))?;
            let player_count = player_names.as_ref().map(|ns| ns.len()).ok_or(err("missing 'p' command"))?;
            if current_table.len() != rules.row_count { return Err(err("wrong number of rows on table")); }

            let actions = words[1..].iter().cloned().map(parse_card)
                .collect::<io::Result<Vec<Card>>>()?;
            if actions.len() != player_count { return Err(err("wrong number of actions")); }

            if !my_hand.remove(&actions[0]) { return Err(err("my action was not in my hand")); }
            past_rounds.push(Round { table: Table::new(current_table), actions });
            current_table = Vec::new();
        } else {
            return Err(err("unknown command"));
        }
    }

    let err = |reason| {
        let msg = format!("end: {}", reason);
        io::Error::new(io::ErrorKind::InvalidInput, msg)
    };

    let mut my_hand = my_hand.map(|h| h.into_iter().collect::<Vec<_>>()).ok_or(err("missing 'h' command"))?;
    my_hand.sort_by_key(|&card| card.idx());
    let player_count = player_names.map(|ns| ns.len()).ok_or(err("missing 'p' command"))?;
    if current_table.len() != rules.row_count { return Err(err("wrong number of rows on table")); }
    let table = Table::new(current_table);
    Ok(GameState { my_hand, past_rounds, table, player_count })
}
