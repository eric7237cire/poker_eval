<!-- <span
                    v-else-if="
                      column.type === 'percentage' || column.type === 'action'
                    "
                    :data-set="
                      (strTmp = toFixed1(summary[columnIndex(column)] * 100))
                    "
                  >
                    <span>{{ strTmp.slice(0, -1) }}</span>
                    <span class="text-xs">{{ strTmp.slice(-1) }}%</span>
                  </span> -->

<template>
  <span>
    <span>{{ strTmp.slice(0, -1) }}</span>
    <span class="text-xs">{{ strTmp.slice(-1) }}%</span>
  </span>
</template>
<script setup lang="ts">
import { toFixed1 } from '@src/utils';
import { PercOrBetter } from '@src/worker/result_types';
import { computed, defineComponent, watch } from 'vue';

const props = defineProps<{
  perc: number
}>();

const strTmp = computed(() => {
  return toFixed1(100*props.perc);
});
</script>
