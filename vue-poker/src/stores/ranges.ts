// stores/counter.js
import { defineStore } from 'pinia';
import { RangeManager } from '../../../ui/pkg/range/range';

const MAX_PLAYERS = 5;

function buildInitialRange() {
  const range = [];
  for (let i = 0; i < MAX_PLAYERS; i++) {
    range.push(Array.from({ length: 13 * 13 }, () => 0));
  }

  return range;
}

const rangeMgr = RangeManager.new();
const trimRegex = /\s*([-:,])\s*/g;

function buildInitialRangeRaw() {
  const range = [];
  for (let i = 0; i < MAX_PLAYERS; i++) {
    range.push(Float32Array.from({ length: (52 * 51) / 2 }, () => 0));
  }
  return range;
}

function buildInitialRangeStrs(): Map<number, string> {
  const range = new Map<number, string>();
  for (let i = 0; i < MAX_PLAYERS; i++) {
    range.set(i, '');
  }
  return range;
}

function buildInitialRangeStrs2(): { [key: string]: string } {
  const range: { [key: string]: string } = {};
  for (let i = 0; i < MAX_PLAYERS; i++) {
    range[i.toString()] = '';
  }
  return range;
}

export const useRangeStore = defineStore('range', {
  state: () => {
    return {
      range: buildInitialRange(),
      rangeRaw: buildInitialRangeRaw(),
      rangeStrs: buildInitialRangeStrs2()
    };
  },
  getters: {
    // rangeValue(state, index: number): number {
    //     return state.range[index];
    // }
    getRangeValue: (state) => {
      return (playerIndex: number, rangeIndex: number) => state.range[playerIndex][rangeIndex];
    },
    getRangeRaw: (state) => {
      return (playerIndex: number, rangeIndex: number) => state.rangeRaw[playerIndex][rangeIndex];
    },
    getRangeStr: (state) => {
      return (playerIndex: number) => state.rangeStrs[playerIndex.toString()];
    }
  },
  // could also be defined as
  // state: () => ({ count: 0 })
  actions: {
    //weight is 0 to 100
    setRangeValue(playerIndex: number, row: number, col: number, weight: number) {
      console.log(`setRangeValue(${playerIndex}, ${row}, ${col}, ${weight})`);
      const rangeIndex = 13 * (row - 1) + col - 1;
      this.range[playerIndex][rangeIndex] = weight;
      this.rangeRaw[playerIndex] = rangeMgr.raw_data();
      rangeMgr.update(row, col, weight / 100);
      this.rangeStrs[playerIndex.toString()] = rangeMgr.to_string();
    },
    // setRangeRaw(playerIndex: number, newRaw: Float32Array) {
    //   this.rangeRaw[playerIndex] = newRaw
    // },

    clear() {
      rangeMgr.clear();
    },
    setRangeString(playerIndex: number, rangeText: string) {
      //this.rangeStrs.set(playerIndex, rangeText);
      this.rangeStrs[playerIndex.toString()] = rangeText;
      const trimmed = rangeText.replace(trimRegex, '$1').trim();

      const errorString = rangeMgr.from_string(trimmed);

      if (errorString) {
        throw new Error(errorString);
      } else {
        const weights = rangeMgr.get_weights();
        for (let i = 0; i < 13 * 13; ++i) {
          this.range[playerIndex][i] = weights[i] * 100;
        }

        this.rangeRaw[playerIndex] = rangeMgr.raw_data();
      }
    }
  }
});
