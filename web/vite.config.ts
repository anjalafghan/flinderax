import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from "path"

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      '/common': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
      '/user': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
      '/card': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
      '/health': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      }
    }
  }
})
