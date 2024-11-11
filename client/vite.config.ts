// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import { nodePolyfills } from 'vite-plugin-node-polyfills';
import { fileURLToPath } from 'url';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import tsconfigPaths from 'vite-tsconfig-paths';
import { viteStaticCopy } from 'vite-plugin-static-copy';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [
          ['@babel/plugin-transform-react-jsx', { runtime: 'automatic' }],
          ['@babel/plugin-syntax-import-attributes', { deprecatedAssertSyntax: true }]
        ],
      },
      jsxImportSource: '@emotion/react',
    }),
    nodePolyfills({
      globals: {
        Buffer: true,
        global: true,
        process: true
      },
      protocolImports: true
    }),
    wasm(),
    topLevelAwait(),
    tsconfigPaths(),
    viteStaticCopy({
      targets: [
        {
          src: 'src/wasm/*.wasm',
          dest: 'assets'
        }
      ]
    })
  ],

  build: {
    target: 'esnext',
    sourcemap: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html')
      },
      output: {
        format: 'es',
        chunkFileNames: 'assets/[name].[hash].js',
        entryFileNames: 'assets/[name].[hash].js',
        assetFileNames: 'assets/[name].[hash].[ext]'
      }
    }
  },

  optimizeDeps: {
    exclude: [
      '@ton/ton',
      'vite-plugin-wasm',
      '@/wasm/overpass_rs'
    ],
    esbuildOptions: {
      target: 'esnext',
      supported: {
        bigint: true
      }
    }
  },

  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Resource-Policy': 'cross-origin',
      'Content-Security-Policy': [
        "default-src 'self'",
        "script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' chrome-extension: blob:",
        "script-src-elem 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' chrome-extension: blob:",
        "worker-src 'self' blob: 'wasm-unsafe-eval'",
        "connect-src 'self' https://ton.access.orbs.network https://toncenter.com https://connect.tonhubapi.com ws: wss:",
      ].join('; ')
    }
  }
});