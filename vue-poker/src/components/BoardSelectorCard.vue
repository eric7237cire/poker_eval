<template>
  <button
    :class="buttonClassNames"

    :style="{
      '--width': width,
      '--font-size': fontSize,
      width: 'var(--width)',
      'padding-top': 'calc(var(--width) * 1.4 - 2px)'
    }"
  >
    <span
      :class="'absolute top-0 font-semibold ' + colorClass"
      :style="{
        left: '15%',
        'font-size': 'calc(var(--font-size) * 1.25)'
      }"
    >
      {{ rank }}
    </span>
    <span
      :class="'absolute ' + colorClass"
      :style="{
        bottom: '5%',
        right: '10%',
        'font-size': 'var(--font-size)'
      }"
    >
      {{ suit }}
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed, defineComponent } from 'vue';
import { cardText } from '../utils';

const buttonClassNames = computed(() => {
      const baseClasses = 'relative rounded-lg border select-none enabled:shadow';

      if (props.isSelected) {
        return baseClasses + ' bg-yellow-300 ring-1 ring-red-600 border-red-600';
      }
      if (props.isUsed) {
        return baseClasses + ' bg-gray-10 border-gray-400';
      }
      
      return baseClasses + ' bg-white border-black';
    });

const props = defineProps(
  {
    cardId: {
      type: Number,
      required: true
    },
    isSelected: {
      type: Boolean,
      default: false
    },
    isUsed: {
      type: Boolean,
      default: false
    },
    width: {
      type: String,
      default: '40px'
    },
    fontSize: {
      type: String,
      default: '1rem'
    }
  });

  
  const { rank, suit, colorClass } = cardText(props.cardId);


</script>
