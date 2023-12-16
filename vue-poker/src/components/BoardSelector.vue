<template>
  <template v-if="!isEditing && cardList">
    <BoardSelectorCard
      v-for="card in cardList.cards"
      :key="card"
      class="m-1"
      :card-id="card"
      @click="startEditing"
    />
    <button @click="startEditing" v-if="cardList.cards.length == 0">Edit</button>
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
        v-if="props.expected_length > 0 && props.modelValue.cards.length !== props.expected_length"
        class="mt-5 text-orange-500 font-semibold"
      >
        <span class="underline">Warning:</span>
        Expecting {{ props.expected_length }} Cards
      </div>
    </div>
  </template>
</template>

<style lang="postcss" scoped>
.editor {
  z-index: 10;
  position: relative;
  opacity: 1;
  background-color: rgb(20, 20, 20);
  padding: 20px;
  border: 1px solid green;
}
</style>

<script setup lang="ts">
import { defineComponent, ref } from 'vue';
import { CardList, useBoardStore } from '../stores/board';
import { cardText, parseCardString } from '../utils';

import BoardSelectorCard from './BoardSelectorCard.vue';

interface Props {
  expected_length: number;
  //min_cards: number,
  modelValue: CardList;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  updateModelValue: [value: CardList];
}>();

const cardList = props.modelValue;

if (cardList && !Array.isArray(cardList.cards)) {
  cardList.cards = [];
}

const isEditing = ref(false);

// //Initialize
// onBoardTextChange();

function toggleCard(cardId: number, updateText = true) {
  if (cardList.cards.includes(cardId)) {
    cardList.cards = cardList.cards.filter((card) => card !== cardId);
  } else if (cardList.cards.length < 5) {
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
