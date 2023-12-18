<template>
  {{ buildInfo }}
</template>
<script setup lang="ts">
import { ref } from 'vue';

const buildInfo = ref<string>('');
const build_info_url = './build_info.txt';
//console.log("build_info_url: " + build_info_url);
const r = await fetch(build_info_url);
//console.log("r: " + r);
const text = await r.text();
//console.log("text: " + text);
buildInfo.value = text;
</script>
