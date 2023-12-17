<template>
    <span>
       
        <template v-if="playerData.state == PlayerState.USE_HOLE">
            <Card v-for="card in playerData.holeCards.cards" :cardNumber="card" />
        </template>
        <template v-if="playerData.state == PlayerState.USE_RANGE">
            <span class="perc">
                {{ formatNumber(100 * playerData.percHands) }}%
            </span>
        </template>
       <span class="plus">+</span>
       <Card v-for="card in boardCards" :cardNumber="card" /> 
       
        </span>
  </template>
  <script setup lang="ts">
  import { useBoardStore } from '@src/stores/board';
import { PlayerState, usePlayerStore } from '@src/stores/player';
import { cardText } from '@src/utils';
  import { computed } from 'vue';
  import Card from '@src/components/result/Card.vue';
  
  const props = defineProps<{
    playerId: number;
  }>();
  
    const playerStore = usePlayerStore();
    const boardStore = useBoardStore();
  
    const boardCards = computed(() => {
      return boardStore.board.cards;
    });

    const playerData = computed(() => {
        return playerStore.playerDataForId(props.playerId);
    });

    PlayerState
  
    
function formatNumber(num: number) {
  return num.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 1
  });
}
  </script>
  