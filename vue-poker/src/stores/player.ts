export const enum PlayerIds {
  HERO = 0,
  WEST = 1,
  NORTH_WEST = 2,
  NORTH_EAST = 3,
  EAST = 4
}

export interface Player {
  id: PlayerIds;
  rangeStr: string;
  percHands: number;
}

// stores/counter.js
import { defineStore } from 'pinia';
import { RangeManager } from '../../../ui/pkg/range/range';


function initializePlayers(): Array<Player> {
  const players: Array<Player> = [];

  for (let i = 0; i < 5; i++) {
    players.push({
      id: i,
      rangeStr: '',
      percHands: 0
    });
  }
  return players;
}

//private local to update some stats
const range = RangeManager.new();

export const usePlayerStore = defineStore('player', {
  state: () => {
    return {
      currentPlayer: PlayerIds.HERO,
      players: initializePlayers()
    };
  },
  getters: {
    curPlayerData: (state) => state.players[state.currentPlayer],
    playerDataForId: (state) => (id: PlayerIds) => state.players[id]
    //currentPlayer: (state) => state.currentPlayer
  },
  actions: {
    setCurrentPlayer(newCurrentPlayer: PlayerIds) {
      this.currentPlayer = newCurrentPlayer;
    },
    updateRangeStr(newRangeStr: string) {
        this.updateRangeStrForPlayer(this.currentPlayer, newRangeStr);
    },
    updateRangeStrForPlayer(playerId: PlayerIds, newRangeStr: string) {
        console.log('updateRangeStrForPlayer', playerId, newRangeStr);
        this.players[playerId].rangeStr = newRangeStr;

        //update stats
        range.from_string(newRangeStr);
        const rawData = range.raw_data();
        const numCombos = rawData.reduce((acc, cur) => acc + cur, 0);
        this.players[playerId].percHands = numCombos / (52*51/2);
    }
  }
});
