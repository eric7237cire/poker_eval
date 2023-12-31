export enum PlayerState {
  DISABLED = 0,
  //We are setting the cards
  USE_HOLE = 1,
  USE_RANGE = 2
}

export interface Player {
  index: number;
  name: string;

  state: PlayerState;
  holeCards: CardList;
  rangeStr: string;
  percHands: number;

  //13 * 13 array with 0 to 100%
  range: Array<number>;
}

const PLAYER_ID_HERO = 0;

// stores/counter.js
import { defineStore } from 'pinia';
import { RangeManager } from '@pkg/range';
import { useLocalStorage } from '@vueuse/core';
import { CardList } from './board';
import { parseCardString } from '@src/utils';

function initializePlayers(): Array<Player> {
  const players: Array<Player> = [];

  for (let i = 0; i < 5; i++) {
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

export const usePlayerStore = defineStore('player', {
  state: () => {
    return {
      currentPlayer: PLAYER_ID_HERO,
      players: useLocalStorage('playerData', initializePlayers())
    };
  },
  getters: {
    curPlayerData: (state) => state.players[state.currentPlayer],
    playerDataForId: (state) => (id: number) => state.players[id]
    //currentPlayer: (state) => state.currentPlayer
  },
  actions: {
    setCurrentPlayer(newCurrentPlayer: number) {
      this.currentPlayer = newCurrentPlayer;
    },
    updateRangeStr(newRangeStr: string) {
      this.updateRangeStrForPlayer(this.currentPlayer, newRangeStr);
    },
    updateRangeStrForPlayer(playerId: number, newRangeStr: string) {
      if (range == null) {
        console.log('Range not initialized yet');
        return;
      }
      console.log('updateRangeStrForPlayer', playerId, newRangeStr);
      this.players[playerId].rangeStr = newRangeStr;

      //update stats
      range.from_string(newRangeStr);
      const rawData = range.raw_data();
      const numCombos = rawData.reduce((acc, cur) => acc + cur, 0);
      this.players[playerId].percHands = numCombos / ((52 * 51) / 2);
      const weights = range.get_weights();
      for (let i = 0; i < 13 * 13; ++i) {
        this.players[playerId].range[i] = weights[i] * 100;
      }
    }
  }
});

export function loadHeroCardsFromUrl(): number | null {
  const urlParams = new URLSearchParams(window.location.search);
  const queryParamCardText = urlParams.get('hero') || '';

  if (!queryParamCardText) {
    return null;
  }

  return parseCardString(queryParamCardText);
}
