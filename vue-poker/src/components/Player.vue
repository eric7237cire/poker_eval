<!--
Select a range profile or disable
or select a range
-->

<template>
  <div class="title">
    <span class="player-name">
      <input v-model="playerData.name" :placeholder="'Player ' + playerId" />
    </span>
    <div class="button-row">
      <button
        @click="handleChangeState(PlayerState.DISABLED)"
        v-if="playerData.state != PlayerState.DISABLED"
        class="button-base button-red"
      >
        Disable
      </button>
      <button
        @click="handleChangeState(PlayerState.USE_RANGE)"
        v-if="playerData.state == PlayerState.DISABLED || playerData.state == PlayerState.USE_HOLE"
        class="button-base button-blue"
      >
        Range
      </button>
      <button
        @click="handleChangeState(PlayerState.USE_HOLE)"
        v-if="playerData.state == PlayerState.DISABLED || playerData.state == PlayerState.USE_RANGE"
        class="button-base button-blue"
      >
        Hole
      </button>
    </div>
  </div>
  <RangeMiniViewer
    :playerId="playerId"
    @click="handleRangeClick()"
    v-if="playerData.state == PlayerState.USE_RANGE"
  />
  <div v-if="playerData.state == PlayerState.USE_RANGE" class="text-center">
    Range %: {{ formatNumber(100 * playerData.percHands) }}
  </div>

  <div v-if="playerData.state == PlayerState.USE_HOLE" class="board-selector">
    <BoardSelector v-model="playerData.holeCards" :expected_length="2" />
  </div>
</template>

<style lang="postcss" scoped>
div.title {
  display: flex;
  flex-direction: column;
  align-items: center;

  .player-name {
    font-size: 1rem;
    font-weight: bold;
    text-align: center;
    display: block;

    input {
      width: 90%;
      box-sizing: border-box;
      background-color: black;
      color: green;
    }
  }

  button {
    flex-grow: 0;
    @apply py-1  m-1 inline;
    padding-left: 0.5rem;
    padding-right: 0.5rem;
  }
}

.board-selector {
  display: flex;
  justify-content: center;
  align-items: center;

  div {
    flex-grow: 0;
  }
}
</style>

<script setup lang="ts">
import { computed, defineComponent, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { PlayerState, usePlayerStore } from '../stores/player';
import { Store, PiniaCustomStateProperties, storeToRefs } from 'pinia';
import RangeMiniViewer from './RangeMiniViewer.vue';
import BoardSelector from './BoardSelector.vue';

const props = defineProps({
  playerId: {
    type: Number,
    required: true
  }
});
const navStore = useNavStore();
const playerStore = usePlayerStore();

const playerData = computed(() => playerStore.playerDataForId(props.playerId));

function handleRangeClick() {
  console.log('range clicked');
  navStore.setCurrentPage(CurrentPage.RANGE_EDITOR);
  playerStore.setCurrentPlayer(props.playerId);
}

function handleChangeState(state: PlayerState) {
  console.log('handleChangeState', state);
  playerStore.players[props.playerId].state = state;
}

function formatNumber(num: number) {
  return num.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 1
  });
}
</script>
