<template>
  <div class="user-messages">
    {{ userMessage }}
  </div>
  <div class="results-table-container">
    <Transition>
      <ResultTable
        :results="myResultsList"
        :equityOnly="equityOnly"
        v-if="myResultsList.length > 0"
      />
    </Transition>
  </div>

  <div class="go-row-container">
    <div class="go-row">
      <input type="checkbox" id="checkbox" v-model="equityOnly" />
      <label for="checkbox">Equity Only</label>
      <button @click="go" class="button-base button-green">Go</button>
      <button @click="stop" class="button-base button-red">Stop</button>
      <button @click="load" class="button-base button-blue">Load</button>
      <div class="status">{{ num_iterations }} Iterations</div>
    </div>
  </div>

  <div class="">
    <div class="board-selector-container" style="height: calc(100% - 2rem)">
      <BoardSelector
        class="child"
        v-model="boardStore.board"
        :max_expected_length="5"
        :min_expected_length="3"
      />
      <v-btn v-if="boardStore.board.cards.length > 3" @click="handleStashCard()">
        <!--right-->
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="w-6 h-6"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M17.25 8.25 21 12m0 0-3.75 3.75M21 12H3"
          />
        </svg>
      </v-btn>

      <v-btn
        v-if="boardStore.reserveCards.length > 0 && boardStore.board.cards.length != 4"
        @click="handleUnstashCard()"
      >
        <!--left -->
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="w-6 h-6"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6.75 15.75 3 12m0 0 3.75-3.75M3 12h18"
          />
        </svg>
      </v-btn>
      <BoardSelectorCard
        v-for="card in boardStore.reserveCards"
        :key="card"
        class="m-1"
        :card-id="card"
      />
      <v-btn v-if="boardStore.reserveCards.length > 0" @click="handleUnstashCard()">
        <!--left -->
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="w-6 h-6"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6.75 15.75 3 12m0 0 3.75-3.75M3 12h18"
          />
        </svg>
      </v-btn>
    </div>

    <!-- This pops up if we are editing a range -->
    <div v-show="navStore.currentPage === CurrentPage.RANGE_EDITOR">
      <RangeEditor />
    </div>

    <!--Show 'players'-->
    <div class="players-container">
      <div class="players">
        <div v-for="player in playerStore.players" :key="player.index" class="player">
          <Player :playerId="player.index" />
        </div>
      </div>
    </div>

    <!--Range Narrower-->
    <RangeNarrower />

    <div class="footer-container">
      <Suspense>
        <template v-slot:default>
          <Footer />
        </template>
        <template v-slot:fallback>
          <div>Loading...</div>
        </template>
      </Suspense>
    </div>
  </div>
</template>

<style scoped src="./assets/App.css"></style>

<style lang="postcss">
/*use non scoped styles*/
.board-selector-container div {
  grid-column-start: 4;
  grid-column-end: 6;
}

.results-table-container {
  width: 100vw;
  box-sizing: border-box;
}

.fade-in {
  opacity: 1;
  animation-name: fadeInOpacity;
  animation-iteration-count: 1;
  animation-timing-function: ease-in;
  animation-duration: 2s;
}

@keyframes fadeInOpacity {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}

.v-enter-active,
.v-leave-active {
  transition: opacity 0.5s ease;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}
</style>

<script setup lang="ts">
import BoardSelector from './components/BoardSelector.vue';
import Player from './components/Player.vue';
import RangeEditor from './components/RangeEditor.vue';
import ResultTable from './components/ResultTable.vue';
import { Ref, computed, defineComponent, onMounted, ref } from 'vue';
import { useNavStore, CurrentPage } from './stores/navigation';
import { init, handler } from './worker/global-worker';
import { PlayerState, usePlayerStore } from './stores/player';
import { useBoardStore } from './stores/board';
import { useResultsStore } from './stores/results';
import { useRangesStore } from './stores/ranges';
import RangeNarrower from './components/RangeNarrower.vue';
import { useCssVar } from '@vueuse/core';
import Footer from './components/Footer.vue';
import BoardSelectorCard from './components/BoardSelectorCard.vue';
import { loadCardsFromUrl, parseCardString } from './lib/utils';
import * as _ from 'lodash';

const navStore = useNavStore();
const playerStore = usePlayerStore();
const boardStore = useBoardStore();
const resultsStore = useResultsStore();
const rangeStore = useRangesStore();

const pauseAfterTickMs = 500;

const equityOnly = ref(true);

const iterationsPerTick = computed(() => {
  if (equityOnly.value) {
    return 25_000;
  } else {
    return 1_000;
  }
});

const maxIterations = computed(() => {
  if (equityOnly.value) {
    return 500_000;
  } else {
    return 50_000;
  }
});

boardStore.$subscribe((board) => {
  console.log('boardStore.$subscribe', board);
  //handler!.reset(0, board);
});

resultsStore.$subscribe((results) => {
  console.log('resultsStore.$subscribe', results);
});

const myResultsList = computed(() => resultsStore.results);

