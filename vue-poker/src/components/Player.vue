<!--
Select a range profile or disable
or select a range
-->

<template>
  <div class="title">
    <span class="inline ml-2">P {{ playerId }}</span>
    <button
      @click="handleChangeState(PlayerState.DISABLED)"
      v-if="playerData.state != PlayerState.DISABLED"
      class="button-base button-red"
    >
      Off
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
  <RangeMiniViewer
    :playerId="playerId"
    @click="handleRangeClick()"
    v-if="playerData.state == PlayerState.USE_RANGE"
  />
  <div v-if="playerData.state == PlayerState.USE_RANGE" class="text-center">
    Range %: {{ formatNumber(100 * playerData.percHands) }}
  </div>

  <div v-if="playerData.state == PlayerState.USE_HOLE">
    <BoardSelectorCard
      
      class="m-1"
      :card-id="51"
    />
    <BoardSelectorCard
      
      class="m-1"
      :card-id="0"
    />
    <div>Cards:  {{ playerData.holeCards }}</div>
  </div>

</template>

<style lang="postcss" scoped>
div.title {
  display: flex;
  flex-direction: row;
  align-items: center;

  span {
    flex-grow: 1;
  }
  button {
    flex-grow: 0;
    @apply py-1  m-1 inline;
    padding-left: 0.5rem;
    padding-right: 0.5rem;
  }
}
</style>

<script lang="ts">
import { computed, defineComponent, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { PlayerState, usePlayerStore } from '../stores/player';
import { Store, PiniaCustomStateProperties, storeToRefs } from 'pinia';
import RangeMiniViewer from './RangeMiniViewer.vue';

export default defineComponent({
  props: {
    playerId: {
      type: Number,
      required: true
    }
  },

  components: {
    RangeMiniViewer
  },

  setup(props) {
    const navStore = useNavStore();
    const playerStore = usePlayerStore();

    const playerData = computed(() => playerStore.playerDataForId(props.playerId));

    return {
      playerId: props.playerId,
      handleRangeClick,
      handleChangeState,
      playerStore,
      currentPlayer: playerStore.currentPlayer,
      formatNumber,
      playerData,
      PlayerState
    };

    function handleRangeClick() {
      console.log('range clicked');
      navStore.setCurrentPage(CurrentPage.RANGE_EDITOR);
      playerStore.setCurrentPlayer(props.playerId);
    }

    function handleChangeState(state: PlayerState) {
      console.log('handleChangeState', state);
      playerStore.players[props.playerId].state = state;
    }
  }
});

function formatNumber(num: number) {
  return num.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 1
  });
}
</script>
