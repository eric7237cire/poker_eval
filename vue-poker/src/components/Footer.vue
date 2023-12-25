<template>
  {{ buildInfo }}
</template>
<script setup lang="ts">
import { ref } from 'vue';

interface BuildInfo {
    build_date: number,
    github_sha: string,
}

const buildInfo = ref<string>('');
const build_info_url = './build_info.txt';
//console.log("build_info_url: " + build_info_url);
const r = await fetch(build_info_url);
//console.log("r: " + r);
const buildInfoJson: BuildInfo = await r.json();
//console.log("text: " + text);
const options: Intl.DateTimeFormatOptions = { year: 'numeric', month: '2-digit', day: '2-digit', weekday: 'narrow' };
const formattedDate = new Date(buildInfoJson.build_date).toLocaleTimeString('fr-CH', options);
buildInfo.value = `Built: ${formattedDate}\nCommit Hash: ${buildInfoJson.github_sha}`;

</script>
