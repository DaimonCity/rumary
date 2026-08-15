import {defineConfig} from 'vite';
// @ts-ignore
import react from '@vitejs/plugin-react';

export default defineConfig({
    plugins: [react()],
    server: {
        host: '0.0.0.0',
        port: 5718,
        proxy: {'/api': 'http://localhost:3000', '/health': 'http://localhost:3000'},
        allowedHosts: ['rumary.lekraft.org']
    }
});
