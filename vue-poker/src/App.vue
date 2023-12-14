<template>
  <div class="ml-10">
    <img alt="Vue logo" class="logo" src="./assets/logo.svg" width="125" height="125" />

    <div class="wrapper">
      <HelloWorld msg="You did it!" />
    </div>
  </div>

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
      board23
      <BoardSelector />
      the board
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
</style>

<script lang="ts">
//import BoardSelectorCard from './components/BoardSelectorCard.vue'
import HelloWorld from './components/HelloWorld.vue';
import BoardSelector from './components/BoardSelector.vue';
import Player from './components/Player.vue';
import RangeEditor from './components/RangeEditor.vue';
import { defineComponent } from 'vue';
import { useNavStore, CurrentPage } from './stores/navigation';
import { useRangeStore } from './stores/ranges';
import { PlayerIds, usePlayerStore } from './stores/player';

export default defineComponent({
  components: {
    HelloWorld,
    BoardSelector,
    Player,
    RangeEditor
  },

  setup() {
    const navStore = useNavStore();
    const playerStore = usePlayerStore();

    const players = [
      { id: 0, class: 'player0' },
      { id: 1, class: 'player1' },
      { id: 2, class: 'player2' },
      { id: 3, class: 'player3' },
      { id: 4, class: 'player4' }
    ];

    const rangeStore = useRangeStore();

    playerStore.players[PlayerIds.HERO].rangeStr = 'TT+';
    playerStore.players[PlayerIds.WEST].rangeStr = '22+';
    playerStore.players[PlayerIds.NORTH_WEST].rangeStr = '22+, 72+';
    playerStore.players[PlayerIds.NORTH_EAST].rangeStr = 'A2o+, Q3o+';

    return {
      navStore,
      CurrentPage,
      players
    };
  }
});
</script>
