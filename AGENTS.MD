# JM Boom Project Notes

本文件给后续接手的 Codex 使用，目标是快速理解项目现状、约束和阅读器性能优化的下一步。

## 接手约定

- 项目是 Tauri 2 + React 19 + Vite + TanStack Router/Query + Zustand + shadcn/ui + Rust reqwest。
- 当前优先桌面端体验，小屏适配不是近期重点。
- 不要手动编辑 `src/routeTree.gen.ts`，路由树由 TanStack Router 插件、HMR 或 build 生成。
- 读取或写入包含中文的文件时必须显式使用 UTF-8，避免 Windows 终端默认编码导致乱码。
- 工具调用：可以并行读取文件或搜索；文件编辑必须单次调用 `apply_patch`，不要重复或并行套同一补丁；`cargo check`、`vite build`、`tsc` 等会占用输出目录或 build lock 的验证命令不要并行重复触发。
- 前端和 Rust 通信用 Tauri `invoke`，项目不使用 axios，新接口不要新增 axios 调用。
- UI 优先复用 shadcn/ui 已安装组件；需要新的 shadcn 组件时，告诉用户安装，不要自己重造复杂基础组件。
- 项目已有 `floating-nav`，不要新增传统 sidebar。
- Rust 依赖可直接添加并用 `cargo check` 下载；前端依赖不要直接安装，先告诉用户。
- 用户此前表示暂时不要纠结 `lint` ；不要因为 `lint` 结果扩大无关改动。

## 验证命令

- 常规前端验证优先跑 `bun run build`。
- 如果 Windows 环境没有 `bun`，跑：
  - `node_modules/.bin/tsc.exe --noEmit`
  - `node_modules/.bin/vite.exe build`
- 改 `src-tauri` Rust 代码后跑：
  - 工作目录 `src-tauri`
  - `cargo check`
- 只有在新增或调整测试时再跑相关 `cargo test`；不要并行跑多个 Rust/Vite 构建。

## 参考项目

- API / 数据源主要参考同级 `jm-mobile`。
- 漫画阅读、预览、排版可以参考同级 `jm-mobile`、`Breeze-plugin-JmComic`、`Breeze`。
- 当前判断：`jm-mobile` 更偏移动端；`Breeze` 更适合作为桌面/跨平台阅读器排版与翻页参考。

## 项目结构要点

- 前端路由在 `src/routes`。
- 普通页面在 `_app` 下，阅读页是独立路由 `src/routes/reader/$comicId.tsx`，不挂 `_app`，所以没有 `floating-nav`。
- 全局用户状态在 `src/stores/user-store.ts`。
- 设置状态在 `src/stores/settings-store.ts`，使用 Zustand persist，包含 API endpoint、图片线路、预载数量、缓存上限、代理和封面隐私模式；阅读预载数量当前可选 `6-9`，语义是当前页前后各预载 N 张。
- 阅读历史在 `src/stores/reading-history-store.ts`，persist key 是 `jm-boom-reading-history`。
- Rust API 已拆分：
  - `src-tauri/src/api.rs`：远程设置、搜索、首页、每周推荐、详情、评论、登录、签到等普通接口。
  - `src-tauri/src/reader.rs`：阅读 manifest、图片下载、解码、缓存、预取。
  - `src-tauri/src/lib.rs`：Tauri command 包装和 invoke handler 注册。

## 当前功能状态

- 首页：`src/routes/_app/index.tsx`，使用 `getHomeFeed`，信息流纵向展示，右侧轻量导航，含 BackTop。
- 每周推荐：`src/routes/_app/weekly.tsx`，使用 shadcn `Tabs`/`Select`，items 用 `ComicGrid`。
- 详情页：`src/routes/_app/comic/$comicId.tsx`，展示详情、章节、相关推荐、评论 Drawer；章节进入阅读页时传 `title`、`chapter`、`albumId`、`pageIndex: '0'`、`nextId`、`nextChapter`。
- 历史观看：`src/routes/_app/history.tsx`，以作品 `comicId` 为唯一键覆盖历史，`chapterId` 保存当前章节/read id；点击历史进入 `/reader/$comicId` 并带 `pageIndex` 恢复进度。
- 阅读页：`src/routes/reader/$comicId.tsx` + `src/features/reader/*`，`$comicId` 语义是 read/chapter id，作品 id 来自 search `albumId`。
- 设置页：`src/routes/_app/settings.tsx`，动态发现 API endpoint，支持 HTTP/SOCKS5 代理、主题、封面隐私、阅读线路、预载数量、缓存上限和缓存管理。
- 个人中心：`src/routes/_app/me.tsx`，登录态来自 Zustand，支持签到数据和签到操作。

## 阅读器行为

