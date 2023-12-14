<template>
  <div class="flex mt-1">
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
              'flex-grow mr-6 px-2 py-1 rounded-lg text-sm text-black ' +
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
          Weight:
          <input
            v-model="weight"
            type="range"
            class="ml-3 w-40 align-middle"
            min="0"
            max="100"
            step="5"
            @change="onWeightChange"
          />
          <input
            v-model="weight"
            type="number"
            :class="
              'w-20 ml-4 px-2 py-1 rounded-lg text-sm text-center text-black' +
              (weight < 0 || weight > 100 ? 'input-error' : '')
            "
            min="0"
            max="100"
            step="5"
            @change="onWeightChange"
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

    <div class="flex-grow max-w-[18rem] ml-6">
      <DbItemPicker
        store-name="ranges"
        :index="playerIndex"
        :value="rangeText"
        :allow-save="rangeText !== '' && rangeTextError === ''"
        @load-item="loadRange"
      />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, reactive, ref, watch } from 'vue';
//import { useConfigStore } from "../store";
import { usePlayerStore } from '../stores/player';
import { ranks, rankPat } from '../utils';
import { RangeManager } from '../../../ui/pkg/range/range';
import { useRangeStore } from '../stores/ranges';
import { CurrentPage, useNavStore } from '../stores/navigation';

import DbItemPicker from './DbItemPicker.vue';
import { Store, PiniaCustomStateProperties, storeToRefs } from 'pinia';

const yellow500 = '#eab308';

const comboPat = `(?:(?:${rankPat}{2}[os]?)|(?:(?:${rankPat}[cdhs]){2}))`;
const weightPat = '(?:(?:[01](\\.\\d*)?)|(?:\\.\\d+))';
const trimRegex = /\s*([-:,])\s*/g;
const rangeRegex = new RegExp(
  `^(?<range>${comboPat}(?:\\+|(?:-${comboPat}))?)(?::(?<weight>${weightPat}))?$`
);

type DraggingMode = 'none' | 'enabling' | 'disabling';

export default defineComponent({
  components: {
    DbItemPicker
  },

  setup() {
    //const config = useConfigStore();
    //const rangeStore = useRangeStore();
    const playerStore = usePlayerStore();
    const navStore = useNavStore();

    const range = RangeManager.new();
    //const rangeStore = config.range[props.player];
    //const rangeStoreRaw = config.rangeRaw[props.player];
    const rangeText = ref('');
    const rangeTextError = ref('');
    //const rangeStore = ref(new Array(13 * 13).fill(0));
    const rangeStore = reactive(new Array(13 * 13).fill(0));
    let rangeStoreRaw = new Float32Array();
    const weight = ref(100);
    const numCombos = ref(0);

    const { currentPlayer } = storeToRefs(playerStore);

    watch(currentPlayer, (newValue, oldValue) => {
      console.log(`The re cp changed from ${oldValue} to ${newValue}`);
      const playerIndex = currentPlayer.value.valueOf();
      const p = playerStore.curPlayerData;
      console.log(`p is ${JSON.stringify(p)}`);
      console.log(`range text is set to [ ${p.rangeStr} ]`);
      rangeText.value = p.rangeStr;
      onRangeTextChange();
      //rangeStore = range.
      //rangeText.value = rangeStore.getRangeStr(playerIndex);
    });

    // const { getRangeStr } = storeToRefs(rangeStore);

    // watch(getRangeStr, (newValue, oldValue) => {
    //   console.log(`The re range changed from ${oldValue} to ${newValue}`);
    //   const playerIndex = currentPlayer.value.valueOf();
    //   rangeText.value = rangeStore.getRangeStr(playerIndex);
    // })

    const playerIndex = currentPlayer.value.valueOf();
    // rangeText.value = rangeStore.getRangeStr(playerIndex);

    // const { rangeStrs } = storeToRefs(rangeStore);

    console.log(`Setting up range editor for player ${playerIndex}`);
    console.log(`rangeText.value is ${rangeText.value}`);

    let draggingMode: DraggingMode = 'none';

    const cellText = (row: number, col: number) => {
      const r1 = 13 - Math.min(row, col);
      const r2 = 13 - Math.max(row, col);
      return ranks[r1] + ranks[r2] + ['s', '', 'o'][Math.sign(row - col) + 1];
    };

    const cellIndex = (row: number, col: number) => {
      return 13 * (row - 1) + col - 1;
    };

    const cellValue = (row: number, col: number) => {
      return rangeStore[cellIndex(row, col)];
    };

    const onUpdate = () => {
      const rawData = range.raw_data();
      rangeStoreRaw = rawData;
      //rangeStoreRaw.set();
      rangeText.value = range.to_string();
      playerStore.updateRangeStr(rangeText.value);
      rangeTextError.value = '';
      numCombos.value = rawData.reduce((acc, cur) => acc + cur, 0);
    };

    const update = (row: number, col: number, weight: number) => {
      const idx = 13 * (row - 1) + col - 1;
      range.update(row, col, weight / 100);
      rangeStore[idx] = weight;
      onUpdate();
    };

    const onRangeTextChange = () => {
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

      const errorString = range.from_string(trimmed);

      if (errorString) {
        rangeTextError.value = errorString;
      } else {
        const weights = range.get_weights();
        for (let i = 0; i < 13 * 13; ++i) {
          rangeStore[i] = weights[i] * 100;
        }
        onUpdate();
      }
    };

    const dragStart = (row: number, col: number) => {
      const idx = 13 * (row - 1) + col - 1;

      if (rangeStore[idx] !== weight.value) {
        draggingMode = 'enabling';
        update(row, col, weight.value);
      } else {
        draggingMode = 'disabling';
        update(row, col, 0);
      }
    };

    const dragEnd = () => {
      draggingMode = 'none';
    };

    const mouseEnter = (row: number, col: number) => {
      if (draggingMode === 'enabling') {
        update(row, col, weight.value);
      } else if (draggingMode === 'disabling') {
        update(row, col, 0);
      }
    };

    const onWeightChange = () => {
      weight.value = Math.round(Math.max(0, Math.min(100, weight.value)));
    };

    const clearRange = () => {
      range.clear();
      rangeStore.fill(0);
      rangeStoreRaw.fill(0);
      rangeText.value = '';
      rangeTextError.value = '';
      weight.value = 100;
      numCombos.value = 0;
      playerStore.updateRangeStr("");
    };

    const loadRange = (rangeStr: unknown) => {
      rangeText.value = String(rangeStr);
      onRangeTextChange();
    };

    const handleDone = () => {
      navStore.currentPage = CurrentPage.MAIN;
    };

    return {
      yellow500,
      cellText,
      cellValue,
      rangeStore,
      playerIndex,
      rangeText,
      rangeTextError,
      weight,
      numCombos,
      onRangeTextChange,
      dragStart,
      dragEnd,
      mouseEnter,
      onWeightChange,
      clearRange,
      loadRange,
      handleDone
    };
  }
});
</script>
