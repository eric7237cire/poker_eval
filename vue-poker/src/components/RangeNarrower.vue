<template>
  <div class="root">
    Range Narrower
    <div class="instructions">
      <ol>
        <li>1. Choose a range to narrow down</li>
        <li>2. Choose ranges to evaluate against</li>
        <li>3. Choose minimum equity</li>
        <li>4. Number of iterations</li>
        <li>5. See result</li>
      </ol>
    </div>
    <div class="copy-from">
      Copy From:
      <ul>
        <li v-for="player in playersWithRanges">
          <input
            type="radio"
            :id="'from' + player.index.toString()"
            :value="player.index"
            v-model="copyFrom"
          />
          <label :for="'from' + player.index.toString()">{{ player.name }}</label>
          <div class="range">
            Range:
            {{ player.rangeStr }}
          </div>
        </li>
      </ul>
    </div>
    <div class="two-pane">
      <div class="pane-one">
        <button class="button-base button-blue" @click="handleCopyRange()">Copy</button>
        <button class="button-base button-red" @click="handleCalculate()">Calculate</button>
        <button class="button-base button-blue" @click="handleCopyResult()">Copy Result</button>
      </div>
      <div class="pane-two">
        <div class="range-to-narrow">
          <input type="radio" id="to0" :value="0" v-model="copyTo" />
          <label for="to0">Range to narrow</label>

          <RangeMiniViewer :range="narrowStore.state.rangeToNarrow.range" />
        </div>
        <div class="num-opponents">
          Number of opponents
          <input type="number" v-model="narrowStore.state.numOpponents" />
        </div>
        <div class="copy-to">
          Opponent ranges
          <ul>
            <li v-for="(oppRange, opp_index) in narrowStore.state.opponentRanges">
              <input
                type="radio"
                :id="'to' + (opp_index + 1).toString()"
                :value="1 + opp_index"
                v-model="copyTo"
              />
              <label :for="'to' + (opp_index + 1)">Opponent {{ opp_index }}</label>
              <div class="range">Range: {{ oppRange.rangeStr }}</div>
            </li>
          </ul>
        </div>
        <div class="num-simul">
          Number of simulations
          <input type="number" v-model="narrowStore.state.numSimulations" />
        </div>
        <div class="min-equity">
          Minimum equity
          <v-slider
            v-model="narrowStore.state.minEquity"
            :min="0"
            :max="1"
            :step="0.01"
            thumb-label
          ></v-slider>
        </div>
        <div class="result">
          Result:
          <div class="range">{{ narrowStore.state.result.rangeStr }}</div>
        </div>
      </div>
    </div>
    <div class="copy-results-to">
      Copy Results To:
      <ul>
        <li v-for="player in playerStore.players">
          <input
            type="radio"
            :id="'resultTo' + player.index.toString()"
            :value="player.index"
            v-model="copyResultTo"
          />
          <label :for="'resultTo' + player.index.toString()">{{ player.name }}</label>
        </li>
      </ul>
    </div>
  </div>
</template>

<style lang="postcss" scoped>
.root {
  background-color: rgba(0, 0, 255, 0.05);
  color: white;
  border-radius: 4px;

  box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);

  margin: 20px;

  display: block;
  box-sizing: border-box;

  .two-pane {
    display: grid;
    grid-template-columns: 100px 1fr;

    .pane-one {
      button {
        margin: 5px;
      }
    }

    .pane-two {
      input {
        background-color: white;
      }
      .result {
        max-width: 400px;

        .range {
          text-wrap: wrap;
          overflow: hidden;
        }
      }
    }
  }

  .copy-from,
  .copy-to,
  .range-to-narrow,
  .copy-results-to {
    label {
      margin-left: 10px;
    }
  }

  .copy-from,
  .copy-results-to {
    border: 1px solid blue;
    padding: 10px;

    .range {
      display: inline;
    }
  }
}
</style>

<script setup lang="ts">
import { useNarrowStore } from '@src/stores/narrow';
import { CurrentPage, useNavStore } from '@src/stores/navigation';
import { usePlayerStore } from '@src/stores/player';
import { computed, ref, watch } from 'vue';
import RangeMiniViewer from '@src/components/RangeMiniViewer.vue';
import * as _ from 'lodash';
import { init, handler } from '@src/worker/global-worker';
import { useBoardStore } from '@src/stores/board';
import { RangeManager } from '@pkg/range';

const playerStore = usePlayerStore();
const narrowStore = useNarrowStore();
const boardStore = useBoardStore();

const copyFrom = ref(0);
const copyTo = ref(0);
const copyResultTo = ref(0);

const playersWithRanges = computed(() => {
  return playerStore.players.filter((p) => p.rangeStr.length > 0);
});

let range: RangeManager | null = null;

initRangeManager().then(() => {
  console.log('Range initialized');
});

//below are functions only

async function initRangeManager() {
  let mod = await import('@pkg/range');
  await mod.default();

  range = RangeManager.new();
}

watch(
  () => narrowStore.state.numOpponents,
  (newVal) => {
    if (!_.isInteger(newVal) || newVal <= 0 || newVal >= 10) {
      return;
    }

    if (narrowStore.state.opponentRanges.length < newVal) {
      for (let i = narrowStore.state.opponentRanges.length; i < newVal; i++) {
        narrowStore.state.opponentRanges.push({
          range: [],
          rangeStr: '',
          percHands: 0
        });
      }
    } else {
      narrowStore.state.opponentRanges = narrowStore.state.opponentRanges.slice(0, newVal);
    }
  }
);

function handleCopyRange() {
  console.log('handleCopyRange', copyFrom, copyTo);

  const fromPlayer = playerStore.players[copyFrom.value];

  if (copyTo.value == 0) {
    narrowStore.state.rangeToNarrow.range = fromPlayer.range;
    narrowStore.state.rangeToNarrow.rangeStr = fromPlayer.rangeStr;
    narrowStore.state.rangeToNarrow.percHands = fromPlayer.percHands;
  } else {
    const toPlayer = narrowStore.state.opponentRanges[copyTo.value - 1];

    toPlayer.range = fromPlayer.range;
    toPlayer.rangeStr = fromPlayer.rangeStr;
    toPlayer.percHands = fromPlayer.percHands;
  }
}

async function handleCalculate() {
  const boardCards = Uint8Array.from(boardStore.board.cards);

  if (!handler) {
    console.log('handler not initialized');
    return;
  }

  const response = await handler.narrowRange(
    narrowStore.state.rangeToNarrow.rangeStr,
    narrowStore.state.opponentRanges.map((r) => r.rangeStr),
    narrowStore.state.minEquity,
    boardCards,
    narrowStore.state.numSimulations
  );

  narrowStore.state.result.rangeStr = response;
}

function handleCopyResult() {
  if (!range) {
    console.log('range not initialized');
    return;
  }
  const resultPlayer = playerStore.players[copyResultTo.value];

  resultPlayer.rangeStr = narrowStore.state.result.rangeStr;

  range.from_string(resultPlayer.rangeStr);
  const weights = range.get_weights();
  for (let i = 0; i < 13 * 13; ++i) {
    resultPlayer.range[i] = weights[i] * 100;
  }
}
</script>
