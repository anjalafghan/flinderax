import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from "path"
import { visualizer } from "rollup-plugin-visualizer"
import { compression } from 'vite-plugin-compression2'
import InjectPreload from 'unplugin-inject-preload/vite'

import tailwind from '@tailwindcss/vite'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const isProd = mode === 'production'

  return {
    plugins: [
      react(),
      tailwind(),
      InjectPreload({
        files: [
          {
            outputMatch: /inter-latin(-ext)?-(400|500|600|700)-normal.*\.woff2$/,
            attributes: {
              type: 'font/woff2',
              as: 'font',
              crossorigin: 'anonymous',
            },
          },
        ],
      }),
      isProd && visualizer({
        open: false,
        filename: "bundle-analysis.html",
        gzipSize: true,
        brotliSize: true,
      }),
      isProd && compression({
        algorithms: ['brotliCompress'],
        exclude: [/\.(br)$/, /\.(gz)$/],
      }),
      isProd && compression({
        algorithms: ['gzip'],
        exclude: [/\.(br)$/, /\.(gz)$/],
      }),
    ].filter(Boolean) as any,
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
    },
    build: {
      target: 'esnext',
      minify: 'terser',
      terserOptions: {
        compress: {
          drop_console: true,
          drop_debugger: true,
          pure_funcs: ['console.log', 'console.info', 'console.debug'],
          passes: 2,
        },
        format: {
          comments: false,
        },
        mangle: {
          safari10: true,
        },
      } as any,
      rollupOptions: {
        output: {
          manualChunks: {
            'react-vendor': ['react', 'react-dom', 'react-router-dom'],
            'query-vendor': ['@tanstack/react-query'],
            'ui-vendor': ['framer-motion', 'lucide-react', 'sonner'],
            'api-vendor': ['axios'],
          },
        },
      },
      chunkSizeWarningLimit: 1000,
      reportCompressedSize: true,
      cssMinify: 'lightningcss',
    },
  }
})
