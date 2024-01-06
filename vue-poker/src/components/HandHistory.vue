<template>
    Hand History for {{ file_name  }}
</template>

<script setup lang="ts">
    import { useRoute } from 'vue-router';

const route = useRoute();  
const file_name = route.params.file_name; // read parameter id (it is reactive) 

</script>