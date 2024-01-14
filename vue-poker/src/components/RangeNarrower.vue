<template>
  <div class="root">
    Range Narrower
    <!-- <div class="instructions">
      <ol>
        <li>1. Choose a range to narrow down</li>
        <li>2. Choose ranges to evaluate against</li>
        <li>3. Choose minimum equity</li>
        <li>4. Number of iterations</li>
        <li>5. See result</li>
      </ol>
    </div> -->
    <div class="narrow-type">
      <v-switch
        v-model="narrowStore.state.useEquity"
        :label="narrowStore.state.useEquity ? 'Equity' : 'Pref'"
        color="success"
      />
    </div>
    <div class="min-equity" v-if="narrowStore.state.useEquity">
      Minimum equity
      <v-slider
        v-model="narrowStore.state.minEquity"
        :min="0"
        :max="1"
        :step="0.01"
        thumb-label="always"
      ></v-slider>
    </div>
    <div class="min-equity" v-if="!narrowStore.state.useEquity">
      Minimum Preference (Currently {{ narrowStore.getLikesHandMinimumString() }})
      <v-slider
        v-model="narrowStore.state.likesHandMinimum"
        :min="0"
        :max="4"
        :step="1"
        thumb-label="always"
      ></v-slider>
    </div>
    <div class="num-opponents">
      <v-text-field
        type="number"
        label="Number of opponents"
        v-model.number="narrowStore.state.numOpponents"
      ></v-text-field>
    </div>
    <div class="copy-to">
      Opponent ranges
      <ul>
        <li v-for="(oppRange, opp_index) in narrowStore.state.opponentRanges">
          <label :for="'to' + (opp_index + 1)">Opponent {{ opp_index }}</label>
          <div class="selectRangeContainer">
            <v-select
              label="Common Ranges"
              v-model="oppRange.rangeStr"
              :items="selectableRanges"
            ></v-select>
          </div>
          <v-text-field label="Range" v-model="oppRange.rangeStr"></v-text-field>
        </li>
      </ul>
    </div>
    <div class="num-simul">
      <v-text-field
        type="number"
        label="Number of simulations"
        v-model.number="narrowStore.state.numSimulations"
      ></v-text-field>
    </div>
  </div>
</template>

<style lang="postcss" scoped>
.root {
  background-color: rgba(0, 0, 255, 0.05);
  color: white;
  border-radius: 4px;

  box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);

  margin: 20px;

  display: block;
  box-sizing: border-box;

  input {
    background-color: white;
    color: black;
  }
}
</style>

<script setup lang="ts">
import { useNarrowStore } from '@src/stores/narrow';
import { ref, watch } from 'vue';

import * as _ from 'lodash';

import { SELECTABLE_RANGES } from '@src/stores/ranges';

const narrowStore = useNarrowStore();

const selectableRanges = SELECTABLE_RANGES;

//below are functions only

watch(
  () => narrowStore.state.numOpponents,
  (newVal) => {
    if (!_.isInteger(newVal) || newVal <= 0 || newVal >= 10) {
      console.log(`invalid numOpponents: ${newVal}`);
      return;
    }

    if (narrowStore.state.opponentRanges.length < newVal) {
      for (let i = narrowStore.state.opponentRanges.length; i < newVal; i++) {
        narrowStore.state.opponentRanges.push({
          range: [],
          rangeStr: '',
          percHands: 0
        });
      }
    } else {
      narrowStore.state.opponentRanges = narrowStore.state.opponentRanges.slice(0, newVal);
    }
  }
);
</script>
