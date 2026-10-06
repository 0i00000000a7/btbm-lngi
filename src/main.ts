import { createApp } from 'vue';
import App from './App.vue';
import { initWasm } from './core/wasm';
import './styles.css';

await initWasm();

createApp(App).mount('#app');
