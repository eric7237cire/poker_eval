import { useLocalStorage } from '@vueuse/core';

export enum CurrentPage {
  MAIN = 0,
  RANGE_EDITOR = 1
}

// stores/counter.js
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNavStore = defineStore('nav', () => {
  const currentPage = useLocalStorage('pinia/current_page', CurrentPage.MAIN, {
    mergeDefaults: true
  });

  const rangeEditorTryTopY = ref(0);

  return {
    currentPage,
    rangeEditorTryTopY
  };
});
