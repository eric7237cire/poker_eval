import { defineStore } from 'pinia';
import { RangeInStore } from './player';
import { ref } from 'vue';
import { useLocalStorage } from '@vueuse/core';

interface NarrowStoreData {
  rangeToNarrow: RangeInStore;
  numOpponents: number;
  opponentRanges: Array<RangeInStore>;
  numSimulations: number;
  // between 0 and 1
  minEquity: number;
  result: RangeInStore;
}

export const useNarrowStore = defineStore('narrow', () => {
  const state = useLocalStorage<NarrowStoreData>(
    'narrow',
    {
      rangeToNarrow: {
        rangeStr: '',
        range: [],
        percHands: 0
      },
      numOpponents: 2,
      numSimulations: 1000,
      minEquity: 0.3,
      opponentRanges: [
        {
          rangeStr: '',
          range: [],
          percHands: 0
        },
        {
          rangeStr: '',
          range: [],
          percHands: 0
        }
      ],
      result: {
        rangeStr: '',
        range: [],
        percHands: 0
      }
    },
    {
      mergeDefaults: true
    }
  );

  return {
    state
  };
});
