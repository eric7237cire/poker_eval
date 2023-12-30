<!--
This is a card like in a result table
-->
<template>
  <span class="card-text" :class="card.colorClass">{{ card.rank + card.suit }} </span>
</template>
<script setup lang="ts">
import { cardText } from '@src/utils';
import { computed } from 'vue';

const props = defineProps<{
  cardNumber: number;
}>();

const card = computed(() => {
  return cardText(props.cardNumber);
});
</script>
