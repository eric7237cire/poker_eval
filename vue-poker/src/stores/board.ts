// stores/counter.js
import { useLocalStorage } from '@vueuse/core';
import { defineStore } from 'pinia';

export const useBoardStore = defineStore('board', {
  state: () => {
    return {
      board: [] as number[],
      boardText: useLocalStorage('boardText', '')
    };
  },
  getters: {
    expectedBoardLength: (state) => 3
  },
  // could also be defined as
  // state: () => ({ count: 0 })
  actions: {
    setBoard(newBoard: number[]) {
      this.board = newBoard;
    }
  }
});
