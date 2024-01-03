<template>
  <div ref="root_element">
    <template v-if="isVisible">
      <div @click="isVisible = false" ref="popup" class="popup" :style="popupStyle">
        Yo
        <table>
          <tr v-for="row in 13" :key="row" class="h-9">
            <td
              v-for="col in 13"
              :key="col"
              :style="cellStyle(row, col)"
              :title="cellPerc(row, col)"
              class="relative w-[2.625rem] border border-black bg-no-repeat"
            >
              {{ cellText(row, col) }}
            </td>
          </tr>
        </table>
      </div>
    </template>
    <template v-else>
      <div @click="setVisible(true)" class="button">Details</div>
    </template>
  </div>
</template>

<style lang="postcss" scoped>
.popup {
  background: gray;
  z-index: 100;

  table {
    margin-left: 20px;
  }
}

.button {
  cursor: pointer;
}
</style>

<script setup lang="ts">
import { ranks } from '@src/utils';
import { ref } from 'vue';

const props = defineProps<{
  range_equity: Array<number | null>;
  range_it_num: Array<number>;
}>();

const root_element = ref<HTMLDivElement | null>(null);

const isVisible = ref(false);

const popupStyle = ref({});

const POPUP_PIXEL_WIDTH = 590;
const POPUP_PIXEL_HEIGHT = 530;

//console.log("My eq is ", Array.from(props.range_equity));

function cellText(row1: number, col1: number) {
  const r1 = 13 - Math.min(row1, col1);
  const r2 = 13 - Math.max(row1, col1);

  /*
  const row = row1 - 1;
  const col = col1 - 1;
  const index = row * 13 + col;
  
  const equity = props.range_equity[index];

  if (equity !== null) {
    const eqPercent = (equity * 100).toFixed(1) + "%";
    return eqPercent;
  }*/

  return ranks[r1] + ranks[r2] + ['s', '', 'o'][Math.sign(row1 - col1) + 1];
}

function cellPerc(row1: number, col1: number): string | undefined {
  const r1 = 13 - Math.min(row1, col1);
  const r2 = 13 - Math.max(row1, col1);

  const row = row1 - 1;
  const col = col1 - 1;
  const index = row * 13 + col;

  const equity = props.range_equity[index];

  const itNum = props.range_it_num[index];

  if (equity !== null) {
    const eqPercent = (equity * 100).toFixed(1) + `% for ${itNum} iterations`;
    return eqPercent;
  }

  return `${itNum} iterations`;

  return undefined;
}

function cellStyle(row1: number, col1: number) {
  const row = row1 - 1;
  const col = col1 - 1;
  const index = row * 13 + col;
  const equity = props.range_equity[index];

  //console.log('index', index);
  // console.log('equity', equity);

  if (equity === null) {
    return {
      'background-color': 'gray'
    };
  }

  const eqPercent = (equity * 100).toFixed(1) + '%';

  if (equity < 0.5) {
    return {
      // "background-color": 'rgba(55,0,0,0.5)',
      'background-color': 'black',
      'background-image': 'linear-gradient(to right, rgba(255, 0, 0, 1), rgba(125, 0, 0, 1))',
      'background-size': `${eqPercent} 100% `
    };
  } else {
    const eqPercent = (equity * 100).toFixed(1) + '%';
    return {
      'background-color': 'black',
      'background-image': 'linear-gradient(to right, rgba(0, 125, 0, 1), rgba(0, 255, 0, 1))',
      'background-size': `${eqPercent} 100% `
    };
  }
}

function setVisible(value: boolean) {
  if (value) {
    positionPopup();
  }

  isVisible.value = value;
}

function positionPopup() {
  //const editorWidth = useCssVar('--editorWidth', editor.value);
  //console.log('editorWidth', editorWidth.value);
  if (!root_element.value) {
    console.log('root_element.value is null');
    return;
  }
  //const computedStyles = getComputedStyle(root_element.value);

  const rect = root_element.value.getBoundingClientRect();

  const popupWidth = POPUP_PIXEL_WIDTH;
  const popupHeight = POPUP_PIXEL_HEIGHT;

  const extraWidth = 50;

  let top = rect.top + window.scrollY - POPUP_PIXEL_HEIGHT / 2;
  let left = rect.left + window.scrollX - POPUP_PIXEL_WIDTH / 2;

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
  if (top < 0) {
    top = 50;
  }
  if (left < 0) {
    left = 50;
  }

  popupStyle.value = {
    position: 'fixed',
    left: `${left}px`,
    top: `${top}px`,
    width: `${popupWidth}px`,
    height: `${popupHeight}px`
  };
}
</script>
