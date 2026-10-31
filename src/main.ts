import { createVaporApp } from 'vue';
import App from './App.vue';
import { initWasm } from './core/wasm';
import 'katex/dist/katex.min.css';
import './styles.css';

await initWasm();

createVaporApp(App).mount('#app');
