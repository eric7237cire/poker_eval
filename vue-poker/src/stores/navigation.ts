import { useLocalStorage } from '@vueuse/core';

export enum CurrentPage {
  MAIN = 0,
  RANGE_EDITOR = 1
}

// stores/counter.js
import { defineStore } from 'pinia';

export const useNavStore = defineStore('nav', {
  state: () => {
    return { currentPage: useLocalStorage('pinia/current_page', CurrentPage.MAIN) };
  },
  getters: {
    currentPlayer: (state) => state.currentPage
  },
  actions: {
    setCurrentPage(newCurrentPage: CurrentPage) {
      this.currentPage = newCurrentPage;
    }
  }
});
