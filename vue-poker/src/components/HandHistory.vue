<template>
  <div class="root">
    <div class="header">Hand History for {{ file_name }}.</div>

    <template v-if="hand_history">
      <div class="board flex flex-wrap justify-center bg-black items-center sticky top-0 z-10">
        <BoardSelectorCard v-for="card in hand_history.board" :cardId="card.index" />
      </div>

      <template v-for="(cur_round, idx_round) in rounds">
        <div class="round-container" :id="cur_round">
          <div class="round-title flex items-center justify-center bg-black">
            <router-link :to="{ path: '/hh' }"> Back </router-link>
            <span class="big-text">{{ cur_round }}</span>
            <span>Jump To:</span>
            <a :href="'#' + j_round" v-for="j_round in rounds">{{ j_round }}</a>
          </div>

          <div
            class="cards-round-start flex flex-col"
            v-if="idx_round > 0 && nonFoldedPlayersAtRoundStart[idx_round]"
          >
            <!--At each round start, display all non folded players-->
            <div
              class="player-entry grid grid-cols-[minmax(90px,200px)_1fr] my-[3px]"
              v-for="(foldedAtRound, playerIndex) in nonFoldedPlayersAtRoundStart[idx_round]"
            >
              <div class="player-name text-center text-[white] self-center">
                # {{ hand_history.player_ranks_per_round[idx_round - 1][playerIndex] }}
                {{ hand_history.players[playerIndex].player_name }}
              </div>

              <div v-if="foldedAtRound == null" class="flex flex-row">
                <div class="flex flex-row gap-[5px] inline-block">
                  <BoardSelectorCard
                    :cardId="hand_history.players[playerIndex].cards.card_hi_lo[0].index"
                  />
                  <BoardSelectorCard
                    :cardId="hand_history.players[playerIndex].cards.card_hi_lo[1].index"
                  />
                </div>

                <div class="player-cards-inner ml-[15px] flex flex-row gap-[5px]">
                  <BoardSelectorCard
                    :cardId="hand_history.best_player_hands[idx_round - 1][playerIndex][0].index"
                  />
                  <BoardSelectorCard
                    :cardId="hand_history.best_player_hands[idx_round - 1][playerIndex][1].index"
                  />
                  <BoardSelectorCard
                    :cardId="hand_history.best_player_hands[idx_round - 1][playerIndex][2].index"
                  />
                  <BoardSelectorCard
                    :cardId="hand_history.best_player_hands[idx_round - 1][playerIndex][3].index"
                  />
                  <BoardSelectorCard
                    :cardId="hand_history.best_player_hands[idx_round - 1][playerIndex][4].index"
                  />
                </div>
              </div>
              <div v-if="foldedAtRound != null">Folded @ {{ foldedAtRound }}</div>
            </div>
          </div>

          <div class="actions w-full box-border grid items-stretch">
            <template
              v-for="[action, player] in getActionPlayerListForRound(cur_round, hand_history)"
            >
              <div class="player-name" :class="getActionType(action)">
                <a :id="'action' + action.index" :href="'#action' + action.index" class="">{{
                  player.player_name
                }}</a>

                <button
                  class="button-base button-green"
                  @click="handleAnalyzeRange(true, action.index)"
                >
                  Exact
                </button>
                <button
                  class="button-base button-blue"
                  @click="handleAnalyzeRange(false, action.index)"
                >
                  Range
                </button>
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

      <!--Show stacks-->
      <div class="stack-changes flex items-center justify-center">
        <table>
          <tr>
            <th>Player</th>
            <th>Stack</th>
            <th>Change</th>
          </tr>
          <tr v-for="(player, index) in hand_history.players">
            <td class="mr-[10px]" style="width: 100px">{{ player.player_name }}</td>
            <td>{{ player.stack / hand_history.bb }}</td>
            <td style="text-align: right">
              {{ (hand_history.final_stacks[index] - player.stack) / hand_history.bb }}
            </td>
          </tr>
        </table>
      </div>
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

    z-index: 9; /* Ensure it stacks above the content */

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

  .player-name {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;

    * {
      margin: 2px;
    }
  }

  .player-comment,
  .action-long {
    border-top: 1px solid rgba(255, 255, 255, 0.4);
    border-left: 1px solid rgba(255, 255, 255, 0.4);
  }
}
</style>

