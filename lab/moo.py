from typing import Set, Tuple,  List
import random

CARDS = set(range(1, 104+1))

def card_weight(card: int) -> int:
    weight = 1
    if card % 5 == 0:
        weight += 1
    if card % 10 == 0:
        weight += 1
    if card % 11 == 0:
        weight += 4
    return weight

def heuristic_fitness(table: List[List[int]], card: int) -> Tuple[int, float]:
    matching_row = None
    for row in table:
        if card > row[-1] and (matching_row is None or row[-1] > matching_row[-1]):
            matching_row = row

    if matching_row is not None:
        row_weight = sum(map(card_weight, matching_row))
        row_slack = 5 - len(matching_row)
        row_distance = card - matching_row[-1] - 1
        if row_slack > 0:
            return (1, row_weight * row_distance / row_slack)
        else:
            return (3, -row_distance)
    else:
        return (2, -card)

def heuristic_order(table: List[List[int]], hand: Set[int]) -> List[int]:
    return sorted(hand, key = lambda card: heuristic_fitness(table, card))

def calc_card_probs(
    tables: List[List[List[int]]],
    actionss: List[List[int]]
) -> Dict[(int, int), float]:
    all_free_cards = CARDS \
        - {card for table in tables for row in table for card in row} \
        - {card for actions in actionss for card in actions}

    player_count = 5
    round_count = 2

    card_probs = {}
    for _ in range(100000):
        free_cards = list(all_free_cards)
        player_hands = {}
        card_assignment = {}
        for player_i in range(player_count):
            player_hand = {actionss[i][player_i] for i in range(round_count)}
            while len(player_hand) < 10:
                card_i = random.randrange(len(free_cards))
                card = free_cards[card_i]
                card_assignment[card] = player_i
                player_hand.add(card)
                free_cards[card_i] = free_cards[-1]
                free_cards.pop()
            player_hands[player_i] = player_hand

        for card in free_cards:
            card_assignment[card] = 'nobody'

        total_prob = 1.0
        for round_i in range(round_count):
            table = tables[round_i]
            for player_i in range(player_count):
                hand_order = heuristic_order(table, player_hands[player_i])
                action = actionss[round_i][player_i]
                action_prob = 0.5**hand_order.index(action)
                total_prob *= action_prob
                player_hands[player_i].remove(action)

        for card in all_free_cards:
            key = (card, card_assignment[card])
            card_probs[key] = card_probs.get(key, 0.0) + total_prob

    whos = list(range(player_count)) + ['nobody']
    normalized_probs = {}
    for card in all_free_cards:
        prob_sum = sum(card_probs.get((card, who), 0.0) for who in whos)
        for who in whos:
            normalized_probs[(card, who)] = card_probs.get((card, who), 0.0) / prob_sum

    return normalized_probs

if __name__ == '__main__':
    tables = [
        [[20,22,26,28], [30,43,52], [50,63,67,82], [80,85,103]],
        [[31], [30,43,52,64], [96], [80,85,103]],
    ]
    actionss = [
        [29,31,64,96,85],
        [70,100,74,34,40],
    ]

