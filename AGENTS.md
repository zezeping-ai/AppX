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
├── router/              # 路由聚合；各功能在 features/<name>/routes.ts 定义
├── features/<name>/     # 功能模块（路由 + 页面 + 组件 + composables）
│   ├── routes.ts        # 导出 xxxRoutes
│   ├── pages/           # 路由入口页（薄壳）
│   ├── components/      # 仅本功能使用的 UI
│   └── index.ts         # 对外 barrel
├── modules/<domain>/    # Tauri IPC 客户端 + 类型（与 Rust app/<domain> 对应）
├── components/          # 跨功能复用的通用组件
├── hooks/               # 跨功能 UI 基础设施
└── shared/              # 纯工具函数
```
- 新增功能：创建 `features/<name>/`，定义 `routes.ts`，在 `router/index.ts` 注册
- 功能私有代码不放 `components/`；仅多功能共用时才提升为全局组件

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
