from typing import Dict
import numpy as np
from random import shuffle
import time
import sys

# https://justinsermeno.com/posts/cfr/
# https://github.com/IanSullivan/PokerCFR/blob/main/kuhn_cfr.py
# https://github.com/bakanaouji/cpp-cfr/blob/master/RegretMinimization/Trainer/Trainer.cpp
class Kunh:

    def __init__(self):
        # str is a card and a history
        self.nodeMap: Dict[str, Node] = {}
        self.expected_game_value = 0
        self.n_cards = 3
        self.nash_equilibrium = dict()
        self.current_player = 0
        self.deck = np.array([0, 1, 2])
        self.n_actions = 2

    def train(self, n_iterations=50000):
        expected_game_value = 0
        for _ in range(n_iterations):
            shuffle(self.deck)
            expected_game_value += self.cfr('', 1, 1)
            for _, v in self.nodeMap.items():
                v.update_strategy()

        expected_game_value /= n_iterations
        display_results(expected_game_value, self.nodeMap)

    def cfr(self, history: str, pr_1, pr_2) -> float:
        """
        Returns expected value of the game for current player

        pr_1 : (0, 1.0), float
        The probability that player A reaches `history`.
        pr_2 : (0, 1.0), float
        The probability that player B reaches `history`.
        """
        n = len(history)
        assert n <= 3
        is_player_1 = n % 2 == 0
        player_card = self.deck[0] if is_player_1 else self.deck[1]

        if self.is_terminal(history):
            card_player = self.deck[0] if is_player_1 else self.deck[1]
            card_opponent = self.deck[1] if is_player_1 else self.deck[0]
            reward = self.get_reward(history, card_player, card_opponent)
            return reward

        node = self.get_node(player_card, history)
        strategy = node.strategy
        assert len(strategy) == 2

        # Counterfactual utility per action.
        action_utils = np.zeros(self.n_actions)

        assert self.n_actions == 2

        for act in range(self.n_actions):
            next_history = history + node.action_dict[act]
            if is_player_1:
                # Utility of this action is the negative utility of the next player.
                # Probability == % they took this action (strategy[act]) * probablity they got here (pr_1)
                action_utils[act] = -1 * self.cfr(next_history, pr_1 * strategy[act], pr_2)
            else:
                action_utils[act] = -1 * self.cfr(next_history, pr_1, pr_2 * strategy[act])

        # Utility of information set.
        util = sum(action_utils * strategy)
        regrets = action_utils - util

        assert len(regrets) == 2
        # action is pass/bet
        # print('*' * 80)
        # print(f"Player1 Card: {self.deck[0]} Player2 Card: {self.deck[1]}\nHistory: {history} Pr1: {pr_1} Pr2: {pr_2}")
        # print(f"Action Utils: {action_utils}")
        # print(f"Strategy: {strategy}")
        print(f"Util: {util}")
        # print(f"Regrets: {regrets}")

        # In regrets, positive is 'good' for that player
        # though the description says the opposite
        # the strategy to pass is [regret_sum[pass] / (regret_sum[pass] + regret_sum[bet])]

        # regrets[action] is counter factual for not taking that action
        # so if its positive, it means you regret not taking that action
        # so you should take that action more often

        if is_player_1:
            node.reach_pr += pr_1
            # regrets are 'weighted' by the probability of this node
            node.regret_sum += pr_2 * regrets
        else:
            node.reach_pr += pr_2
            node.regret_sum += pr_1 * regrets

        # Utility is the money gained / lost, so varies from 2 (winning bet & ante) to -2 (losing bet & ante)
        return util

    @staticmethod
    def is_terminal(history):
        if history[-2:] == 'pp' or history[-2:] == "bb" or history[-2:] == 'bp':
            return True

    @staticmethod
    def get_reward(history, player_card, opponent_card) -> int:
        assert len(history) in [2,3]
        terminal_pass = history[-1] == 'p'
        double_bet = history[-2:] == "bb"
        if terminal_pass:
            if history[-2:] == 'pp':
                # win the ante of 1
                return 1 if player_card > opponent_card else -1
            else:
                # opponent passed
                assert history[-1] == 'p'
                # we bet
                assert history[-2] == 'b'
                assert history[-2:] == 'bp'
                
                return 1
        elif double_bet:
            return 2 if player_card > opponent_card else -2
        
        raise Exception(f"Invalid history: {history}")

    def get_node(self, card: int, history: str) -> "Node":
        if card < 0 or card > 2:
            raise Exception(f"Invalid card: {card}")
        key = str(card) + " " + history
        if key not in self.nodeMap:
            action_dict = {0: 'p', 1: 'b'}
            info_set = Node(key, action_dict)
            self.nodeMap[key] = info_set
            return info_set
        return self.nodeMap[key]


class Node:
    def __init__(self, key, action_dict, n_actions=2):
        self.key = key
        self.n_actions = n_actions
        self.regret_sum = np.zeros(self.n_actions)
        self.strategy_sum = np.zeros(self.n_actions)
        self.action_dict = action_dict
        self.strategy = np.repeat(1/self.n_actions, self.n_actions)
        self.reach_pr = 0
        self.reach_pr_sum = 0

    def update_strategy(self):
        self.strategy_sum += self.reach_pr * self.strategy
        self.reach_pr_sum += self.reach_pr
        self.strategy = self.get_strategy()
        self.reach_pr = 0

    def get_strategy(self):
        regrets = self.regret_sum
        regrets[regrets < 0] = 0
        normalizing_sum = sum(regrets)
        if normalizing_sum > 0:
            return regrets / normalizing_sum
        else:
            return np.repeat(1/self.n_actions, self.n_actions)

    def get_average_strategy(self):
        strategy = self.strategy_sum / self.reach_pr_sum
        # Re-normalize
        total = sum(strategy)
        strategy /= total
        return strategy

    def __str__(self):
        strategies = ['{:03.2f}'.format(x)
                      for x in self.get_average_strategy()]
        return '{} {}'.format(self.key.ljust(6), strategies)


def display_results(ev, i_map):
    print('player 1 expected value: {}'.format(ev))
    print('player 2 expected value: {}'.format(-1 * ev))

    print()
    print('player 1 strategies:')
    sorted_items = sorted(i_map.items(), key=lambda x: x[0])
    # 0 pb means  has a jack, and passed, and the other bet
    for _, v in filter(lambda x: len(x[0]) % 2 == 0, sorted_items):
        print(v)
        # reach pr is set to 0
        # print(f"Reach pr: {v.reach_pr}")
        print(f"Regret pr sum: {v.reach_pr_sum}")
        print(f"Strategy sum: {v.strategy_sum}")
        print(f"Strategy: {v.strategy}")

    print()
    print('player 2 strategies:')
    for _, v in filter(lambda x: len(x[0]) % 2 == 1, sorted_items):
        print(v)


if __name__ == "__main__":
    time1 = time.time()
    trainer = Kunh()
    trainer.train(n_iterations=2000)
    print(abs(time1 - time.time()))
    print(sys.getsizeof(trainer))