// stores/counter.js
import { defineStore } from 'pinia'

const MAX_PLAYERS = 5;

function buildInitialRange() {
    const range = [];
    for(let i = 0; i < MAX_PLAYERS; i++) {
        range.push(Array.from({ length: 13 * 13 }, () => 0));
    }
    return range;
}

function buildInitialRangeRaw() {
    const range = [];
    for(let i = 0; i < MAX_PLAYERS; i++) {
        range.push(Float32Array.from({ length: (52 * 51) / 2 }, () => 0));
    }
    return range;
}

export const useRangeStore = defineStore('range', {
  state: () => {
    return { range: buildInitialRange(),
      rangeRaw: buildInitialRangeRaw(),
     }
  },
  getters: {
    // rangeValue(state, index: number): number {
    //     return state.range[index];
    // }
    getRangeValue: (state) => {
        return (playerIndex: number, rangeIndex: number) => state.range[playerIndex][rangeIndex]
      },
  },
  // could also be defined as
  // state: () => ({ count: 0 })
  actions: {
    setRangeRaw(playerIndex: number, newRaw: Float32Array ) {
        this.rangeRaw[playerIndex] = newRaw;
    }
    
   
  }
})