<script setup lang="ts">
import { Action, HandHistory, Player, Round } from '@src/lib/hand_history_interfaces';
import { useRoute, useRouter } from 'vue-router';
import BoardSelectorCard from './BoardSelectorCard.vue';
import { computed, ref } from 'vue';
import { PlayerState, usePlayerStore } from '@src/stores/player';
import { cardTextStr } from '@src/lib/utils';
import { CardList, useBoardStore } from '@src/stores/board';
import { SELECTABLE_RANGES } from '@src/stores/ranges';
import { nextTick } from 'vue';
import * as _ from 'lodash';

const route = useRoute();
const router = useRouter();
const file_name = route.params.file_name; // read parameter id (it is reactive)

const hand_history = ref<HandHistory | null>(null);
console.log(`File name [${file_name}]`);

const playerStore = usePlayerStore();
const boardStore = useBoardStore();

//Find out what rounds we have by looking at the actions
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

const nonFoldedPlayersAtRoundStart = computed(() => {
  //1st index flop, turn, river
  //stores round in which they folded
  const r: Array<Array<Round | null>> = [];
  if (!hand_history.value) {
    return r;
  }
  let round = 'Preflop';
  const isFolded: Array<Round | null> = hand_history.value.players.map((_) => null);
  //preflop no one has folded
  r.push(_.cloneDeep(isFolded));

  for (const action of hand_history.value.actions) {
    if (action.round != round) {
      round = action.round;
      r.push(_.cloneDeep(isFolded));
    }
    if (action.round == 'River') {
      break;
    }

    if (action.action == 'Fold') {
      isFolded[action.player_index] = action.round;
    }
  }
  return r;
});

fetch(`/src/assets/hand_history/${file_name}`)
  .then((response) => response.json())
  .then((data) => {
    //console.log(data);
    hand_history.value = data;

    const anchor = getAnchor();
    if (anchor) {
      nextTick(() => {
        const offset = getOffsetTop(document.getElementById(anchor)!);
        console.log(`Scrolling to ${anchor} to offset ${offset}`);
        window.scrollTo({
          top: offset - 300,
          //bottom: document.getElementById(anchor)!.offsetTop - 300,
          left: 0,
          behavior: 'smooth'
        });
      });
    }
  });

function getAnchor(): string | null {
  const currentUrl = document.URL;
  const urlParts = currentUrl.split('#');

  return urlParts.length > 1 ? urlParts[1] : null;
}

//https://stackoverflow.com/questions/34422189/get-item-offset-from-the-top-of-page
function getOffsetTop(element: HTMLElement | null): number {
  if (!element) return 0;
  return getOffsetTop(element.offsetParent as HTMLElement) + element.offsetTop;
}

