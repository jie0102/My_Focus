import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig({
  // 防止vite在运行时清除屏幕
  clearScreen: false,
  
  // 设置根目录
  root: './src',
  
  // Tauri期望固定端口，如果该端口不可用则失败
  server: {
    port: 1420,
    strictPort: true,
  },
  
  // 在开发环境中使用env vars前缀VITE_，在生产环境中TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE, TAURI_DEBUG将被替换
  envPrefix: ['VITE_', 'TAURI_'],
  
  build: {
    // Tauri支持es2021
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // 在debug构建中不minify
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // 在debug构建中产生sourcemaps
    sourcemap: !!process.env.TAURI_DEBUG,
    
    // 设置输出目录
    outDir: '../dist',
    
    // 静态资源处理
    assetsDir: 'assets',
    
    // 清空输出目录
    emptyOutDir: true,
    
    // 构建入口配置
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/index.html')
      }
    }
  },
  
  // 基础路径配置
  base: './',
  
  // 静态资源目录
  publicDir: resolve(__dirname, 'src/assets'),
}) 