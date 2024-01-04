import { defineStore } from 'pinia';
import { ResultsInterface } from '@src/worker/result_types';
import { ref } from 'vue';

export const SELECTABLE_RANGES: Array<{ title: string; value: string }> = [
  { title: '9max UTG-UTG+2', value: '77+,A4s+,AJo+,K9s+,K5s,KQo,QTs+,JTs' },
  { title: '9max MP1-MP2', value: '55+,A3s+,ATo+,K8s+,K6s-K5s,KJo+,Q9s+,J9s+,T9s,76s' },
  { title: '9max HJ-CO', value: '33+,A2s+,A9o+,K4s+,KTo+,Q6s+,QTo+,J8s+,JTo,T8s+,97s+,87s' },
  {
    title: '9max Button',
    value: '22+,A2+,K2s+,K7o+,Q2s+,Q8o+,J3s+,J8o+,T5s+,T8o+,96s+,98o,85s+,87o,75s+,76o,64s+,53s+'
  },
  {
    title: '9max SB',
    value: '22+,A2+,K2+,Q2+,J2+,T2s+,T6o+,92s+,95o+,82s+,84o+,72s+,74o+,62s+,65o,53s+'
  },
  { title: 'All', value: '22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32' },
  { title: 'Without Premium', value: 'TT-22,AJs-A2s,AQo-A2o,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32' },
  {
    title: 'Custom',
    value: ''
  }
];

export const useRangesStore = defineStore('ranges', () => {
  /*
  ref()s become state properties
computed()s become getters
function()s become actions*/

  //range strings (like AA, KJo, 33, ordered strongest first)
  const heads_up_ranges = ref([] as Array<String>);
  const multiway_ranges = ref([] as Array<String>);

  async function init_ranges() {
    const hu = await fetch_range_from_asset('./heads_up_rank.txt');
    const mw = await fetch_range_from_asset('./multiway_rank.txt');

    heads_up_ranges.value = hu;
    multiway_ranges.value = mw;

    //console.log('ranges init', heads_up_ranges.value, multiway_ranges.value);
  }

  return {
    heads_up_ranges,
    multiway_ranges,
    init_ranges
  };
});

async function fetch_range_from_asset(asset_url: string): Promise<Array<String>> {
  //fetches a range from an asset url

  const r = await fetch(asset_url);

  const text = await r.text();

  const ret = [] as Array<String>;

  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }

    if (trimmed[0] == '#') {
      continue;
    }

    ret.push(line);
  }

  return ret;
}