- 阅读页左右方向键翻页，`Esc` 返回，左右透明点击区也可翻页。
- 图片使用 `h-screen w-screen object-contain`，尽量占满视口但不裁切。
- 顶部 toolbar 鼠标移动时出现，短暂后隐藏；键盘操作时隐藏。
- 底部悬浮进度条显示页码；最后一页且存在下一章时显示“下一章”按钮。
- `pageIndex` search 作为初始页，`useReaderPages` 会规范化并在 manifest 返回页数后 clamp。
- 阅读进度只在已有章节标题且 `pageCount > 0` 时写入历史，避免保存无效页数。
- 当前主要优化目标仍是桌面单页阅读的首次准备和连续翻页体感；滚动模式、双页模式、Breeze 风格排版还未落地。

## Rust / Tauri API 要点

- 已注册主要 commands：`discover_api_endpoints`、`search_comics`、`get_home_feed`、`get_week_filters`、`get_week_items`、`get_comic_detail`、`get_comic_comments`、`login`、`get_sign_in_data`、`sign_in`、`clear_session`、`configure_network_proxy`、`get_reader_cache_stats`、`clear_reader_cache`、`open_reader_cache_dir`、`get_comic_read_manifest`、`get_comic_read_page`、`prefetch_comic_read_pages`。
- 移动端 API `tokenparam` 版本对齐 Breeze 插件为 `2.0.20`。
- API endpoint 动态发现从 `newsvr-2025.txt` 解密 host 池，合并本地 fallback 后测速；业务请求只规范化 http/https endpoint，空 endpoint 回退到 `https://www.cdnhth.club`。
- Rust 侧支持全局代理配置，`configure_network_proxy` 会重建共享 reqwest client；HTTP/SOCKS5 依赖 reqwest `socks` feature。
- 登录使用共享 reqwest client + cookie jar 维持会话，重新登录前和退出时调用 `clear_session`。
- 小说分组已过滤，当前只实现漫画图片阅读链路。

## 阅读器 Rust 当前实现

- `get_comic_read_manifest` 读取 `chapter_view_template`，用 `json5` 解析源端宽松 JS/JSON 对象。
- `get_comic_read_page` 下载图片、必要时解扰、写入 app cache 的 `reader` 目录，再通过 Tauri asset protocol 给前端显示。
- `tauri.conf.json` 已允许 `$APPCACHE/reader/**`。
- 缓存上限默认约 512MB，修剪目标约 82%。
- 非扰图直接缓存原始 bytes，不再整图解码；冷缓存返回宽高 `0/0`，前端当前不依赖宽高排版。
- 扰图缓存输出 lossy WebP，当前使用 `webp` crate / libwebp，质量 `75.0`，缓存文件仍为 `.webp`。
- 扰图解码先 `image::load_from_memory`，转 RGB 后按 raw row copy 重排；`reorder_ms` 通常只有数毫秒，瓶颈主要是源图 decode 的 `load_ms` 和 WebP `encode_ms`。
- 预取半径来自设置页“预载数量”，语义是当前页两侧各预取 N 张；前端 API 和 Rust command 的 `radius` 为必传，不再保留隐藏默认值。
- Rust 预取并发固定为 `1`，已和设置里的预取半径解耦；目标是避免后台多路 decode/encode 抢当前页 CPU。
- 预取顺序按近邻优先：`+1, -1, +2, -2...`。
- 前端 `useReaderPages` 会在当前页 ready 后触发预取；同一窗口内不会每页重复触发，移动距离达到 `ceil(prefetchCount / 2)` 后才开启下一轮窗口。
- Rust 侧有同缓存路径 single-flight 锁，避免同一页重复冷处理。
- Rust 侧有预取 generation 取消：新预取窗口会让旧窗口 stale，旧预取在拿锁后或下载后会跳过；如果 stale 预取已经下载完成且同一页有当前请求在等，则继续写缓存给当前请求复用，避免重复下载。
- 缓存写入使用同目录 `.tmp` 文件，完成后 rename 为正式缓存文件；缓存统计和清理跳过 `.tmp`，避免并发读取半写文件导致 `failed to fill whole buffer`。
- 当前阅读器性能日志较详细，包含 `origin=current|prefetch`、`lock_wait_ms`、`download_ms`、`cache_ms`、`load_ms`、`reorder_ms`、`encode_ms`、`write_ms`、`cleanup_ms`、`source_kb`、`output_kb`、`ratio`、`delta_kb`。

## 已落地的阅读器优化

