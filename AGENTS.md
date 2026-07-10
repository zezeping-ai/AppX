# 项目背景
- AppX 桌面应用（Tauri + Vue 3）

# 代码规范
- 必须添加必要注释，不写废话
- 优先现代语法，不兼容老旧环境
- 代码结构清晰，注意代码重构和可用性、可维护性
- 代码风格：简洁、紧凑、无冗余

# 项目规则（每次对话自动生效）
## 前端架构
```
src/
├── router/              # 路由聚合
│   ├── index.ts         # createRouter + 守卫 + 对外 barrel
│   ├── pages.ts         # 主窗口 Layout 子路由
│   ├── standalone.ts    # 独立窗口 / 全屏流程页
│   ├── events.ts        # 仅 router 内部使用，经 index.ts 对外导出
│   └── types.ts         # pages / standalone 共用类型
├── pages/<name>/        # 路由页面（index.vue + 页面私有 components/composables）
├── features/<name>/     # 非路由能力（settings Section、布局组件等）
│   ├── components/      # 功能内复用 UI（如 PreferencesLayout）
│   └── settings/        # 偏好设置 Section
├── modules/<domain>/    # Tauri IPC 客户端 + 类型（与 Rust app/<domain> 对应）
├── models/              # SQLite ActiveRecord 模型
├── components/          # 跨功能复用的通用组件
├── hooks/               # 跨功能 UI 基础设施
└── shared/              # 纯工具函数
```
- 新增主窗口页面：在 `pages/<name>/` 创建，在 `router/pages.ts` 注册
- 新增独立窗口/流程页：在 `pages/<name>/` 创建，在 `router/standalone.ts` 注册
- `features/` 不放 routes、不放页面入口；偏好设置各 Tab 用 Section 组件
- **目录 + `index.ts` 仅用于有子文件时**（如 `pages/editor/index.vue` + `components/`）；单文件模块直接用 `foo.ts`，不要包成 `foo/index.ts`
- **私有依赖跟父模块走**：若 `b.ts` 只服务于 `a`（仅被 `a` 引入），应放在 `a/` 目录内，而不是与 `a` 同级。例如 `a/index.ts` + `a/b.ts`，而非根目录的 `a.ts` + `b.ts`。对外只从 `a/index.ts`（或包入口）导出
  - ✅ `shared/tauri/index.ts` + `shared/tauri/invoke.ts`（invoke 仅 tauri 入口使用）
  - ✅ `modules/crypto/index.ts` + `modules/crypto/client.ts`
  - ✅ `pages/code-snippets/composables/useCodeSnippetList.ts` + `normalizeGlobalShortcut.ts`
  - ❌ 单文件 `events.ts` 不要做成 `events/index.ts`；❌ 仅为 index 服务的 `routes.ts` 应内联进 `index.ts` 或并入父目录

## 前端
1. 修改项目支持 `@/` 目录引入
2. 引入 vueuse、lodash-es、sass、@vitejs/plugin-vue-jsx、tailwindcss、vue-router 和 pinia、ant-design-vue、@iconify/vue
3. ant-design-vue 默认最小尺寸，图标库统一用 @iconify/vue
4. template lang 默认 tsx，支持在 setup 中写 tsx 渲染
5. 前端拆分组件：若 `XXX.vue` 需要再拆分则变成 `XXX/index.vue`，自身依赖或独享子组件放在当前目录下
6. 前端 composable 同理：`useXXX.ts` 需拆分时变成 `useXXX/index.ts`，自身依赖或独享 composable 放在当前目录下

## Rust 端
1. 编写 GitHub workflow，支持通过 git tag（如 `v0.0.1`）自动构建 macOS x64/arm、Windows x64/arm、Ubuntu x64/arm
2. 引入托盘功能，主窗口点击关闭不退出程序，左键点击托盘重新显示窗口，右键菜单显示主窗口
3. 引入 tauri-plugin-updater，帮助菜单提供「检查更新」，自动检测 GitHub 最新 release 并下载安装
4. Rust 拆分模块：若 `XXX.rs` 需要拆分则变成 `XXX/mod.rs`，自身独享依赖放在文件夹内
5. 涉及更新器（updater）改动时，始终同步检查：`pubkey`、签名私钥来源、release 产物匹配关系
