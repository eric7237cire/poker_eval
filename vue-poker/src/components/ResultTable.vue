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
          <tr
            v-for="(item, index) in results"
            :class="'relative ' + 'bg-gray-50'"
            style="height: calc(1.9rem + 1px)"
          >
            <td>Player {{ index }}</td>
            <td>
              <Percentage :perc="item.equity" />
            </td>
            <td v-for="index in 9" :key="index">
               <!-- {{item.rank_family_count}}  -->
               <!-- {{ index }} -->
              <!-- {{item.rank_family_count[index-1].perc}} -->
              <Percentage :perc="item.rank_family_count[index-1].perc" />
            </td>
          </tr>

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

const columnNames = ['Player Id', 'Equity', ...RANK_FAMILY_NAMES];

const results = computed(() => props.results);
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
