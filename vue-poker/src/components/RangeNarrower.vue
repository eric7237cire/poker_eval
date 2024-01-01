<template>
  <div class="root">
    Range Narrower
    <div class="instructions">
      <ol>
        <li>1. Choose a range to narrow down</li>
        <li>2. Choose ranges to evaluate against</li>
        <li>3. Choose minimum equity</li>
        <li>4. Number of iterations</li>
        <li>5. See result</li>
      </ol>
    </div>
    <div class="copy-from">
      Copy From:
      <ul>
        <li v-for="player in playerStore.players">
          <input
            type="radio"
            :id="player.index.toString()"
            :value="player.index"
            v-model="copyFrom"
          />
          <label :for="player.index.toString()">{{ player.name }}</label>
          <div class="range">
            Range:
            {{ player.rangeStr }}
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<style lang="postcss" scoped>
.root {
  background-color: rgba(0, 0, 255, 0.05);
  border-radius: 4px;

  box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);

  margin: 20px;

  display: block;
  box-sizing: border-box;

  .copy-from {
    border: 1px solid blue;
    padding: 10px;
    label {
      margin-left: 10px;
    }
    .range {
      display: inline;
    }
  }
}
</style>

<script setup lang="ts">
import { usePlayerStore } from '@src/stores/player';
import { ref } from 'vue';

const playerStore = usePlayerStore();

const copyFrom = ref(0);
</script>
