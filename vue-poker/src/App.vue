<template>
  <div class="ml-10">
    <img alt="Vue logo" class="logo" src="./assets/logo.svg" width="125" height="125" />
  </div>

  <ResultTable />

  <button @click="go" class="button-base button-blue" style="position: relative; left: 400px;" >Go</button>

  <div class="ml-10">
    <div v-show="navStore.currentPage === CurrentPage.RANGE_EDITOR">
      <RangeEditor />
    </div>

    <!--Show 'players'-->
    <div class="players">
      <div v-for="player in players" :key="player.id" :class="['player', player.class]">
        <Player :playerId="player.id" />
      </div>
    </div>

    <div class="flex-grow my-4 px-6 pt-2 overflow-y-auto" style="height: calc(100% - 2rem)">
      <BoardSelector v-model="boardStore.board" :expected_length="3" />
    </div>
  </div>
</template>

<style scoped src="./assets/App.css">

</style>

<script setup lang="ts">
import BoardSelector from './components/BoardSelector.vue';
import Player from './components/Player.vue';
import RangeEditor from './components/RangeEditor.vue';
import ResultTable from './components/ResultTable.vue';
import { defineComponent, onMounted } from 'vue';
import { useNavStore, CurrentPage } from './stores/navigation';
import { init, handler } from './global-worker';
import { PlayerIds, PlayerState, usePlayerStore } from './stores/player';
import { useBoardStore } from './stores/board';

const navStore = useNavStore();
const playerStore = usePlayerStore();
const boardStore = useBoardStore();

boardStore.$subscribe((board) => {
  console.log('boardStore.$subscribe', board);
  //handler!.reset(0, board);
});

onMounted(async () => {
  console.log(`the component is now mounted.`);
  await init(1);
  await handler!.reset(0, []);
});

const players = [
  { id: 0, class: 'player0' },
  { id: 1, class: 'player1' },
  { id: 2, class: 'player2' },
  { id: 3, class: 'player3' },
  { id: 4, class: 'player4' }
];

async function go() {
  if (!handler) {
    console.log('handler is not ready');
    return;
  }

  await handler.reset();
  await handler.setBoardCards(Uint8Array.from(boardStore.board.cards));
  for(let i = 0; i < playerStore.players.length; i++) {
    const player = playerStore.players[i];
    if (player.state == PlayerState.USE_HOLE && Array.isArray(player.holeCards.cards) && player.holeCards.cards.length === 2) {
      await handler.setPlayerCards(i, Uint8Array.from(player.holeCards.cards));
    }
    if (player.state == PlayerState.USE_RANGE) {
      await handler.setPlayerRange(i, player.rangeStr);
    }
    await handler.setPlayerRange(i, player.rangeStr);
    await handler.setPlayerState(i, player.state.valueOf());
  }

  await handler.simulateFlop(200);

  //const result = await handler.getResults();
  for(let i = 0; i < playerStore.players.length; i++) {
    const result = await handler.getResult(i);
    console.log(`player ${i}`, result);
    console.log(`player ${i} win rate`, result.num_iterations);
  }

  // for(const r of result) {
  //   console.log(r);
  // }
}


// playerStore.updateRangeStrForPlayer(PlayerIds.HERO, 'TT+');
// playerStore.updateRangeStrForPlayer(PlayerIds.WEST, '83+');
// playerStore.updateRangeStrForPlayer(PlayerIds.NORTH_WEST, '22+, 72+');
// playerStore.updateRangeStrForPlayer(PlayerIds.NORTH_EAST, 'A2o+, Q3o+');
</script>
