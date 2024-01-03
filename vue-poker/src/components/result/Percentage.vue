<template>
  <span>
    <span>{{ strTmp.slice(0, -1) }}</span>
    <span class="text-xs">{{ strTmp.slice(-1) }}%</span>
  </span>
</template>
<script setup lang="ts">
import { toFixed1 } from '@src/utils';
import { computed} from 'vue';

const props = defineProps<{
  perc: number;
}>();

const strTmp = computed(() => {
  return toFixed1(100 * props.perc);
});
</script>
