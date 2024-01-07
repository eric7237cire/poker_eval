<template>
  Hand History for {{ file_name }}

  <template v-if="hand_history">
    <div class="board flex flex-wrap justify-center items-center sticky top-0">
      <BoardSelectorCard v-for="card in hand_history.board" :cardId="card.index" />
    </div>

    <template v-for="cur_round in rounds">
      <div class="round-title flex items-center justify-center">{{ cur_round }}</div>
      <div class="actions">
        <template v-for="[action, player] in getActionPlayerListForRound(cur_round, hand_history)">
          <div class="player-name">
            {{ player.player_name }}
          </div>
          <div class="player-cards">
            <BoardSelectorCard :cardId="player.cards.card_hi_lo[0].index" />
            <BoardSelectorCard :cardId="player.cards.card_hi_lo[1].index" />
          </div>
          <div class="action-short">
            {{ formatShortActionText(action, player) }}
          </div>
          <div class="action-long whitespace-pre-wrap self-stretch p-1 font-mono">
            {{ formatActionText(action, player) }}
          </div>
          <div class="player-comment whitespace-pre-wrap self-stretch p-1 font-mono">
            {{ action.player_comment.replaceAll(';', '\n') }}
          </div>
        </template>
      </div>
    </template>
  </template>
</template>

<style lang="postcss" scoped>
.board {
  * {
    margin: 2px;
  }

  :nth-child(4),
  :nth-child(5) {
    margin-left: 10px;
  }
}

.round-title {
    font-size: 3rem;
    margin: 20px;
}

.actions {
  width: 100%;
  box-sizing: border-box;
  display: grid;
  grid-template-columns: minmax(90px, 200px) 90px 1fr 1fr 1fr;
  
  justify-items: stretch;
  align-items: center;

  .player-cards {
    * {
      margin: 2px;
    }
  }

  .player-name, .action-short {
    justify-self: center;
  }

  .player-comment, .action-long {
    white-space: pre-wrap;
    border-top: 1px solid rgba(255,255,255,0.4);
    border-left: 1px solid rgba(255,255,255,0.4);
    align-self: stretch;
    padding: 5px;
  }

}
</style>

<script setup lang="ts">
import { Action, HandHistory, Player } from '@src/lib/hand_history_interfaces';
import { useRoute } from 'vue-router';
import BoardSelectorCard from './BoardSelectorCard.vue';
import { computed, ref } from 'vue';

const route = useRoute();
const file_name = route.params.file_name; // read parameter id (it is reactive)

const hand_history = ref<HandHistory | null>(null);
console.log(`File name [${file_name}]`);

const rounds = computed(() => {
  const r: Array<string> = [];
  if (hand_history.value) {
    for (const action of hand_history.value.actions) {
      if (r.length == 0 || r[r.length - 1] != action.round) {
        r.push(action.round);
      }
    }
  }
  return r;
});

fetch(`/src/assets/hand_history/${file_name}`)
  .then((response) => response.json())
  .then((data) => {
    //console.log(data);
    hand_history.value = data;
  });

function getActionPlayerListForRound(
  round: string,
  handHistory: HandHistory
): Array<[Action, Player]> {
  const r: Array<[Action, Player]> = [];
  for (const action of handHistory.actions) {
    if (action.round == round) {
      r.push([action, handHistory.players[action.player_index]]);
    }
  }

  return r;
}

function getActionType(action: Action): string {
  if (action.action == 'Fold') {
    return `Fold`;
  } else if (action.action == 'Check') {
    return 'Check';
  } else if ('Call' in action.action) {
    return `Call`;
  } else if ('Bet' in action.action) {
    return `Bet`;
  } else if ('Raise' in action.action) {
    return `Raise`;
  } else {
    return `Unknown action ${action.action}`;
  }
}

function formatShortActionText(action: Action, player: Player): string {
  const isAllInComment = action.is_all_in ? ' (All In)' : '';

  if (action.action == 'Fold') {
    return `Folds`;
  } else if (action.action == 'Check') {
    return 'Checks';
  } else if ('Call' in action.action) {
    const callAmount = action.action.Call;
    return `Calls ${formatChips(callAmount)}` + isAllInComment;
  } else if ('Bet' in action.action) {
    const betAmount = action.action.Bet;
    const betEquity = betAmount / (betAmount + action.pot);
    return `Bet ${formatChips(betAmount)}` + isAllInComment;
  } else if ('Raise' in action.action) {
    const increase = action.action.Raise[0];
    const raiseAmount = action.action.Raise[1];
    const amountPutIn = raiseAmount - action.amount_put_in_pot_this_round;
    const raiseEquity = amountPutIn / (amountPutIn + action.pot);
    return (
      `Raise ${formatChips(increase)} to ` +
      `${formatChips(raiseAmount)} ` + isAllInComment
    );
  } else {
    return `Unknown action ${action.action}`;
  }
}

function formatActionText(action: Action, player: Player): string {
  const callEquity = action.current_amt_to_call / (action.current_amt_to_call + action.pot);

  const isAllInComment = action.is_all_in ? ' and is all in' : '';

  if (action.action == 'Fold') {
    return `Folds to bet/raise of ${formatChips(
      action.current_amt_to_call - action.amount_put_in_pot_this_round
    )} leaving pot ${formatChips(action.pot)} with pot equity ${formatEquity(callEquity)}`;
  } else if (action.action == 'Check') {
    return `Left to act: ${action.players_left_to_act}`;
  } else if ('Call' in action.action) {
    const callAmount = action.action.Call;
    const callEquity = callAmount / (callAmount + action.pot);
    return `Calling   : ${formatEquity(callAmount/action.pot)} of pot\n` +
    `Pot       : ${formatChips(action.pot)}\n` +
    `Pot Equity: ${formatEquity(callEquity)}${isAllInComment}`;
  } else if ('Bet' in action.action) {
    const betAmount = action.action.Bet;
    const betEquity = betAmount / (betAmount + action.pot);
    return `Bet ${formatEquity(betAmount/action.pot)} of pot\npot: ${formatChips(action.pot)}\nPot Equity ${formatEquity(
      betEquity
    )}${isAllInComment}`;
  } else if ('Raise' in action.action) {
    const increase = action.action.Raise[0];
    const raiseAmount = action.action.Raise[1];
    const amountPutIn = raiseAmount - action.amount_put_in_pot_this_round;
    const raiseEquity = amountPutIn / (amountPutIn + action.pot);
    return (
      `Raise ${formatEquity(amountPutIn/action.pot)} of pot\n` +
      `Pot: ${formatChips(action.pot)} ` +
      `Pot Equity ${formatEquity(raiseEquity)}${isAllInComment}`
    );
  } else {
    return `Unknown action ${action.action}`;
  }
}

function formatChips(chips: number): string {
  const bb = hand_history.value!.bb;
  return `${chips.toLocaleString('en-US')} (${(chips / bb).toLocaleString('en-US')}bb)`;
}

function formatEquity(equity: number): string {
  return `${(equity * 100).toFixed(2)}%`;
}
</script>
