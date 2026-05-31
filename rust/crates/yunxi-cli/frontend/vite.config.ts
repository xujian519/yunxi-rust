import path from "path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

const isWindows = process.env.TAURI_ENV_PLATFORM === "windows"

// https://vite.dev/config/
export default defineConfig({
  base: './',
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  plugins: [
    react(),
    {
      name: 'tauri-html-fix',
      transformIndexHtml: {
        order: 'post',
        handler(html) {
          // Tauri 自定义协议下 crossorigin 会导致资源加载失败
          let out = html.replace(/ crossorigin/g, '')
          // 启动占位：若仍见白屏且无此文字，说明 index.html 本身未加载
          out = out.replace(
            '<div id="root"></div>',
            '<div id="root"><div style="padding:24px;font-family:system-ui;color:#6B6560">云熙加载中…</div></div>',
          )
          // script 放到 body 末尾，兼容部分 WebView
          const scriptRe =
            /<script type="module" src="(\.\/assets\/[^"]+\.js)"><\/script>\s*/g
          const scripts: string[] = []
          out = out.replace(scriptRe, (_m, src) => {
            scripts.push(`<script type="module" src="${src}"></script>`)
            return ''
          })
          if (scripts.length > 0) {
            if (out.includes('</body>')) {
              out = out.replace('</body>', `  ${scripts.join('\n  ')}\n  </body>`)
            } else {
              out = out.replace('</html>', `  ${scripts.join('\n  ')}\n</body>\n</html>`)
            }
          }
          return out
        },
      },
    },
  ],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    // esnext 在 Tauri WebView (Safari) 中可能导致 JS 静默失败 → 白屏
    target: isWindows ? 'chrome105' : 'safari14',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    commonjsOptions: {
      transformMixedEsModules: true,
    },
  },
  server: {
    port: 3000,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
