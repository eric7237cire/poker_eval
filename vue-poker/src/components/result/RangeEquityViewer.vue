<template>
    <div ref="root_element">
        
        <template v-if="isVisible" >
            <div @click="isVisible = false" ref="popup" class="popup" :style="popupStyle"
            
            >Yo</div>
        </template>
        <template v-else>
            <div @click="setVisible(true)" class="button"
            
            >Details</div>
        </template>
    </div>
</template>

<style lang="postcss" scoped>
.popup {
    background: green;
    z-index: 100;
}

.button {
    cursor: pointer;
}
</style>

<script setup lang="ts">
import { ref } from 'vue';


const props = defineProps<{
    range_equity: Array<number | null>
}>();

const root_element = ref<HTMLDivElement|null>(null);

const isVisible = ref(false);

const popupStyle = ref({});

const POPUP_PIXEL_WIDTH = 600;
const POPUP_PIXEL_HEIGHT = 400;

function setVisible(value: boolean) {
    if(value) {
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

  let top = rect.top + window.scrollY - POPUP_PIXEL_HEIGHT/2;
  let left = rect.left + window.scrollX - POPUP_PIXEL_WIDTH/2;

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
    position: "fixed",
    left: `${left}px`,
    top: `${top}px`,
    width: `${popupWidth}px`,
    height: `${popupHeight}px`,
  };
}

</script>