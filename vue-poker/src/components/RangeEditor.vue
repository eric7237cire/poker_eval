<template>
  <div>
    <div id="overlay"></div>
    <div class="root mt-1" :style="rootStyle">
      <div>Editing player # {{ currentPlayerComputed }}</div>
      <div class="flex">
        <div class="shrink-0 ml-1">
          <table class="shadow-md select-none snug" @mouseleave="dragEnd">
            <tr v-for="row in 13" :key="row" class="h-9">
              <td
                v-for="col in 13"
                :key="col"
                class="relative w-[2.625rem] border border-black"
                @mousedown="dragStart(row, col)"
                @mouseup="dragEnd"
                @mouseenter="mouseEnter(row, col)"
                :title="cellComment(row, col)"
              >
                <div
                  :class="
                    'absolute w-full h-full left-0 top-0 ' +
                    (row === col ? 'bg-neutral-700' : 'bg-neutral-800')
                  "
                >
                  <div
                    class="absolute w-full h-full left-0 top-0 bg-bottom bg-no-repeat"
                    :style="{
                      'background-image': `linear-gradient(${yellow500} 0% 100%)`,
                      'background-size': `100% ${cellValue(row, col)}%`
                    }"
                  ></div>
                </div>
                <div
                  :class="
                    'absolute -top-px left-[0.1875rem] z-10 text-shadow ' +
                    (cellValue(row, col) > 0 ? 'text-white' : 'text-neutral-500')
                  "
                >
                  {{ cellText(row, col) }}
                </div>
                <div class="absolute bottom-px right-1 z-10 text-sm text-shadow text-white">
                  {{
                    cellValue(row, col) > 0 && cellValue(row, col) < 100
                      ? cellValue(row, col).toFixed(1)
                      : ''
                  }}
                </div>
              </td>
            </tr>
          </table>

          <div class="mt-5">
            <div class="flex">
              <input
                v-model="rangeText"
                type="text"
                :class="
                  'range-text flex-grow mr-6 px-2 py-1 rounded-lg text-sm text-black ' +
                  (rangeTextError ? 'input-error' : '')
                "
                @focus="($event.target as HTMLInputElement).select()"
                @change="onRangeTextChange"
              />

              <button class="button-base button-blue" @click="clearRange">Clear</button>
              <button class="ml-3 button-base button-blue" @click="handleDone">Done</button>
            </div>

            <div v-if="rangeTextError" class="mt-1 text-red-500">Error: {{ rangeTextError }}</div>
          </div>

          <div class="flex mt-3.5 items-center">
            <div>
              Range
              <input
                v-model="percRange"
                type="range"
                class="ml-3 w-40 align-middle"
                min="0"
                max="100"
                step="1"
                @change="onPercRangeChange"
              />
              <input
                v-model="percRange"
                type="number"
                :class="
                  'range-perc-input w-20 ml-4 px-2 py-1 rounded-lg text-sm text-center text-black' +
                  (percRange < 0 || percRange > 100 ? 'input-error' : '')
                "
                min="0"
                max="100"
                step="5"
                @change="onPercRangeChange"
              />
              %
            </div>

            <span class="inline-block ml-auto">
              {{ numCombos.toFixed(1) }} combos ({{
                ((numCombos * 100) / ((52 * 51) / 2)).toFixed(1)
              }}%)
            </span>
          </div>
        </div>

        <div class="flex-grow max-w-[18rem] ml-6 item-picker">
          <DbItemPicker
            store-name="ranges"
            :value="rangeText"
            :allow-save="rangeText !== '' && rangeTextError === ''"
            @load-item="loadRange"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="postcss" scoped>
#overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 200;

  background-color: rgba(0, 0, 0, 0.98);
}
.root {
  margin-left: 10px;
  z-index: 210;
  position: relative;

  .range-text,
  .range-perc-input {
    background: white;
  }
}
</style>
<script setup lang="ts">
import { computed, defineComponent, reactive, ref, watch } from 'vue';
//import { useConfigStore } from "../store";
import { usePlayerStore } from '../stores/player';
import { ranks, rankPat } from '@src/lib/utils';
import { RangeManager } from '@pkg/range';

import { CurrentPage, useNavStore } from '../stores/navigation';

import DbItemPicker from './DbItemPicker.vue';
import { useRangesStore } from '@src/stores/ranges';

const yellow500 = '#eab308';

const comboPat = `(?:(?:${rankPat}{2}[os]?)|(?:(?:${rankPat}[cdhs]){2}))`;
const weightPat = '(?:(?:[01](\\.\\d*)?)|(?:\\.\\d+))';
const trimRegex = /\s*([-:,])\s*/g;
const rangeRegex = new RegExp(
  `^(?<range>${comboPat}(?:\\+|(?:-${comboPat}))?)(?::(?<weight>${weightPat}))?$`
);

type DraggingMode = 'none' | 'enabling' | 'disabling';

const playerStore = usePlayerStore();
const navStore = useNavStore();

const rangeText = ref('');
const rangeTextError = ref('');
const rangeArray = reactive(new Array(13 * 13).fill(0));
const rangeArrayComments = reactive(new Array(13 * 13).fill(''));

const percRange = ref(100);
const numCombos = ref(0);

const rootStyle = ref({});

const rangeStore = useRangesStore();

