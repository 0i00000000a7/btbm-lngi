import { createVaporApp } from 'vue';
import Dev from './Dev.vue';
import './dev.css';
import { initWasm } from '../core/wasm';

await initWasm();

createVaporApp(Dev).mount('#dev');
