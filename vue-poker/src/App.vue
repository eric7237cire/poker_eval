<script setup lang="ts">
import BoardSelectorCard from './components/BoardSelectorCard.vue'
import HelloWorld from './components/HelloWorld.vue'
import BoardSelector from './components/BoardSelector.vue'
import TheWelcome from './components/TheWelcome.vue'
</script>

<template>
  <div>
    <img alt="Vue logo" class="logo" src="./assets/logo.svg" width="125" height="125" />

    <div class="wrapper">
      <HelloWorld msg="You did it!" />
    </div>
  </div>

  <div class="flex-grow my-4 px-6 pt-2 overflow-y-auto" style="height: calc(100% - 2rem)">
    board23
    <BoardSelector />
    the board
  </div>
</template>

<style scoped>
header {
  line-height: 1.5;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    place-items: center;
    padding-right: calc(var(--section-gap) / 2);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }
}
</style>
