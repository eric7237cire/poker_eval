import './assets/main.css';

import { createApp } from 'vue';

import { createPinia } from 'pinia';
import App from './App.vue';

import 'vuetify/styles';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { createRouter, createWebHashHistory, createWebHistory } from 'vue-router';

import HandHistorySelectorVue from './components/HandHistorySelector.vue';
import HandHistoryVue from './components/HandHistory.vue';

const vuetify = createVuetify({
  components,
  directives
});

const routes = [
  { path: '/', component: App },
  { path: '/hh', component: HandHistorySelectorVue },
  { path: '/hh/:file_name', component: HandHistoryVue }
  // { path: '/about', component: About },
];

// 3. Create the router instance and pass the `routes` option
// You can pass in additional options here, but let's
// keep it simple for now.
const router = createRouter({
  // 4. Provide the history implementation to use. We are using the hash history for simplicity here.
  //history: createWebHashHistory(),
  history: createWebHistory("/poker_eval/"),
  routes, // short for `routes: routes`
  
});

const app = createApp({});

app.use(router);

app.use(createPinia()).use(vuetify);

app.mount('#app');
