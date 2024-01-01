<template>
  <div class="root">
    <div class="street-toggles">
      <v-switch v-model="resultsStore.streetVisible[0]" label="Flop"   />
      <v-switch v-model="resultsStore.streetVisible[1]" label="Turn"  />
      <v-switch v-model="resultsStore.streetVisible[2]" label="River"  />
    </div>

    <div class="flex flex-col w-full border-l border-gray-500 overflow-x-auto">
      <div ref="tableDiv" class="flex-grow overflow-y-scroll will-change-scroll">
        <table class="w-full h-full text-sm text-center align-middle">
          <thead class="sticky top-0 z-30 shadow">
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
                  class="relative"
                  style="height: calc(1.9rem + 1px)"
                  v-if="resultsStore.streetVisible[street_index - 1]"
                >
                  <td>
                    <template v-if="item.player_index >= 0">
                      {{
                        playerStore.players[item.player_index].name || 'Player ' + item.player_index
                      }}
                    </template>
                    <template v-else> Player >= 1 </template>
                  </td>
                  <td><PlayerPreflop :player-id="item.player_index" /></td>
                  <td>{{ getStreetName(street_index) }}</td>
                  <td>
                    <Percentage :perc="item.street_results[street_index - 1].equity" />

                    <RangeEquityViewer
                      :range_it_num="
                        item.street_results[street_index - 1].it_num_by_simple_range_idx
                      "
                      :range_equity="item.street_results[street_index - 1].eq_by_simple_range_idx"
                    />
                  </td>
                  <td v-for="index in 9" :key="index">
                    <!-- {{item.rank_family_count}}  -->
                    <!-- {{ index }} -->
                    <!-- {{item.rank_family_count[index-1].perc}} -->
                    <Percentage
                      :perc="
                        item.street_results[street_index - 1].rank_family_count[index - 1].perc
                      "
                    />
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

    <!--Draw table-->
    <div v-if="!equityOnly" class="flex flex-col w-full border-l border-gray-500 overflow-x-auto">
      <div ref="tableDiv" class="flex-grow overflow-y-scroll will-change-scroll">
        <table class="w-full h-full text-sm text-center align-middle">
          <thead class="sticky top-0 z-30 shadow">
            <tr style="height: calc(1.9rem + 1px)">
              <th
                v-for="columnLabel in drawColumnNames"
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
                v-for="columnLabel in drawColumnNames"
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
              <template v-for="draw_index in 2" :key="draw_index">
                <tr :class="'relative '" style="height: calc(1.9rem + 1px)">
                  <td>
                    <template v-if="item.player_index >= 0">
                      Player {{ item.player_index }}
                    </template>
                    <template v-else> Player >= 1 </template>
                  </td>
                  <td><PlayerPreflop :player-id="item.player_index" /></td>
                  <td>{{ getStreetName(draw_index) }}</td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].flush_draw /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].str8_draw /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].gut_shot /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].hi_paired /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].lo_paired /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].pp_paired /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].two_overcards /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].one_overcard /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
                  </td>
                  <td>
                    <Percentage
                      :perc="
                        item.draw_results[draw_index - 1].backdoor_flush_draw /
                        item.draw_results[draw_index - 1].num_iterations
                      "
                    />
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
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, reactive, ref, toRefs, watch } from 'vue';
import { RANK_FAMILY_NAMES, ResultsInterface } from '@src/worker/result_types';
import Percentage from '@src/components/result/Percentage.vue';
import PlayerPreflop from './result/PlayerPreflop.vue';
import RangeEquityViewer from './result/RangeEquityViewer.vue';
import { usePlayerStore } from '@src/stores/player';
import { useResultsStore } from '@src/stores/results';


const props = defineProps<{
  results: Array<ResultsInterface>;
  equityOnly: boolean;
}>();

const columnNames = ['Player Id', 'Cards', 'Street', 'Equity', ...RANK_FAMILY_NAMES];

const drawColumnNames = [
  'Player Id',
  'Cards',
  'Street',
  'Flush Draw',
  'Straight Draw',
  'Gutshot Draw',
  'Hi Paired',
  'Lo Paired',
  'PP paired',
  'Two Overcards',
  'One Overcard',
  'Backdoor Flush Draw'
];

const playerStore = usePlayerStore();

const results = computed(() => props.results);

const resultsStore = useResultsStore();

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
.root {
  background-color: rgb(20, 20, 20);

  .street-toggles {
    display: flex;
    flex-direction: row;
    justify-content: center;
    gap: 10px;
    margin-top: 20px;
    padding-top: 10px;

    > * {
      max-width: 100px;
    }

    ::v-deep label {
      color: white;
    }
  }

  thead {
    color: green;
    font-weight: bold;
  }

  tbody {
    tr {
      color: white;
    }
  }
}
.header-divider::before {
  content: '';
}

.row-divider::before {
  content: '';
}
</style>

