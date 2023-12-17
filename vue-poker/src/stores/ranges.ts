import { defineStore } from 'pinia';
import { ResultsInterface } from '@src/worker/result_types';
import { ref } from 'vue';

export const useRangesStore = defineStore('ranges', () => {
  /*
  ref()s become state properties
computed()s become getters
function()s become actions*/

  //range strings (like AA, KJo, 33, ordered strongest first)
  const heads_up_ranges = ref([] as Array<String>);
  const multiway_ranges = ref([] as Array<String>);

  async function init_ranges() {
    const hu = await fetch_range_from_asset('/assets/ranges/heads_up_rank.txt');
    const mw = await fetch_range_from_asset('/assets/ranges/multiway_rank.txt');

    heads_up_ranges.value = hu;
    multiway_ranges.value = mw;
  }

});

async function fetch_range_from_asset(asset_url: string): Promise<Array<String>> {
  //fetches a range from an asset url

  const r = await fetch(asset_url);

  const text = await r.text();

  const ret = [] as Array<String>;

  for(const line of text.split('\n')) {
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
