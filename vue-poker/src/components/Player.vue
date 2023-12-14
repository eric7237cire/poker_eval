<!--
Select a range profile or disable
or select a range
-->

<template>
    Profile Name: {{playerId}} buh 
    <div @click="handleRangeClick()">Range: 22+, A2s+, K2s+, Q2s</div>
    <button>Disabl</button>
</template>

<script lang="ts">

import { defineComponent } from 'vue';
import { CurrentPage, useNavStore } from '../stores/navigation';

export default defineComponent({
  props: {
    playerId: {
      type: Number,
      required: true,
    },
  },

  setup(props) {

    const navStore = useNavStore();

    const handleRangeClick = () => {
      console.log("range clicked");
        navStore.setCurrentPage(CurrentPage.RANGE_EDITOR);
    }

    return {
        "playerId": props.playerId,
        handleRangeClick
    }
  }
});
</script>