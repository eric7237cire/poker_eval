export enum PlayerState {
  DISABLED = 0,
  //We are setting the cards
  USE_HOLE = 1,
  USE_RANGE = 2
}

export interface RangeInStore {
  rangeStr: string;
  percHands: number;

  //13 * 13 array with 0 to 100%
  range: Array<number>;
}

export interface Player extends RangeInStore {
  index: number;
  name: string;

  state: PlayerState;
  holeCards: CardList;

  //oldest is 1st in the array
  rangeStrHistory: Array<string>;

  //index of the current rangeStr in the history, -1 for when user edits
  historyIndex: number;
}

const PLAYER_ID_HERO = 0;

const RANGE_HISTORY_LIMIT = 5;

// stores/counter.js
import { defineStore } from 'pinia';
import { RangeManager } from '@pkg/range';
import { useLocalStorage } from '@vueuse/core';
import { CardList } from './board';
import { parseCardString } from '@src/utils';
import { computed, ref, watch } from 'vue';
import * as _ from 'lodash';

function initializePlayers(): Array<Player> {
  const players: Array<Player> = [];

  for (let i = 0; i < 10; i++) {
    players.push({
      index: i,
      name: `Player ${i}`,
      rangeStr: '',
      rangeStrHistory: [],
      historyIndex: -1,
      holeCards: {
        cardText: '',
        cards: []
      },
      percHands: 0,
      range: [],
      state: PlayerState.DISABLED
    });
  }
  return players;
}

//private local to update some stats
let range = ref(null as RangeManager | null);

async function initRangeManager() {
  let mod = await import('@pkg/range');
  await mod.default();

  range.value = RangeManager.new();
}

initRangeManager().then(() => {
  console.log('Range initialized');
});

export const usePlayerStore = defineStore('player', () => {
  /*
  ref()s become state properties
computed()s become getters
function()s become actions*/

  const currentPlayer = ref(PLAYER_ID_HERO);

  const players = useLocalStorage('playerData', initializePlayers(), {
    mergeDefaults: true
  });

  for (let i = 0; i < players.value.length; ++i) {
    if (!_.isArray(players.value[i].rangeStrHistory)) {
      players.value[i].rangeStrHistory = [];
    }
  }

  const curPlayerData = computed(() => {
    return players.value[currentPlayer.value];
  });

  function playerDataForId(id: number) {
    return players.value[id];
  }

  function setCurrentPlayer(newCurrentPlayer: number) {
    currentPlayer.value = newCurrentPlayer;
  }
  function updateRangeStr(newRangeStr: string) {
    updateRangeStrForPlayer(currentPlayer.value, newRangeStr);
  }
  function updateRangeStrForPlayer(playerId: number, newRangeStr: string, saveHistory=false) {
    if (range.value == null) {
      console.log('Range not initialized yet');
      return;
    }
    const pData = players.value[playerId];
    if (saveHistory) {
      //save history
      if (pData.rangeStrHistory.length >= RANGE_HISTORY_LIMIT) {
        pData.rangeStrHistory.shift();
      }
      pData.rangeStrHistory.push(newRangeStr);
      pData.historyIndex = pData.rangeStrHistory.length - 1;
    }
    //console.log('updateRangeStrForPlayer', playerId, newRangeStr);
    pData.rangeStr = newRangeStr;

    //update stats
    range.value.from_string(newRangeStr);
    const rawData = range.value.raw_data();
    const numCombos = rawData.reduce((acc, cur) => acc + cur, 0);
    pData.percHands = numCombos / ((52 * 51) / 2);
    const weights = range.value.get_weights();
    for (let i = 0; i < 13 * 13; ++i) {
      pData.range[i] = weights[i] * 100;
    }
    const check = pData.range.filter((r) => r > 0).length;
    console.log(`updateRangeStrForPlayer range check ${check}; saved history ${saveHistory}`);
  }

  return {
    players,
    currentPlayer,
    setCurrentPlayer,
    playerDataForId,
    updateRangeStr,
    updateRangeStrForPlayer,
    curPlayerData,
    //wsm object
    range
  };
});

export function loadHeroCardsFromUrl(): number | null {
  const urlParams = new URLSearchParams(window.location.search);
  const queryParamCardText = urlParams.get('hero') || '';

  if (!queryParamCardText) {
    return null;
  }

  return parseCardString(queryParamCardText);
}
