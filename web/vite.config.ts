import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from "path"
import { visualizer } from "rollup-plugin-visualizer"
import { compression } from 'vite-plugin-compression2'
// import InjectPreload from 'unplugin-inject-preload/vite'

import tailwind from '@tailwindcss/vite'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const isProd = mode === 'production'
  const apiUrl = isProd ? 'https://flinderax-backend.fly.dev' : 'http://0.0.0.0:3000'

  return {
    plugins: [
      react(),
      tailwind(),
      /* InjectPreload({
        files: [
          {
            outputMatch: /inter-latin(-ext)?-(400|700)-normal.*\.woff2$/,
            attributes: {
              type: 'font/woff2',
              as: 'font',
              crossorigin: 'anonymous',
            },
          },
        ],
      }), */
      {
        name: 'html-transform',
        transformIndexHtml(html: string) {
          return html.replace(/__API_URL__/g, apiUrl)
        }
      },
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
          pure_funcs: ['console.log', 'console.info', 'console.debug', 'console.warn'],
          passes: 3,
          ecma: 2020,
          module: true,
          toplevel: true,
        },
        format: {
          comments: false,
          ascii_only: true,
        },
        mangle: {
          safari10: true,
          toplevel: true,
        },
      } as any,
      rollupOptions: {
        output: {
          manualChunks(id) {
            // Group large core libraries
            if (id.includes('node_modules')) {
              if (id.includes('react') || id.includes('react-dom') || id.includes('react-router')) {
                return 'react-core';
              }
              if (id.includes('@tanstack')) {
                return 'query-core';
              }
              if (id.includes('framer-motion') || id.includes('lucide-react')) {
                return 'ui-libs';
              }
              // Catch-all for other small modules to prevent too many small files
              return 'vendor-misc';
            }
          },
        },
      },
      chunkSizeWarningLimit: 600,
      reportCompressedSize: false,
      cssMinify: 'lightningcss',
    },
  }
})
