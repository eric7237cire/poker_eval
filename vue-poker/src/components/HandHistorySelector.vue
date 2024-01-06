<template>
    Hello
    <ul>
        <li v-for="file_name in json_filenames" :key="file_name">
            <router-link :to="{ path: '/hh/' + file_name, params: { file_name } }">
                {{ file_name }}
            </router-link>
        </li>
    </ul>
</template>

<script setup lang="ts">
import { ref } from 'vue';

    const json_filenames = ref([]);

    console.log("History", json_filenames);
    fetch('/src/assets/hand_history/overview.json')
        .then(response => response.json())
        .then(data => {
            console.log(data);
            json_filenames.value = data.json_filenames;
        });
</script>