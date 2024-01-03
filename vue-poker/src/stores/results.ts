import { defineStore } from 'pinia';
import { ResultsInterface } from '@src/worker/result_types';
import { ref } from 'vue';
import { useLocalStorage } from '@vueuse/core';

export const useResultsStore = defineStore('results', () => {
  const results = ref([] as Array<ResultsInterface>);

  const streetVisible = useLocalStorage(
    'streetVisible',
    () => {
      return [true, true, true];
    },
    {
      mergeDefaults: true
    }
  );

  return {
    results,
    streetVisible
  };
});
