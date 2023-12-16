<template>
  <div class="flex flex-col w-full border-l border-gray-500 overflow-x-auto">
    <div ref="tableDiv" class="flex-grow overflow-y-scroll will-change-scroll">
      <table class="w-full h-full text-sm text-center align-middle">
        <thead class="sticky top-0 z-30 bg-gray-100 shadow">
          <tr style="height: calc(1.9rem + 1px)">
            <th
              v-for="column in columns"
              :key="column.label"
              scope="col"
              :class="'whitespace-nowrap select-none '"
              :style="{
                'min-width':
                  (column.type === 'card' ? '4' : column.type === 'bar' ? '6' : '3.5') + 'rem'
              }"
            >
              <span>{{ column.label }}</span>
            </th>
          </tr>

          <tr style="height: calc(1.9rem + 1px)">
            <th
              v-for="column in columns"
              :key="column.label"
              scope="col"
              :class="'header-divider '"
            ></th>
          </tr>
        </thead>

        <tbody>
          <!-- Body -->
          <tr
            v-for="item in resultsRendered"
            :key="item[0]"
            :class="'relative ' + 'bg-gray-50'"
            style="height: calc(1.9rem + 1px)"
          >
            <td v-for="column in columns" :key="column.label"></td>
          </tr>

          <!-- No results -->
          <tr v-if="resultsRendered.length === 0">
            <td
              class="relative bg-gray-50 row-divider"
              style="height: calc(1.9rem + 1px)"
              :colspan="columns.length"
            ></td>
          </tr>

          <!-- Spacer -->
          <tr>
            <td :colspan="columns.length" class="relative row-divider"></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, reactive, ref, toRefs, watch } from 'vue';

import {
  ranks,
  suitLetters,
  cardText,
  cardPairOrder,
  toFixed1,
  toFixed,
  toFixedAdaptive,
  capitalize
} from '../utils';


import { Tippy } from 'vue-tippy';
import { ArrowTopRightOnSquareIcon } from '@heroicons/vue/24/solid';
import { Results } from '../../pkg/poker_eval';


interface ExpandedResults extends Results {
  player_id: number;
  equity: number;

  ranks: PercOrBetter[];
}

const props = defineProps<{
  results: Array<Results>
}>();


  const columnNames = [
    'equity'

  ]
    const flopResults = Array.from(props.results.entries()).map(([rIdx, r])=>{

      const equity = (r.win_eq + r.tie_eq) / r.num_iterations;

      const hiCard: PercOrBetter = {
        perc: r.num_hi_card / r.num_iterations,
        better: r.num_hi_card / r.num_iterations
      };

      const pair: PercOrBetter = {
        perc: r.num_pair / r.num_iterations,
        better: r.num_pair / r.num_iterations 
      };

      const twoPair: PercOrBetter = {
        perc: r.num_two_pair / r.num_iterations,
        better: r.num_two_pair / r.num_iterations
      };

      return {
        player_id: rIdx,
        equity: equity,
        ranks: [hiCard]
      }
    })

    


    const resultsRendered = resultsSorted;

    
</script>

<style scoped>
.header-divider::before {
  content: '';
  @apply absolute left-0 -bottom-px w-full border-b border-gray-300;
}

.row-divider::before {
  content: '';
  @apply absolute left-0 top-0 w-full border-t border-gray-300;
}
</style>
