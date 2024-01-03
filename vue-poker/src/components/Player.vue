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
      <button
        @click="handleNarrowRange()"
        v-if="playerData.state == PlayerState.USE_RANGE"
        class="button-base button-green"
      >
        Narrow
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
    <BoardSelector
      v-model="playerData.holeCards"
      :max_expected_length="2"
      :min_expected_length="2"
    />
  </div>

  <div class="selectRangeContainer">
    <v-select
      v-if="playerData.state == PlayerState.USE_RANGE"
      label="Common Ranges"
      v-model="selectedRange"
      @update:model-value="handleSelectedRangeUpdate"
      :items="selectableRanges"
    ></v-select>
  </div>

  <div class="undo-redo" v-if="playerData.state == PlayerState.USE_RANGE && playerData.rangeStrHistory.length > 0 && playerData.historyIndex >= 0">
    <button
        @click="handleUndo()"
        v-if="playerData.historyIndex - 1 >= 0"
        class="button-base button-blue"
      >
        Undo
    </button>
    <button
        @click="handleRedo()"
        v-if="playerData.historyIndex + 1 < playerData.rangeStrHistory.length"
        class="button-base button-green"
      >
        Redo
    </button>
    <button
        @click="handleClearHistory()"
        v-if="playerData.rangeStrHistory.length > 0 "
        class="button-base button-red"
      >
        Clear
    </button>
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

.undo-redo {
  button {
    margin-left: 5px;
  }
}
</style>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';
import { PlayerState, usePlayerStore } from '../stores/player';
import RangeMiniViewer from './RangeMiniViewer.vue';
import BoardSelector from './BoardSelector.vue';
import * as _ from 'lodash';
import { handler } from '@src/worker/global-worker';
import { useNarrowStore } from '@src/stores/narrow';
import { useBoardStore } from '@src/stores/board';
import { SELECTABLE_RANGES } from '@src/stores/ranges';

const selectedRange = ref('');

const props = defineProps({
  playerId: {
    type: Number,
    required: true
  }
});
const navStore = useNavStore();
const playerStore = usePlayerStore();
const narrowStore = useNarrowStore();
const boardStore = useBoardStore();
const playerData = computed(() => playerStore.playerDataForId(props.playerId));
const selectableRanges = SELECTABLE_RANGES;

if (selectableRanges.find((r) => r.value == playerData.value.rangeStr)) {
  selectedRange.value = playerData.value.rangeStr;
}

//Any updates to player range resets combo box
watch(
  () => playerData.value.rangeStr,
  (newPlayerRangeString) => {
    // const check = playerData.value.range.filter((r) => r > 0).length;
    // console.log(
    //   `Player ${props.playerId} updated range to ${newPlayerRangeString}; check ${check}`
    // );
    if (!selectableRanges.find((r) => r.value == newPlayerRangeString)) {
      selectedRange.value = '';
      return;
    } else {
      selectedRange.value = newPlayerRangeString;
    }
  }
);

//When not a reactive property, can watch directly
// watch(selectedRange, (wSelRange) => {
//   console.log(`Player ${props.playerId} selected range ${wSelRange}`);
//   if (!_.isString(wSelRange) || wSelRange.length <= 0) {
//     return;
//   }
//   playerStore.updateRangeStrForPlayer(props.playerId, wSelRange, true);
// });

//We use @update specifically because we don't want this being called when
//for example undo/redo is changing the player range
function handleSelectedRangeUpdate(newRange: string) {
  console.log(`handleSelectedRangeUpdate ${newRange}`);
  if (!_.isString(newRange) || newRange.length <= 0) {
    return;
  }
  playerStore.updateRangeStrForPlayer(props.playerId, newRange, true);
}

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

function handleUndo() {
  const pData = playerStore.players[props.playerId];
  
  pData.historyIndex -= 1;

  console.log(`handleUndo historyIndex=${pData.historyIndex} len =${pData.rangeStrHistory.length}`);

  playerStore.updateRangeStrForPlayer(
    props.playerId,
    pData.rangeStrHistory[pData.historyIndex],
    false
  );
}

function handleRedo() {
  const pData = playerStore.players[props.playerId];
  pData.historyIndex += 1;

  console.log(`handleRedo historyIndex=${pData.historyIndex} len =${pData.rangeStrHistory.length}`);

  playerStore.updateRangeStrForPlayer(
    props.playerId,
    pData.rangeStrHistory[pData.historyIndex],
    false
  );

  
}

function handleClearHistory() {
  const pData = playerStore.players[props.playerId];
  pData.historyIndex = -1;
  pData.rangeStrHistory = [];
}

async function handleNarrowRange() {
  if (!handler) {
    console.log('handler not initialized');
    return;
  }
  const boardCards = Uint8Array.from(boardStore.board.cards);

  console.log('handleNarrowRange', narrowStore.state);

  if (narrowStore.state.useEquity) {
    console.log('handleNarrowRange by equity', narrowStore.state.minEquity);

    const response = await handler.narrowRange(
      playerStore.players[props.playerId].rangeStr,
      narrowStore.state.opponentRanges.map((r) => r.rangeStr),
      narrowStore.state.minEquity,
      boardCards,
      narrowStore.state.numSimulations
    );

    playerStore.updateRangeStrForPlayer(props.playerId, response, true);
  } else {
    console.log('handleNarrowRange by pref', narrowStore.state.likesHandMinimum);

    const response = await handler.narrowRangeByPref(
      playerStore.players[props.playerId].rangeStr,
      narrowStore.state.likesHandMinimum,
      boardCards,
      narrowStore.state.numOpponents+1
    );

    playerStore.updateRangeStrForPlayer(props.playerId, response, true);
  }
}
</script>
