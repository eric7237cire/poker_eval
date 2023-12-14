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
}

// stores/counter.js
import { defineStore } from 'pinia';

function initializePlayers() {
  const players: { [key: number]: Player } = {};

  for (let i = 0; i < 5; i++) {
    players[i] = {
      id: i,
      rangeStr: ''
    };
  }

  return players;
}
function initializePlayers2(): Array<Player> {
  const players: Array<Player> = [];

  for (let i = 0; i < 5; i++) {
    players.push({
      id: i,
      rangeStr: ''
    });
  }
  return players;
}

export const usePlayerStore = defineStore('player', {
  state: () => {
    return {
      currentPlayer: PlayerIds.HERO,
      players: initializePlayers2()
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
        this.players[this.currentPlayer].rangeStr = newRangeStr;
    }
  }
});
