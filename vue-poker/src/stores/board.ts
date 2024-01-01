// stores/counter.js
import { parseCardString } from '@src/utils';
import { useLocalStorage } from '@vueuse/core';
import { defineStore } from 'pinia';

export interface CardList {
  cardText: string;
  cards: number[];
}

export const useBoardStore = defineStore('board', {
  state: () => {
    return {
      board: useLocalStorage(
        'boardText',
        {
          cardText: '',
          cards: [] as number[]
        },
        {
          mergeDefaults: true
        }
      )
    };
  },
  getters: {
    expectedBoardLength: (state) => 3
  }
  // could also be defined as
  // state: () => ({ count: 0 })
  // actions: {
  //   setBoard(newBoard: number[]) {
  //     this.board = newBoard;
  //   }
  // }
});
