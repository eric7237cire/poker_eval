<template>
  <div class="root">
    Hand History for {{ file_name }}

    <template v-if="hand_history">
      <div class="board flex flex-wrap justify-center bg-black items-center sticky top-0 z-10">
        <BoardSelectorCard v-for="card in hand_history.board" :cardId="card.index" />
      </div>

      <template v-for="cur_round in rounds">
        <div class="round-container" :id="cur_round">
          <div class="round-title flex items-center justify-center bg-black">
            <span class="big-text">{{ cur_round }}</span>
            <span>Jump To:</span>
            <a :href= "'#' + j_round" v-for="j_round in rounds">{{j_round}}</a> 
          </div>
          <div class="actions w-full box-border grid items-stretch">
            <template
              v-for="[action, player] in getActionPlayerListForRound(cur_round, hand_history)"
            >
              <div class="player-name grid" :class="getActionType(action)">
                <span class="m-auto">{{ player.player_name }}</span>
              </div>
              <div class="player-cards grid" :class="getActionType(action)">
                <div class="w-full m-auto">
                  <div class="w-full player-cards-inner">
                    <BoardSelectorCard :cardId="player.cards.card_hi_lo[0].index" />
                    <BoardSelectorCard :cardId="player.cards.card_hi_lo[1].index" />
                  </div>
                </div>
              </div>
              <!--grid is to get margin auto to center-->
              <div class="action-short font-mono grid" :class="getActionType(action)">
                <span class="m-auto">
                  {{ formatShortActionText(action, player) }}
                </span>
              </div>
              <div
                class="action-long whitespace-pre-wrap self-stretch p-1 font-mono"
                :class="getActionType(action)"
              >
                {{ formatActionText(action, player) }}
              </div>
              <div
                class="player-comment whitespace-pre-wrap self-stretch p-1 font-mono"
                :class="getActionType(action)"
              >
                {{ action.player_comment.replaceAll(';', '\n') }}
              </div>
            </template>
          </div>
        </div>
      </template>
    </template>
  </div>
</template>

<style lang="postcss" scoped>
/*set less variable to use in css*/
.root {
  --board-height: 70px;

  .board {
    * {
      margin: 2px;
    }

    :nth-child(4),
    :nth-child(5) {
      margin-left: 10px;
    }
    height: var(--board-height);
  }

  .round-title {
    position: sticky;
    top: var(--board-height);

    z-index: 10; /* Ensure it stacks above the content */

    text-align: center;

    .big-text {
      font-size: 3rem;
      margin-right: 10px;
    }

    a {
      font-size: 1rem;
      margin: 5px;
      color: lightblue;
    }
  }
}

.actions {
  grid-template-columns: minmax(90px, 200px) 90px 3fr 4fr 5fr;

  .player-cards-inner {
    * {
      margin: 2px;
    }
  }

  .bet {
    background-color: rgba(0, 0, 255, 0.2);
    color: white;
  }
  .fold {
    background-color: rgba(55, 55, 55, 0.8);
    color: white;
  }
  .raise {
    background-color: rgba(255, 0, 0, 0.2);
    color: white;
  }
  .call {
    background-color: rgba(0, 255, 0, 0.2);
    color: white;
  }

  .player-comment,
  .action-long {
    border-top: 1px solid rgba(255, 255, 255, 0.4);
    border-left: 1px solid rgba(255, 255, 255, 0.4);
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
    return `fold`;
  } else if (action.action == 'Check') {
    return 'check';
  } else if ('Call' in action.action) {
    return `call`;
  } else if ('Bet' in action.action) {
    return `bet`;
  } else if ('Raise' in action.action) {
    return `raise`;
  } else {
    return `Unknown action ${action.action}`;
  }
}

function formatShortActionText(action: Action, player: Player): string {
  const isAllInComment = action.is_all_in ? ' (All In)' : '';

  if (action.action == 'Fold') {
    const neededToPutIn = action.current_amt_to_call - action.amount_put_in_pot_this_round;
    return `Folds to ${formatChips(neededToPutIn)}`;
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
    return `Raise ${formatChips(increase)} to ` + `${formatChips(raiseAmount)} ` + isAllInComment;
  } else {
    return `Unknown action ${action.action}`;
  }
}

function formatActionText(action: Action, player: Player): string {
  const callEquity = action.current_amt_to_call / (action.current_amt_to_call + action.pot);

  if (action.action == 'Fold') {
    const neededToPutIn = action.current_amt_to_call - action.amount_put_in_pot_this_round;
    return (
      `Folds to   : ${formatEquity(neededToPutIn / action.pot)} of pot\n` +
      `Pot        : ${formatChips(action.pot)}\n` +
      `Pot equity : ${formatEquity(callEquity)}`
    );
  } else if (action.action == 'Check') {
    return `Left to act: ${action.players_left_to_act}`;
  } else if ('Call' in action.action) {
    const callAmount = action.action.Call;
    const callEquity = callAmount / (callAmount + action.pot);
    return (
      `Calling    : ${formatEquity(callAmount / action.pot)} of pot\n` +
      `Pot        : ${formatChips(action.pot)}\n` +
      `Pot Equity : ${formatEquity(callEquity)}`
    );
  } else if ('Bet' in action.action) {
    const betAmount = action.action.Bet;
    const betEquity = betAmount / (betAmount + action.pot);
    return (
      `Bet        : ${formatEquity(betAmount / action.pot)} of pot\n` +
      `Pot        : ${formatChips(action.pot)}\n` +
      `Pot Equity : ${formatEquity(betEquity)}`
    );
  } else if ('Raise' in action.action) {
    const increase = action.action.Raise[0];
    const raiseAmount = action.action.Raise[1];
    const amountPutIn = raiseAmount - action.amount_put_in_pot_this_round;
    const raiseEquity = amountPutIn / (amountPutIn + action.pot);
    return (
      `Raise      : ${formatEquity(amountPutIn / action.pot)} of pot\n` +
      `Pot        : ${formatChips(action.pot)}\n` +
      `Pot Equity : ${formatEquity(raiseEquity)}`
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
