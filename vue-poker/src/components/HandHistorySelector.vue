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
    import {json_filenames} from '../assets/hand_history/overview.json';

    console.log("History", json_filenames);
</script>