function getActionPlayerListForRound(
  round: string,
  handHistory: HandHistory
): Array<[Action, Player]> {
  const r: Array<[Action, Player]> = [];
  for (const [actionIndex, action] of handHistory.actions.entries()) {
    action.index = actionIndex;
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
  //
  if (action.action == 'Fold') {
    const neededToPutIn = action.current_amt_to_call - action.amount_put_in_pot_this_round;
    const callEquity = action.current_amt_to_call / (action.current_amt_to_call + action.pot);
    return (
      `Folds to   : ${formatEquity(neededToPutIn / action.pot)} of pot\n` +
      formatActionTextCommon(callEquity, action)
    );
  } else if (action.action == 'Check') {
    return `Left to act: ${action.players_left_to_act}`;
  } else if ('Call' in action.action) {
    const callAmount = action.action.Call;
    const callEquity = callAmount / (callAmount + action.pot);
    return (
      `Calling    : ${formatEquity(callAmount / action.pot)} of pot\n` +
      formatActionTextCommon(callEquity, action)
    );
  } else if ('Bet' in action.action) {
    const betAmount = action.action.Bet;
    const betEquity = betAmount / (betAmount + action.pot);
    return (
      `Bet        : ${formatEquity(betAmount / action.pot)} of pot\n` +
      formatActionTextCommon(betEquity, action)
    );
  } else if ('Raise' in action.action) {
    const increase = action.action.Raise[0];
    const raiseAmount = action.action.Raise[1];
    const amountPutIn = raiseAmount - action.amount_put_in_pot_this_round;
    const raiseEquity = amountPutIn / (amountPutIn + action.pot);
    return (
      `Raise      : ${formatEquity(amountPutIn / action.pot)} of pot\n` +
      formatActionTextCommon(raiseEquity, action)
    );
  } else {
    return `Unknown action ${action.action}`;
  }
}

function formatActionTextCommon(equity: number, action: Action) {
  return (
    `Pot        : ${formatChips(action.pot)}\n` +
    `Pot equity : ${formatEquity(equity)}\n` +
    `# to act   : ${action.players_left_to_act}\n` +
    `Non folded : ${action.non_folded_players}`
  );
}

function formatChips(chips: number): string {
  const bb = hand_history.value!.bb;
  return `${chips.toLocaleString('en-US')} (${(chips / bb).toLocaleString('en-US')}bb)`;
}

function formatEquity(equity: number): string {
  return `${(equity * 100).toFixed(2)}%`;
}

function handleAnalyzeRange(setExact: boolean, actionIndex: number) {
  if (!hand_history.value) {
    return;
  }

  const folded = hand_history.value!.players.map((_) => false);

  for (let i = 0; i < actionIndex; i++) {
    const action = hand_history.value!.actions[i];
    if (action.action == 'Fold') {
      folded[action.player_index] = true;
    }
  }

  const oHeroIndex = hand_history.value!.actions[actionIndex].player_index;
  const heroName = hand_history.value!.players[oHeroIndex].player_name;

  const nonFoldedPlayers = hand_history.value!.players.filter((_, index) => !folded[index]);
  const nonFoldedHeroIndex = nonFoldedPlayers.findIndex((player) => player.player_name == heroName);

  //Hero is always 1st position here
  if (nonFoldedHeroIndex >= 0) {
    playerStore.players[0].state = PlayerState.USE_HOLE;
    playerStore.players[0].holeCards = getCardList(nonFoldedHeroIndex, nonFoldedPlayers);
    playerStore.players[0].name = hand_history.value!.players[nonFoldedHeroIndex].player_name;
  }

  let playerStoreIndex = 0;
  let playerIndex = nonFoldedHeroIndex >= 0 ? nonFoldedHeroIndex : 0;

  const numPlayers = nonFoldedPlayers.length;

  const allRange = SELECTABLE_RANGES.find((range) => range.title == 'All')!.value;

  const otherPlayers = nonFoldedHeroIndex >= 0 ? numPlayers - 1 : numPlayers;

  //keep relatively the same order
  for (let i = 0; i < otherPlayers; i++) {
    playerStoreIndex++;
    playerIndex++;
    if (playerIndex >= numPlayers) {
      playerIndex = 0;
    }
    if (playerStoreIndex >= numPlayers) {
      playerStoreIndex = 0;
    }

    console.log(
      `Player history index: ${playerIndex} store index: ${playerStoreIndex}`,
      playerStore.players[playerStoreIndex]
    );

    playerStore.players[playerStoreIndex].name = nonFoldedPlayers[playerIndex].player_name;
    playerStore.players[playerStoreIndex].holeCards = getCardList(playerIndex, nonFoldedPlayers);
    playerStore.updateRangeStrForPlayer(playerStoreIndex, allRange);

    if (setExact) {
      playerStore.players[playerStoreIndex].state = PlayerState.USE_HOLE;
    } else {
      playerStore.players[playerStoreIndex].state = PlayerState.USE_RANGE;
    }
  }

  //everyone else disable
  for (let psIndex = numPlayers; psIndex < playerStore.players.length; psIndex++) {
    playerStore.players[psIndex].state = PlayerState.DISABLED;
  }

  let cards = 0;
  let reserveCards = 0;

  const action = hand_history.value!.actions[actionIndex];
  if (action.round == 'Flop') {
    cards = 3;
  } else if (action.round == 'Turn') {
    cards = 4;
  } else if (action.round == 'River') {
    cards = 5;
  }

  console.log(`Cards: ${cards} Board Len: ${hand_history.value.board.length}`);

  //Set board cards
  boardStore.board.cards = hand_history.value.board.slice(0, cards).map((bCard) => {
    return bCard.index;
  });

  //Set reserve cards
  boardStore.reserveCards = hand_history.value.board.slice(cards).map((bCard) => {
    return bCard.index;
  });

  console.log(`Board cards: ${boardStore.board.cards} Reserve cards: ${boardStore.reserveCards}`);

  const routeData = router.resolve({ path: '/' });
  window.open(routeData.href, '_blank');
  //router.push({ path: '/' });
}

function getCardList(nfPlayerIndex: number, nonFoldedPlayers: Array<Player>): CardList {
  const hiCardIndex = nonFoldedPlayers[nfPlayerIndex].cards.card_hi_lo[0].index;
  const loCardIndex = nonFoldedPlayers[nfPlayerIndex].cards.card_hi_lo[1].index;

  return {
    cards: [hiCardIndex, loCardIndex],
    cardText: cardTextStr(hiCardIndex) + cardTextStr(loCardIndex)
  };
}
</script>
