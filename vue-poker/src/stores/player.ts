const enum PlayerIds {
    HERO = 0,
    WEST = 1,
    NORTH_WEST = 2,
    NORTH_EAST = 3,
    EAST = 4
}

// stores/counter.js
import { defineStore } from 'pinia'

export const usePlayerStore = defineStore('player', {
  state: () => {
    return { currentPlayer: PlayerIds.HERO }
  },
  getters: {
    //currentPlayer: (state) => state.currentPlayer
  },
  actions: {
    setCurrentPlayer(newCurrentPlayer: PlayerIds) {
      this.currentPlayer = newCurrentPlayer
    },
   
  }
})
