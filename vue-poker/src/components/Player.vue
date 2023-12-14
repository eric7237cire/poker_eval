<!--
Select a range profile or disable
or select a range
-->

<template>
  Profile Name: {{ playerId }} buh
  <div @click="handleRangeClick()">Range: {{ playerStore.playerDataForId(playerId).rangeStr }}</div>
  <button>Disabl</button>
  ps {{ playerStore.currentPlayer }}
</template>

<script lang="ts">
import { computed, defineComponent, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { useRangeStore } from '../stores/ranges';
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
    const rangeStore = useRangeStore();
    const playerStore = usePlayerStore();

    const handleRangeClick = () => {
      console.log('range clicked');
      navStore.setCurrentPage(CurrentPage.RANGE_EDITOR);
      playerStore.setCurrentPlayer(props.playerId);
    };

    return {
      playerId: props.playerId,
      handleRangeClick,
      rangeStore,
      playerStore,
      currentPlayer: playerStore.currentPlayer
    };
  }
});
</script>
