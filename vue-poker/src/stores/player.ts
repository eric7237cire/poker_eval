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
}

const PLAYER_ID_HERO = 0;

// stores/counter.js
import { defineStore } from 'pinia';
import { RangeManager } from '@pkg/range';
import { useLocalStorage } from '@vueuse/core';
import { CardList } from './board';
import { parseCardString } from '@src/utils';
import { computed, ref } from 'vue';

function initializePlayers(): Array<Player> {
  const players: Array<Player> = [];

  for (let i = 0; i < 10; i++) {
    players.push({
      index: i,
      name: `Player ${i}`,
      rangeStr: '',
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
let range: RangeManager | null = null;

async function initRangeManager() {
  let mod = await import('@pkg/range');
  await mod.default();

  range = RangeManager.new();
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
  function updateRangeStrForPlayer(playerId: number, newRangeStr: string) {
    if (range == null) {
      console.log('Range not initialized yet');
      return;
    }
    console.log('updateRangeStrForPlayer', playerId, newRangeStr);
    players.value[playerId].rangeStr = newRangeStr;

    //update stats
    range.from_string(newRangeStr);
    const rawData = range.raw_data();
    const numCombos = rawData.reduce((acc, cur) => acc + cur, 0);
    players.value[playerId].percHands = numCombos / ((52 * 51) / 2);
    const weights = range.get_weights();
    for (let i = 0; i < 13 * 13; ++i) {
      players.value[playerId].range[i] = weights[i] * 100;
    }
    const check = players.value[playerId].range.filter((r) => r > 0).length;
    console.log(`updateRangeStrForPlayer range check ${check}`);
  }

  return {
    players,
    currentPlayer,
    setCurrentPlayer,
    playerDataForId,
    updateRangeStr,
    updateRangeStrForPlayer,
    curPlayerData
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
