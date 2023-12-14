<script lang="ts">
//import init, {greet, hello, initSync} from "rsw-hello";
import { ref } from 'vue';
import { init, handler } from '../global-worker';

export default {
  setup() {
    const count = ref(0);
    const whatup = ref('oeu');
    // expose to template and other options API hooks
    return {
      count,
      whatup
    };
  },

  async mounted() {
    console.log(this.count); // 0
    console.log(`the component is now mounted.`);
    await init(3);
    this.whatup = 'Start...';
    this.whatup += await handler!.sayHello('42.73');
    this.whatup += await handler!.sayGameHello(' boo');
    this.count += 17;
  }
};
</script>

<template>
  <div class="greetings">
    <h1 class="green">whut{{ whatup }}</h1>
  </div>
</template>

<style scoped>
h1 {
  font-weight: 500;
  font-size: 2.6rem;
  position: relative;
  top: -10px;
}

h3 {
  font-size: 1.2rem;
}

.greetings h1,
.greetings h3 {
  text-align: center;
}

@media (min-width: 1024px) {
  .greetings h1,
  .greetings h3 {
    text-align: left;
  }
}
</style>
