import { defineStore } from 'pinia';
import { ResultsInterface } from '@src/worker/result_types';

export const useResultsStore = defineStore('results', {
  state: () => {
    return {
      results: [] as Array<ResultsInterface>
    };
  },
  getters: {},
  actions: {}
});
