<!--
Select a range profile or disable
or select a range
-->

<template>
  <div class="title">
    <span class="inline ml-2">Player {{ playerId }}</span>
    <button class="button-base button-blue w-15 mt-2 inline ml-2">Disable</button>
  </div>
  <RangeMiniViewer :playerId="playerId" @click="handleRangeClick()" />
  <div class="text-center">
    Range %: {{ formatNumber(100 * playerStore.playerDataForId(playerId).percHands) }}
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { usePlayerStore } from '../stores/player';
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
