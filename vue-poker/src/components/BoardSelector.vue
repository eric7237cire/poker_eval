<template>
  <div>
    <template v-if="!isEditing && cardList">
      <div class="not_editing">
        <BoardSelectorCard
          v-for="card in cardList.cards"
          :key="card"
          class="m-1"
          :card-id="card"
          @click="startEditing"
        />
        <button
          class="button-base button-blue"
          @click="startEditing"
          v-if="cardList.cards.length == 0"
        >
          Edit {{ props.expected_length == 2 ? 'Hole Cards' : 'Board' }}
        </button>
      </div>
    </template>
    <template v-if="isEditing && cardList">
      <div class="editor">
        <div v-for="suit in 4" :key="suit" class="flex">
          <BoardSelectorCard
            v-for="rank in 13"
            :key="rank"
            class="m-1"
            :card-id="56 - 4 * rank - suit"
            :is-selected="modelValue.cards.includes(56 - 4 * rank - suit)"
            :is-used="usedCards.includes(56 - 4 * rank - suit)"
            @click="toggleCard(56 - 4 * rank - suit)"
          />
        </div>

        <div class="flex mt-4 mx-1 gap-3">
          <input
            v-model="modelValue.cardText"
            type="text"
            class="w-40 px-2 py-1 rounded-lg text-sm text-black"
            @focus="($event.target as HTMLInputElement).select()"
            @change="onBoardTextChange"
          />
          <button class="button-base button-blue" @click="clearBoard">Clear</button>
          <button class="button-base button-blue" @click="generateRandomBoard">Random Flop</button>
          <button class="button-base button-blue" @click="editDone">Ok</button>
        </div>

        <div
          v-if="
            props.expected_length > 0 && props.modelValue.cards.length !== props.expected_length
          "
          class="mt-5 text-orange-500 font-semibold"
        >
          <span class="underline">Warning:</span>
          Expecting {{ props.expected_length }} Cards
        </div>
      </div>
    </template>
  </div>
</template>

<style lang="postcss" scoped>
.not_editing {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
}
.editor {
  z-index: 10;
  position: relative;
  width: 600px;
  opacity: 1;
  background-color: rgb(20, 20, 20);
  padding: 20px;
  border: 1px solid green;
}
</style>

<script setup lang="ts">
import { computed, defineComponent, ref } from 'vue';
import { CardList, useBoardStore } from '../stores/board';
import { cardText, parseCardString } from '../utils';

import BoardSelectorCard from './BoardSelectorCard.vue';
import { PlayerState, usePlayerStore } from '@src/stores/player';

interface Props {
  expected_length: number;

  //Part of v-model
  modelValue: CardList;
}

const props = defineProps<Props>();

//The update part of v-model
const emit = defineEmits<{
  updateModelValue: [value: CardList];
}>();

const cardList = props.modelValue;

if (cardList && !Array.isArray(cardList.cards)) {
  cardList.cards = [];
}

const isEditing = ref(false);

//Listen to the board and player stores in order to know what cards are used
const boardStore = useBoardStore();
const playerStore = usePlayerStore();

const usedCards = computed(() => {
  let uc = [] as Array<number>;

  for (const player of playerStore.players) {
    if (player.state === PlayerState.USE_HOLE) {
      uc = uc.concat(player.holeCards.cards);
    }
  }

  uc = uc.concat(boardStore.board.cards);
  return uc;
});

// //Initialize
// onBoardTextChange();

function toggleCard(cardId: number, updateText = true) {
  if (cardList.cards.includes(cardId)) {
    //removes the card
    cardList.cards = cardList.cards.filter((card) => card !== cardId);
  } else if (cardList.cards.length < 5) {
    //adds the card

    //Unless it's used
    if (usedCards.value.includes(cardId)) {
      console.log(`Card is used: ${cardId}`);
      return;
    }

    cardList.cards.push(cardId);
    if (cardList.cards.length <= 3) {
      cardList.cards.sort((a, b) => b - a);
    }
  }

  if (updateText) {
    setBoardTextFromButtons();
  }
}

function setBoardTextFromButtons() {
  cardList.cardText = cardList.cards
    .map(cardText)
    .map(({ rank, suitLetter }) => rank + suitLetter)
    .join(', ');

  console.log('boardText.value', cardList.cardText);
  //TODO update
  //cardList.cardsText = boardText.value;
}

function editDone() {
  isEditing.value = false;
}

function startEditing() {
  isEditing.value = true;
}

function onBoardTextChange() {
  cardList.cards = [];

  const cardIds = cardList.cardText
    // Allow pasting in things like [Ah Kd Qc], by reformatting to Ah,Kd,Qc
    .trim()
    .replace(/[^A-Za-z0-9\s,]/g, '')
    .replace(/\s+/g, ',')
    .split(',')
    .map(parseCardString)
    .filter((cardId): cardId is number => cardId !== null);

  new Set(cardIds).forEach((cardId) => toggleCard(cardId, false));
  setBoardTextFromButtons();
}

function clearBoard() {
  cardList.cards = [];
  setBoardTextFromButtons();
}

function generateRandomBoard() {
  cardList.cards = [];

  while (cardList.cards.length < props.expected_length) {
    const randomCard = Math.floor(Math.random() * 52);
    if (!cardList.cards.includes(randomCard)) {
      cardList.cards.push(randomCard);
    }
  }

  cardList.cards.sort((a, b) => b - a);
  setBoardTextFromButtons();
}
</script>
