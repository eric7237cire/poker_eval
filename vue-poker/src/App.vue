<template>
  <div class="ml-10">
    <img alt="Vue logo" class="logo" src="./assets/logo.svg" width="125" height="125" />
  </div>

  <ResultTable />

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
      <BoardSelector />
    </div>
  </div>
</template>

<style scoped>
header {
  line-height: 1.5;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    place-items: center;
    padding-right: calc(var(--section-gap) / 2);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }
}

.players {
  display: grid;
  grid-gap: 10;
  grid-template-columns: repeat(8, minmax(75px, 90px));
  grid-template-rows: repeat(4, minmax(110px, 120px));
  /*display: block;*/
  margin-left: 30px;
  /*height: 300px;
  width: 600px;*/
  position: relative;

  .player {
    /*position: absolute;
    width: 150px;*/
    width: 150px;
    border: 1px solid red;
    display: inline-grid;
    margin: 10px;
  }
  .player0 {
    /*grid-column: 3 / 5;*/
    grid-column-start: 4;
    grid-column-end: 6;
    /*grid-row: 3 / 5;*/
    grid-row-start: 3;
    grid-row-end: 5;
    /*bottom: 0px;
    left: 225px;*/
  }
  .player1 {
    grid-column-start: 1;
    grid-column-end: 3;
    grid-row-start: 2;
    grid-row-end: 4;
  }
  .player2 {
    grid-column-start: 3;
    grid-column-end: 5;
    grid-row: 1 / 3;
  }
  .player3 {
    /*grid-column: 4 / 6;*/
    grid-column-start: 5;
    grid-column-end: 7;
    grid-row: 1 / 3;
  }
  .player4 {
    /*grid-column: 6 / 8;*/
    grid-column-start: 7;
    grid-column-end: 9;
    grid-row: 2 / 4;
  }
}
</style>

<script lang="ts">
import BoardSelector from './components/BoardSelector.vue';
import Player from './components/Player.vue';
import RangeEditor from './components/RangeEditor.vue';
import ResultTable from './components/ResultTable.vue';
import { defineComponent } from 'vue';
import { useNavStore, CurrentPage } from './stores/navigation';
import { init, handler } from './global-worker';
import { PlayerIds, usePlayerStore } from './stores/player';
import { useBoardStore } from './stores/board';

export default defineComponent({
  components: {
    BoardSelector,
    Player,
    RangeEditor,
    ResultTable
  },

  async mounted() {
    console.log(`the component is now mounted.`);
    await init(1);
    await handler!.reset(0, []);
  },

  setup() {
    const navStore = useNavStore();
    const playerStore = usePlayerStore();
    const boardStore = useBoardStore();

    boardStore.$subscribe((board) => {
      console.log('boardStore.$subscribe', board);
      //handler!.reset(0, board);
    });

    const players = [
      { id: 0, class: 'player0' },
      { id: 1, class: 'player1' },
      { id: 2, class: 'player2' },
      { id: 3, class: 'player3' },
      { id: 4, class: 'player4' }
    ];

    // playerStore.updateRangeStrForPlayer(PlayerIds.HERO, 'TT+');
    // playerStore.updateRangeStrForPlayer(PlayerIds.WEST, '83+');
    // playerStore.updateRangeStrForPlayer(PlayerIds.NORTH_WEST, '22+, 72+');
    // playerStore.updateRangeStrForPlayer(PlayerIds.NORTH_EAST, 'A2o+, Q3o+');

    return {
      navStore,
      CurrentPage,
      players
    };
  }
});
</script>
