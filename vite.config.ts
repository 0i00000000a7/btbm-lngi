import { defineConfig, loadEnv } from 'vite';
import vue from '@vitejs/plugin-vue';
import { milestoneRebuild } from './src/dev/rebuild.ts';

export default defineConfig(({ mode }) => {
    // GitHub Pages serves a project site from /<repo>/. The deploy workflow
    // passes that path in as VITE_BASE (via actions/configure-pages); locally
    // it is unset, so the app is served from the root.
    const env = loadEnv(mode, '.');
    const base = env.VITE_BASE || '/';

    return {
        base: base.endsWith('/') ? base : `${base}/`,
        plugins: [vue(), milestoneRebuild()],
        server: { host: '127.0.0.1', port: 5173 },
        build: { target: 'es2022' },
    };
});
