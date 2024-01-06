<template>
    Hand History for {{ file_name  }}
</template>

<script setup lang="ts">
    import { useRoute } from 'vue-router';

const route = useRoute();  
const file_name = route.params.file_name; // read parameter id (it is reactive) 

console.log(`File name [${file_name}]`);

fetch(`/src/assets/hand_history/${file_name}`)
    .then(response => response.json())
    .then(data => {
        console.log(data);
    });
//import(`@src/assets/hand_history/${file_name}`).then((module) => {
//import(`../assets/hand_history/${file_name}`).then((module) => {
                //this below are working just fine, so json is parsed OK.
  //              console.log("My module", module);
//});

</script>