- 预取有限并发：`prefetch_comic_read_pages` 使用 `JoinSet` 按近邻优先顺序预取，但后台并发固定为 `1`，不再随 `prefetchCount` 增大；已跑 `cargo check`。
- 预取窗口节流：`useReaderPages` 记录上次预取窗口，走到半个窗口阈值后才开启下一轮预取；已跑 `node_modules/.bin/tsc.exe --noEmit` 和 `node_modules/.bin/vite.exe build`。
- 预载数量调整：设置页预载数量从旧的 `0-6` 改为 `6-9`，默认 `6`；选项文案为“前后各 N 张”；`prefetchComicReadPages` / Tauri command / Rust reader 的 `radius` 已改为必填，删除隐藏默认半径；已跑 `node_modules/.bin/tsc.exe --noEmit`、`node_modules/.bin/vite.exe build` 和 `cargo check`。
- 非扰图快路径：不需要解扰时直接写原始 bytes；已跑 `cargo check`。
- 单页预热渲染：`useReaderPages` 去掉 120ms 翻页等待，当前页 query 立即跟随 `currentIndex`；当前页 ready 后 query 相邻前后页，并用浏览器 `Image.decode()` 预热本地 asset 图片；已跑 `node_modules/.bin/tsc.exe --noEmit` 和 `node_modules/.bin/vite.exe build`。
- 缓存写入原子化：缓存文件先写 `.tmp` 再 rename；已跑 `cargo check`。
- 扰图 raw buffer 重排：解扰时按 RGB 行复制，减少临时分配；已跑 `cargo check`。
- 扰图 WebP q75：参考 Breeze 的 libwebp 方向，引入 `webp = "0.3.1"`；JPEG q85 曾短暂实验但输出约为源图 `2.0x-2.5x`，已放弃；WebP q75 当前通常输出约为源图 `0.8x-0.9x`。
- 同页 single-flight：同一个缓存路径同时只允许一个冷处理任务，避免用户快速翻页或预热导致重复写同页缓存；已跑 `cargo check`。
- stale 预取取消与复用：新窗口会让旧预取 generation 过期；过期预取若未被当前页等待则跳过，若当前页正在等同一缓存路径则继续落盘供当前页命中；已跑 `cargo check`。
- 缓存性能日志：新增缓存体积比较和耗时拆分，`encode_write_ms` 已拆成 `encode_ms` 与 `write_ms`；日志显示磁盘写入通常不是瓶颈，主要成本在 `load_ms` 与 `encode_ms`；已跑 `cargo check`。

## 下一步建议

当前 WebP q75 + 预取并发固定 1 + 半窗口预取触发的体感较顺滑，建议先保留这一版继续观察真实阅读日志。

后续如果仍出现可感知卡顿，优先看日志里的 `origin=current`、`lock_wait_ms`、`load_ms`、`encode_ms`：

- 注意 Rust 日志里的 `origin=current` 目前包含前端相邻页预热请求，不一定等于屏幕可见页；如果要继续精确分析，可考虑前端给 `get_comic_read_page` 增加 `requestOrigin=visible|warm`。
- 如果 `lock_wait_ms` 高但用户体感顺滑，说明等待多半发生在预热请求上，不要急着改架构。
- 如果可见当前页仍频繁冷处理，可继续评估更快的 WebP 参数、降低质量、或 release 构建下的真实耗时。
- 当前日志较适合调参，但最终发布前应考虑加 debug 开关或收敛日志量。

## 性能优化重点工作

- 当前页与预取页分层，保证可见页优先返回，不被后台预热任务拖慢。
- 提前触发预取，不要等当前页完全 ready 才开始下一轮，尽量在接近页尾时就补后续页。
- 继续压低冷处理成本，重点看 `load_ms` 和 `encode_ms`，必要时调整 WebP 编码策略。
- 检查前端 `ReaderImage` 的 loading 状态切换，避免图片已可用但界面还在停顿。
- 拆分可见页与预热页的请求信号，避免预热挤占主线程和网络资源。
- 保持相邻页 warm up，但控制 CPU 峰值和缓存抖动。
- 补更细的阅读器性能日志，区分用户可见页和后台预热页，方便定位真实卡点。
- 后续再评估布局缓存、双页模式、滚动模式等更大范围的阅读体验改造。

后续排版方向参考 Breeze：单页 `contain` 当前已接近；滚动模式可学习 Breeze 的图片尺寸缓存，避免列表高度跳动；双页/RTL/LTR 支持尚未实现。

## 注意事项

- 历史观看依赖 `currentIndex`、`pageCount`、`chapterId`、`albumId` 等前端字段；优化图片解码和缓存时不要改 `get_comic_read_page` 的 `index` 语义、缓存路径命名规则或阅读页 search 参数。
- 阅读缓存路径规则目前是章节 read id 目录下 `0001.ext`；不要随意变更，否则热缓存命中和历史恢复体验会受影响。
- `src/features/reader/reader-bars.tsx` 可能有用户侧小改动，页码显示是 `{current} / {pageCount}`，不要无关回滚。