//This is needed because there is 1 range editor for all players
//If range editor activated, reparse range text to get full weights
watch(
  () => navStore.currentPage,
  (newValue, oldValue) => {
    console.log(`Nav from ${oldValue} to ${newValue}`);
    //const playerIndex = currentPlayer.value.valueOf();
    const p = playerStore.curPlayerData;
    // console.log(`p is ${JSON.stringify(p)}`);
    // console.log(`range text is set to [ ${p.rangeStr} ]`);
    rangeText.value = p.rangeStr;
    onRangeTextChange();
  }
);

watch(
  () => navStore.rangeEditorTryTopY,
  (newValue, oldValue) => {
    console.log(`The re top changed from ${oldValue} to ${newValue}`);
    positionEditor(newValue);
  }
);

const currentPlayerComputed = computed(() => {
  let cp = playerStore.currentPlayer;
  console.log(`Current player is now ${cp}`);
  return cp;
});

let draggingMode: DraggingMode = 'none';

//private local to update some stats

//below are functions only

function cellText(row: number, col: number) {
  const r1 = 13 - Math.min(row, col);
  const r2 = 13 - Math.max(row, col);
  return ranks[r1] + ranks[r2] + ['s', '', 'o'][Math.sign(row - col) + 1];
}

const cellIndex = (row: number, col: number) => {
  return 13 * (row - 1) + col - 1;
};

function cellValue(row: number, col: number) {
  return rangeArray[cellIndex(row, col)];
}

function cellComment(row: number, col: number) {
  return rangeArrayComments[cellIndex(row, col)];
}

function onUpdate() {
  const range = playerStore.range;
  if (!range) {
    console.log('range is not ready');
    return;
  }

  //rangeStoreRaw.set();
  rangeText.value = range.to_string();
  playerStore.updateRangeStr(rangeText.value);
  rangeTextError.value = '';
  const rawData = range.raw_data();
  numCombos.value = rawData.reduce((acc, cur) => acc + cur, 0);

  percRange.value = Math.round((numCombos.value / ((52 * 51) / 2)) * 100);
}

function update(row: number, col: number, enabled: boolean) {
  const range = playerStore.range;
  if (!range) {
    console.log('range is not ready');
    return;
  }
  const idx = 13 * (row - 1) + col - 1;
  range.update(row, col, enabled);
  rangeArray[idx] = enabled ? 100 : 0;
  onUpdate();
}

function onRangeTextChange() {
  const range = playerStore.range;
  if (!range) {
    console.log('range is not ready');
    return;
  }

  const trimmed = rangeText.value.replace(trimRegex, '$1').trim();
  const ranges = trimmed.split(',');

  if (ranges[ranges.length - 1] === '') {
    ranges.pop();
  }

  for (const range of ranges) {
    if (!rangeRegex.test(range)) {
      rangeTextError.value = `Failed to parse range: ${range || '(empty string)'}`;
      return;
    }
  }

  range.from_string(trimmed);

  const weights = range.get_weights();
  for (let i = 0; i < 13 * 13; ++i) {
    rangeArray[i] = weights[i] * 100;
    const col = i % 13;
    const row = i / 13;
    rangeArrayComments[i] = range.get_partial_comment(row, col);
  }
  onUpdate();
}

const dragStart = (row: number, col: number) => {
  const idx = 13 * (row - 1) + col - 1;

  if (rangeArray[idx] <= 0) {
    draggingMode = 'enabling';
    update(row, col, true);
  } else {
    draggingMode = 'disabling';
    update(row, col, false);
  }
};

const dragEnd = () => {
  draggingMode = 'none';
};

const mouseEnter = (row: number, col: number) => {
  if (draggingMode === 'enabling') {
    update(row, col, true);
  } else if (draggingMode === 'disabling') {
    update(row, col, false);
  }
};

function onPercRangeChange() {
  percRange.value = Math.round(Math.max(0, Math.min(100, percRange.value)));

  const mwRanks = rangeStore.multiway_ranges;
  let takeN = Math.round((percRange.value * mwRanks.length) / 100);

  console.log(`takeN is ${takeN}`);

  const rStrParts = [];
  for (let i = 0; i < takeN; i++) {
    rStrParts.push(mwRanks[i]);
  }

  const rStr = rStrParts.join(', ');

  rangeText.value = rStr;
  onRangeTextChange();
}

const clearRange = () => {
  const range = playerStore.range;
  if (!range) {
    console.log('range is not ready');
    return;
  }

  range.clear();
  rangeArray.fill(0);
  rangeText.value = '';
  rangeTextError.value = '';
  numCombos.value = 0;
  percRange.value = 0;
  playerStore.updateRangeStr('');
};

const loadRange = (rangeStr: unknown) => {
  rangeText.value = String(rangeStr);
  onRangeTextChange();
};

const handleDone = () => {
  navStore.currentPage = CurrentPage.MAIN;
};

function positionEditor(y_coord: number) {
  const editorHeight = 600;

  let top = y_coord - editorHeight / 2;

  if (top < 0) {
    top = 0;
  }
  if (top + editorHeight > window.innerHeight) {
    top -= top + editorHeight - window.innerHeight;
  }

  rootStyle.value = {
    position: 'fixed',
    left: `0px`,
    top: `${top}px`
  };
}
</script>
../lib/utils
