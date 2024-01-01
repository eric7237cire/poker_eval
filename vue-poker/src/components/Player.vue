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
    :range="playerData.range"
    @click="handleRangeClick($event)"
    v-if="playerData.state == PlayerState.USE_RANGE"
  />
  <div v-if="playerData.state == PlayerState.USE_RANGE" class="text-center">
    Range %: {{ formatNumber(100 * playerData.percHands) }}
  </div>

  <div v-if="playerData.state == PlayerState.USE_HOLE" class="board-selector">
    <BoardSelector v-model="playerData.holeCards" :expected_length="2" />
  </div>

  <div class="selectRangeContainer">
    <v-select
      v-if="playerData.state == PlayerState.USE_RANGE"
      label="Common Ranges"
      v-model="selectedRange"
      :items="selectableRanges"
    ></v-select>
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

.selectRangeContainer {
  width: 100%;
  box-sizing: border-box;
  color: white;
}
</style>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { PlayerState, usePlayerStore } from '../stores/player';
import RangeMiniViewer from './RangeMiniViewer.vue';
import BoardSelector from './BoardSelector.vue';
import * as _ from 'lodash';

const selectedRange = ref('');

const props = defineProps({
  playerId: {
    type: Number,
    required: true
  }
});
const navStore = useNavStore();
const playerStore = usePlayerStore();

const playerData = computed(() => playerStore.playerDataForId(props.playerId));

const selectableRanges: Array<{ title: string; value: string }> = [
  { title: '9max UTG-UTG+2', value: '77+,AJs+,AQo+,KQs,QJs,JTs' },
  { title: '9max MP1-MP2', value: '55+,A6s+,ATo+,KTs+,KQo,Q9s+,J9s+'},
  { title: '9max HJ-CO', value: '22+,A2s+,A8o+,K8s+,KTo+,Q6s+,QTo+,J7s+,JTo,T8s+,97s+,87s'},
  { title: '9max Button', value: '22+,A2+,K4s+,K6o+,Q2s+,Q6o+,J4s+,J7o+,T6s+,T9o,95s+,98o,85s+,87o,75s+,76o,64s+'},
  { title: '9max SB', value: '22+,A2+,K2+,Q2+,J2+,T2s+,T6o+,92s+,95o+,82s+,84o+,72s+,74o+,62s+,65o,53s+'},
  { title: 'All', value: '22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32'},
  //
  {
    title: 'Custom',
    value: ''
  }
];


if (selectableRanges.find((r) => r.value == playerData.value.rangeStr)) {
    
      selectedRange.value = playerData.value.rangeStr;
    }

//Any updates to player range resets combo box
watch(
  () => playerData.value.rangeStr,
  (newPlayerRangeString) => {
    const check = playerData.value.range.filter(r => r>0).length;
    console.log(`Player ${props.playerId} updated range to ${newPlayerRangeString}; check ${check}`);
    if (!selectableRanges.find((r) => r.value == newPlayerRangeString)) {
      selectedRange.value = "";
      return;
    } else {
      selectedRange.value = newPlayerRangeString;
    }
    
  }
);

//When not a reactive property, can watch directly
watch(selectedRange, (wSelRange) => {
  console.log(`Player ${props.playerId} selected range ${wSelRange}`);
  if (!_.isString(wSelRange) || wSelRange.length <= 0) {
    return;
  }
  playerStore.updateRangeStrForPlayer(props.playerId, wSelRange);
});



function handleRangeClick(event: MouseEvent) {
  console.log(`range clicked y ${event.clientY}`);
  navStore.currentPage = CurrentPage.RANGE_EDITOR;
  playerStore.currentPlayer = props.playerId;

  navStore.rangeEditorTryTopY = event.clientY;
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
