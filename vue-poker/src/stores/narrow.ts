import { defineStore } from 'pinia';
import { RangeInStore } from './player';
import { ref } from 'vue';
import { useLocalStorage } from '@vueuse/core';

interface NarrowStoreData {
  numOpponents: number;
  opponentRanges: Array<RangeInStore>;
  numSimulations: number;
  // between 0 and 1
  minEquity: number;

  //false will use "likes_hands"
  useEquity: boolean;

  // See LikesHandLevel
  likesHandMinimum: number;
}

export const useNarrowStore = defineStore('narrow', () => {
  const state = useLocalStorage<NarrowStoreData>(
    'narrow',
    {
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
      useEquity: true,
      likesHandMinimum: 2 // see LikesHandLevel
    },
    {
      mergeDefaults: true
    }
  );

  function getLikesHandMinimumString() : string {
    switch (state.value.likesHandMinimum) {
      case 0:
        return 'Any';
      case 1:
        return 'Call Small Bet';
      case 2:
        return 'Small Bet (1/3 pot)';
      case 3:
        return 'Large Bet (pot)';
      case 4:
        return 'All-in';
    }

    return 'Any';
  }

  return {
    getLikesHandMinimumString,
    state
  };
});
