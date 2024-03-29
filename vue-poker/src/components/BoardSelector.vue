<template>
  <div ref="root_element" class="template_root">
    <template v-if="!isEditing && cardListRef">
      <div ref="not_editing" class="not_editing">
        <BoardSelectorCard
          v-for="card in cardListRef.cards"
          :key="card"
          class="m-1"
          :card-id="card"
          @click="startEditing"
        />
        <button
          class="button-base button-blue"
          @click="startEditing"
          v-if="cardListRef.cards.length == 0"
        >
          Edit {{ props.max_expected_length == 2 ? 'Hole Cards' : 'Board' }}
        </button>
      </div>
    </template>
    <template v-if="isEditing && cardListRef">
      <div ref="editor" class="editor" :style="editorStyle">
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
            props.min_expected_length > 0 &&
            props.modelValue.cards.length < props.min_expected_length
          "
          class="mt-5 text-orange-500 font-semibold"
        >
          <span class="underline">Warning:</span>
          Expecting {{ props.min_expected_length }} Cards
        </div>
        <div
          v-if="
            props.max_expected_length > 0 &&
            props.modelValue.cards.length > props.max_expected_length
          "
          class="mt-5 text-orange-500 font-semibold"
        >
          <span class="underline">Warning:</span>
          Expecting {{ props.max_expected_length }} Cards
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
  width: var(--editorWidth);
  opacity: 1;
  background-color: rgb(20, 20, 20);
  padding: 20px;
  border: 1px solid green;
}
</style>

<script setup lang="ts">
import { computed, defineComponent, ref } from 'vue';
import { CardList, useBoardStore } from '../stores/board';
import { cardText, parseCardString } from '@src/lib/utils';

import BoardSelectorCard from './BoardSelectorCard.vue';
import { PlayerState, usePlayerStore } from '@src/stores/player';
import { useCssVar } from '@vueuse/core';

const BOARD_EDITOR_PIXEL_WIDTH = 600;
const BOARD_EDITOR_PIXEL_HEIGHT = 400;
const BOARD_EDITOR_CSS_VAR_NAME = '--editorWidth';

interface Props {
  min_expected_length: number;
  max_expected_length: number;

  //Part of v-model
  modelValue: CardList;
}

const props = defineProps<Props>();

//The update part of v-model
const emit = defineEmits<{
  updateModelValue: [value: CardList];
}>();

//Be responsive to model changes, not sure if this is
//really needed
const cardListRef = computed(() => {
  const cardListComputed = props.modelValue;

  if (cardListComputed && !Array.isArray(cardListComputed.cards)) {
    cardListComputed.cards = [];
  }
  console.log('cardListComputed', cardListComputed.cards);
  return cardListComputed;
});

const isEditing = ref(false);

//Will be assigned the editor div
//const editor = ref(null);
const not_editing = ref(null);
const root_element = ref<HTMLDivElement | null>(null);
const width = useCssVar(BOARD_EDITOR_CSS_VAR_NAME, root_element);
width.value = BOARD_EDITOR_PIXEL_WIDTH + 'px';
const editorStyle = ref({});

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

function positionEditor() {
  //const editorWidth = useCssVar('--editorWidth', editor.value);
  //console.log('editorWidth', editorWidth.value);
  if (!root_element.value) {
    console.log('root_element.value is null');
    return;
  }
  //const computedStyles = getComputedStyle(root_element.value);

  const rect = root_element.value.getBoundingClientRect();

  const popupWidth = BOARD_EDITOR_PIXEL_WIDTH;
  const popupHeight = BOARD_EDITOR_PIXEL_HEIGHT;

  const extraWidth = 50;

  let top = rect.top + window.scrollY;
  let left = rect.left + window.scrollX;

  let right = left + popupWidth + extraWidth;

  //console.log('top', top);
  //console.log('left', left);

  // Adjust position to keep the popup on screen
  if (right > window.innerWidth) {
    left -= right - window.innerWidth;
  }
  if (top + popupHeight > window.innerHeight) {
    top -= top + popupHeight - window.innerHeight;
  }

  editorStyle.value = {
    position: 'fixed',
    left: `${left}px`,
    top: `${top}px`
  };
}

function toggleCard(cardId: number, updateText = true) {
  const cardList = cardListRef.value;
  if (cardList.cards.includes(cardId)) {
    //removes the card
    cardList.cards = cardList.cards.filter((card) => card !== cardId);
    if (updateText) {
      setBoardTextFromButtons();
    }
    return;
  }

  //If this puts us over the max, we remove a card
  if (cardList.cards.length + 1 > props.max_expected_length) {
    cardList.cards.shift();
  }

  //we should be under the limit

  if (cardList.cards.length >= props.max_expected_length) {
    return;
  }

  //adds the card

  cardList.cards.push(cardId);
  // if (cardList.cards.length <= 3) {
  //   cardList.cards.sort((a, b) => b - a);
  // }

  if (updateText) {
    setBoardTextFromButtons();
  }
}

function setBoardTextFromButtons() {
  const cardList = cardListRef.value;
  cardList.cardText = cardList.cards
    .map(cardText)
    .map(({ rank, suitLetter }) => rank + suitLetter)
    .join(', ');

  console.log('boardText.value', cardList.cardText);
}

function editDone() {
  isEditing.value = false;
}

function startEditing() {
  positionEditor();
  isEditing.value = true;
}

function onBoardTextChange() {
  const cardList = cardListRef.value;
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
  const cardList = cardListRef.value;
  cardList.cards = [];
  setBoardTextFromButtons();
}

function generateRandomBoard() {
  const cardList = cardListRef.value;
  cardList.cards = [];

  while (cardList.cards.length < props.max_expected_length) {
    const randomCard = Math.floor(Math.random() * 52);
    if (!cardList.cards.includes(randomCard)) {
      cardList.cards.push(randomCard);
    }
  }

  cardList.cards.sort((a, b) => b - a);
  setBoardTextFromButtons();
}
</script>
../lib/utils
