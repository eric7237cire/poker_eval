<!--
Select a range profile or disable
or select a range
-->

<template>
  Profile Name: {{ playerId }} buh
  <div @click="handleRangeClick()">Range %: {{ formatNumber(100*playerStore.playerDataForId(playerId).percHands) }}</div>
  <button>Disabl</button>
  ps {{ playerStore.currentPlayer }}
</template>

<script lang="ts">
import { computed, defineComponent, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { usePlayerStore } from '../stores/player';
import { Store, PiniaCustomStateProperties, storeToRefs } from 'pinia';

export default defineComponent({
  props: {
    playerId: {
      type: Number,
      required: true
    }
  },

  setup(props) {
    const navStore = useNavStore();
    const playerStore = usePlayerStore();

    const handleRangeClick = () => {
      console.log('range clicked');
      navStore.setCurrentPage(CurrentPage.RANGE_EDITOR);
      playerStore.setCurrentPlayer(props.playerId);
    };

    const formatNumber = (num: number) => {
      return num.toLocaleString(undefined, {
        minimumFractionDigits: 0,
        maximumFractionDigits: 1
      });
    };

    return {
      playerId: props.playerId,
      handleRangeClick,
      playerStore,
      currentPlayer: playerStore.currentPlayer,
      formatNumber
    };
  }
});
</script>
