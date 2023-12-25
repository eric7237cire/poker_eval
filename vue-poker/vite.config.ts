import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
//import { ViteRsw } from 'vite-plugin-rsw';
import wasm from "vite-plugin-wasm";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    wasm(),
    //ViteRsw(),
  ],
  //https://stackoverflow.com/questions/74769990/how-to-override-the-rollup-output-format-setting-in-vite
  worker: {
    format: "es"
  },
  build: {
    rollupOptions: {
      
    }
  },
  base: './',
  resolve: {
    alias: {
      //'@': fileURLToPath(new URL('./src', import.meta.url)),
      '@src': fileURLToPath(new URL('./src', import.meta.url)),
      '@pkg': fileURLToPath(new URL('./pkg', import.meta.url))
    }
  }
})
