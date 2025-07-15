import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
  build: {
    target: 'esnext',
  },
  server: {
    port: 5173,
    https: {
      cert: '../deploy/localhost+2.pem',
      key: '../deploy/localhost+2-key.pem',
    },
  },
  optimizeDeps: {
    exclude: ['boid-wars-wasm'],
  },
});