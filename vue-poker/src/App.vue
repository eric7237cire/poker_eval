<template>
  <div class="results-table-container">
    <Transition>
      <ResultTable :results="myResultsList" v-if="myResultsList.length > 0" />
    </Transition>
  </div>

  <div class="go-row-container">
    <div class="go-row">
      <button @click="go" class="button-base button-blue">Go</button>
      <button @click="stop" class="button-base button-red">Stop</button>
      <div class="status">{{ num_iterations }} Iterations</div>
    </div>
  </div>

  <div class="ml-10">
    <div class="board-selector-container" style="height: calc(100% - 2rem)">
      <BoardSelector class="child" v-model="boardStore.board" :expected_length="3" />
    </div>

    <!-- This pops up if we are editing a range -->
    <div v-show="navStore.currentPage === CurrentPage.RANGE_EDITOR">
      <RangeEditor />
    </div>

    <!--Show 'players'-->
    <div class="players">
      <div v-for="player in players" :key="player.id" :class="['player', player.class]">
        <Player :playerId="player.id" />
      </div>
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
import { PlayerIds, PlayerState, usePlayerStore } from './stores/player';
import { useBoardStore } from './stores/board';
import { useResultsStore } from './stores/results';
import { useRangesStore } from './stores/ranges';
import { useCssVar } from '@vueuse/core';

const navStore = useNavStore();
const playerStore = usePlayerStore();
const boardStore = useBoardStore();
const resultsStore = useResultsStore();
const rangeStore = useRangesStore();

const iterationsPerTick = 1_000;
const maxIterations = 50_000;

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
    console.log("worker initialized");
  });

  //fetch the ranges
  rangeStore.init_ranges().then(() => {
    console.log("ranges fetched");
  });
});

const players = [
  { id: 0, class: 'player0' },
  { id: 1, class: 'player1' },
  { id: 2, class: 'player2' },
  { id: 3, class: 'player3' },
  { id: 4, class: 'player4' }
];

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
  for (let i = 0; i < playerStore.players.length; i++) {
    const player = playerStore.players[i];
    if (
      player.state == PlayerState.USE_HOLE &&
      Array.isArray(player.holeCards.cards) &&
      player.holeCards.cards.length === 2
    ) {
      await handler.setPlayerCards(i, Uint8Array.from(player.holeCards.cards));
    }
    if (player.state == PlayerState.USE_RANGE) {
      await handler.setPlayerRange(i, player.rangeStr);
    }
    //await handler.setPlayerRange(i, player.rangeStr);
    await handler.setPlayerState(i, player.state.valueOf());
  }
  num_iterations.value = 0;

  await handler.initResults();

  setTimeoutReturn.value = setTimeout(() => tick(50), 100);
  stopping = false;
}

async function tick(numIterations: number = iterationsPerTick) {
  if (!handler) {
    console.log('handler is not ready');
    return;
  }
  num_iterations.value = num_iterations.value + numIterations;

  if (num_iterations.value >= maxIterations) {
    console.log(`max iterations reached ${maxIterations} > ${num_iterations.value}`);
    stopping = true;
    return;
  }

  await handler.simulateFlop(iterationsPerTick);

  const resultList = await handler.getResults();

  for (const [rIdx, r] of resultList.entries()) {
    //console.log(r.rank_family_count);
    console.log(r);
  }

  if (stopping) {
    return;
  }

  resultsStore.results = resultList;

  //resultList[0].equity = num_iterations.value / maxIterations;

  setTimeoutReturn.value = setTimeout(tick, 100);

  // for(let i = 0; i < playerStore.players.length; i++) {
  //   const result = await handler.getResult(i);
  //   console.log(`player ${i}`, result);
  //   console.log(`player ${i} win rate`, result.win_eq);
  // }

  // for(const r of result) {
  //   console.log(r);
  // }
}

async function stop() {
  stopping = true;

  num_iterations.value = 0;

  resultsStore.results = [];

  if (setTimeoutReturn.value) {
    console.info('clearTimeout');
    clearTimeout(setTimeoutReturn.value);
    setTimeoutReturn.value = null;
  } else {
    console.warn('Timeout is null');
  }
}

// playerStore.updateRangeStrForPlayer(PlayerIds.HERO, 'TT+');
// playerStore.updateRangeStrForPlayer(PlayerIds.WEST, '83+');
// playerStore.updateRangeStrForPlayer(PlayerIds.NORTH_WEST, '22+, 72+');
// playerStore.updateRangeStrForPlayer(PlayerIds.NORTH_EAST, 'A2o+, Q3o+');
</script>
