// stores/counter.js
import { parseCardString } from '@src/utils';
import { useLocalStorage } from '@vueuse/core';
import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export interface CardList {
  cardText: string;
  cards: number[];
}

export const useBoardStore = defineStore('board', () => {
  const board = useLocalStorage(
    'boardText',
    {
      cardText: '',
      cards: [] as number[]
    },
    {
      mergeDefaults: true
    }
  );

  const reserveCards = ref([] as number[]);

  watch( () => board.value.cards.length , (val) => {
    console.log('board changed', val);
    if (val === 0 || val === 5) {
      reserveCards.value = [];
    }
  });

  

  return {
    board,
    reserveCards
  };
});
