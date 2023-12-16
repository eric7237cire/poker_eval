<template>
  <div class="flex flex-col w-full border-l border-gray-500 overflow-x-auto">
    <div ref="tableDiv" class="flex-grow overflow-y-scroll will-change-scroll">
      <table class="w-full h-full text-sm text-center align-middle">
        <thead class="sticky top-0 z-30 bg-gray-100 shadow">
          <tr style="height: calc(1.9rem + 1px)">
            <th
              v-for="columnLabel in columnNames"
              :key="columnLabel"
              scope="col"
              :class="'whitespace-nowrap select-none '"
              :style="{
                'min-width': '3.5rem'
              }"
            >
              <span>{{ columnLabel }}</span>
            </th>
          </tr>

          <tr style="height: calc(1.9rem + 1px)">
            <th
              v-for="columnLabel in columnNames"
              :key="columnLabel"
              scope="col"
              :class="'header-divider '"
            ></th>
          </tr>
        </thead>

        <tbody>
          <!-- Body -->
          <!--3 rows per flop result -->
          <template v-for="item in results" :key="item.player_index">
            <template v-for="street_index in 3" :key="street_index">
              <tr 
                :class="'relative ' + 'bg-gray-50'"
                style="height: calc(1.9rem + 1px)"
              >
                <td>Player {{ item.player_index }}</td>
                <td>{{ getStreetName(street_index) }}</td>
                <td>
                  <Percentage :perc="item.street_results[street_index-1].equity" />
                </td>
                <td v-for="index in 9" :key="index">
                  <!-- {{item.rank_family_count}}  -->
                  <!-- {{ index }} -->
                  <!-- {{item.rank_family_count[index-1].perc}} -->
                  <Percentage :perc="item.street_results[street_index-1].rank_family_count[index - 1].perc" />
                </td>
              </tr>
            </template>
          </template>

          <!-- No results -->
          <tr v-if="results.length === 0">
            <td
              class="relative bg-gray-50 row-divider"
              style="height: calc(1.9rem + 1px)"
              :colspan="columnNames.length"
            ></td>
          </tr>

          <!-- Spacer -->
          <tr>
            <td :colspan="columnNames.length" class="relative row-divider"></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, reactive, ref, toRefs, watch } from 'vue';
import { RANK_FAMILY_NAMES, ResultsInterface } from '@src/worker/result_types';
import Percentage from '@src/components/result/Percentage.vue';

const props = defineProps<{
  results: Array<ResultsInterface>;
}>();

const columnNames = ['Player Id', 'Street', 'Equity', ...RANK_FAMILY_NAMES];

const results = computed(() => props.results);


function getStreetName(street_index: number) {
  switch (street_index) {
    case 1:
      return 'Flop';
    case 2:
      return 'Turn';
    case 3:
      return 'River';
    default:
      return 'Unknown';
  }
}

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