onMounted(() => {
  console.log(`the component is now mounted.`);
  //init the worker
  init(1).then(() => {
    console.log('worker initialized');
  });

  //fetch the ranges
  rangeStore.init_ranges().then(() => {
    console.log('ranges fetched');
  });

  const board = loadCardsFromUrl('board');
  if (board) {
    console.log('board loaded from url', board);
    boardStore.board = board;
  } else {
    console.log('no board loaded from url');
  }

  const heroCards = loadCardsFromUrl('hero');
  if (heroCards) {
    console.log('hero cards loaded from url', heroCards);
    playerStore.playerDataForId(0).holeCards = heroCards;
  } else {
    console.log('no hero cards loaded from url');
  }
});

const userMessage = ref(
  'Welcome, to get started, choose the flop, optionally the turn and river.  Then either specify the exact hole cards or a range for the players you want to simulate'
);

const num_iterations = ref(0);
const setTimeoutReturn: Ref<NodeJS.Timeout | null> = ref(null);
let stopping = false;

const el = ref(null);
const color = useCssVar('--playerWidth', el);
color.value = '150px';

async function go() {
  if (!handler) {
    console.log('handler is not ready');
    return;
  }

  await handler.reset();
  await handler.setBoardCards(Uint8Array.from(boardStore.board.cards));

  let activePlayerCount = 0;

  for (let i = 0; i < playerStore.players.length; i++) {
    const player = playerStore.players[i];
    if (player.state == PlayerState.USE_HOLE) {
      if (Array.isArray(player.holeCards.cards) && player.holeCards.cards.length === 2) {
        await handler.setPlayerCards(i, Uint8Array.from(player.holeCards.cards));
        activePlayerCount += 1;
      } else {
        userMessage.value = `Missing hole cards for player ${i}.  Either specify cards or click 'Off'`;
        return;
      }
    }
    if (player.state == PlayerState.USE_RANGE) {
      await handler.setPlayerRange(i, player.rangeStr);
      activePlayerCount += 1;
    }
    //await handler.setPlayerRange(i, player.rangeStr);
    await handler.setPlayerState(i, player.state.valueOf());
  }

  if (activePlayerCount < 2) {
    userMessage.value =
      'You must specify at least 2 players.  Click Hole to select exact hole cards or Range to select a range';
    return;
  }

  num_iterations.value = 0;
  userMessage.value = `Simulating until ${maxIterations.value} or Stop is clicked...`;

  const resultsOk = await handler.initResults();

  if (!resultsOk) {
    userMessage.value = `Error initializing results`;
    return;
  }

  setTimeoutReturn.value = setTimeout(() => tick(50), 100);
  stopping = false;
}

async function tick(numIterations: number) {
  if (!handler) {
    console.log('handler is not ready');
    return;
  }
  num_iterations.value = num_iterations.value + numIterations;

  if (num_iterations.value >= maxIterations.value) {
    console.log(`max iterations reached ${maxIterations} > ${num_iterations.value}`);
    userMessage.value = ``;
    stopping = true;
    return;
  }

  const ok = await handler.simulateFlop(numIterations, equityOnly.value);

  if (!ok) {
    userMessage.value = `Error simulating flop`;
    stopping = true;
    return;
  }

  const resultList = await handler.getResults();

  for (const [rIdx, r] of resultList.entries()) {
    //console.log(r.rank_family_count);
    console.log(r);
  }

  if (stopping) {
    return;
  }

  resultsStore.results = resultList;

  setTimeoutReturn.value = setTimeout(() => {
    tick(iterationsPerTick.value);
  }, pauseAfterTickMs);
}

async function stop() {
  stopping = true;

  //num_iterations.value = 0;
  userMessage.value = ``;

  //resultsStore.results = [];

  if (setTimeoutReturn.value) {
    console.info('clearTimeout');
    clearTimeout(setTimeoutReturn.value);
    setTimeoutReturn.value = null;
  } else {
    console.warn('Timeout is null');
  }
}

function load() {
  fetch(`/src/assets/live.json`)
  .then((response) => response.json())
  .then((data) => {
    console.log(data);
    const holeCardsStrings = data.hole_cards.split(" ") as Array<string>;
    const holeCards = holeCardsStrings.map(s => parseCardString(s)!)
    const boardCardStrings = data.board_cards.split(" ") as Array<string>;
    const boardCards = boardCardStrings.map(s => parseCardString(s)!)

    playerStore.playerDataForId(0).holeCards = {
      cardText: data.hole_cards,
      cards: holeCards
    };
      
    boardStore.board = {
      cardText: data.board_cards,
      cards: boardCards
    };
  }); 

}

function handleStashCard() {
  const card = boardStore.board.cards.pop();
  if (_.isInteger(card)) {
    boardStore.reserveCards.unshift(card!);
  }
}

function handleUnstashCard() {
  const card = boardStore.reserveCards.shift();
  if (_.isInteger(card)) {
    boardStore.board.cards.push(card!);
  }
}
</script>
./lib/utils
