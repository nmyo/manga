# JM Boom (Android) Project Notes

Fork 自 [ppxb/jm-boom](https://github.com/ppxb/jm-boom)，目标平台改为 Android 移动端。

## 技术栈

- Tauri 2 + React 19 + Vite + TanStack Router/Query + Zustand + shadcn/ui
- Rust 后端：reqwest、image decode、SQLite
- 构建：GitHub Actions CI（Swatinem/rust-cache、NDK 缓存、pnpm 缓存）
- 签名：GitHub Secrets 存储 keystore（KEYSTORE_BASE64 / KEY_ALIAS / KEY_PASSWORD / KEYSTORE_PASSWORD）

## 与上游的差异

### 移动端适配
- 底部导航栏替代左侧浮动栏（fixed bottom-0，常驻显示）
- safe-area-inset-top 适配刘海屏，safe-area-inset-bottom 适配底部手势区
- `_app.tsx` 统一处理固定布局：`fixed inset-0` + `flex-1 overflow-y-auto`
- 所有页面 padding 响应式：`p-4 md:p-[32px_32px_16px_96px]`
- 漫画网格响应式：`grid-cols-2 sm:grid-cols-3 md:grid-cols-4`
- 详情页封面和标题响应式
- 阅读页底部栏/顶部栏适配手机

### 设置页精简
- 移除 DiagnosticsSection（调试日志）
- 移除缓存路径显示和打开文件夹按钮（Android 不支持）
- 更新链接指向 `nmyo/manga`（原为 ppxb/jm-boom）

### 编译优化
- release profile: `opt-level=s`, `lto=false`, `codegen-units=16`, `strip=true`
- CI 缓存：Swatinem/rust-cache（shared-key: android-arm64）、NDK 缓存、pnpm store 缓存
- 首次约 6 分钟，缓存后约 3 分钟

### 其他改动
- 默认缓存大小 128MB（原 512MB）
- 禁用 overscroll 弹性效果（消除滑动提示）
- CSS 工具类：`.reader-no-overscroll`、`.scrollbar-none`、`.reader-no-select`、`.safe-area-bottom`

## 未解决的问题（待后续处理）

1. 首页内容可能无法无限滚动（需要确认 API 是否支持分页）
2. 每日推荐日期选择器在手机上太宽（Tabs + Select 并排显示）
3. 检查更新在 Android 上不工作（Tauri updater 不支持 Android APK 自动安装）
4. 清理缓存按钮在 Android 上可能无效（需要验证 Tauri API 兼容性）

## 发布流程

- 版本号同步：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`
- CI 自动构建签名 APK，下载 artifact 即可
- APK 通过 Telegram Bot 发送给用户

## 参考

- 上游项目：https://github.com/ppxb/jm-boom
- Fork 项目：https://github.com/nmyo/manga
