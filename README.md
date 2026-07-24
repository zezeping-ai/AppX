# AppX

- 托盘（关闭窗口隐藏到托盘，左键恢复，右键菜单）
- 自动更新（`tauri-plugin-updater` + GitHub Releases）
- GitHub Actions 多平台发布（macOS / Windows / Ubuntu，x64 + arm64）

## MacOS安装打不开

```bash
sudo xattr -d com.apple.quarantine /Applications/AppX.app
```

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm tauri build
```

## 自动更新配置

1. 生成本地签名密钥（私钥不会提交到仓库）：

```bash
mkdir -p src-tauri/keys
CI=1 pnpm tauri signer generate -w src-tauri/keys/updater.key -f --ci -p <your-password>
```

2. 将 `src-tauri/keys/updater.key.pub` 中的公钥写入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`。

3. 在 GitHub 仓库 Settings → Secrets 中添加：

- `TAURI_SIGNING_PRIVATE_KEY`：`updater.key` 文件**完整单行内容**（`cat src-tauri/keys/updater.key`，不要多空格或换行）
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：生成密钥时 `-p` 使用的密码（与私钥完全一致，不要首尾空格）

本地可先验证密钥与密码是否匹配，再写入 Secrets：

```bash
./scripts/verify-updater-signing.sh
```

4. 推送 tag 触发发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

CI 会构建各平台安装包，合并 `latest.json` 并创建 GitHub Release。客户端通过「帮助 → 检查更新」拉取最新版本。

## 项目结构

```
src/                    # Vue 3 前端（@/ 别名、tsx、pinia、router）
src-tauri/src/app/      # Rust 业务模块（menu / tray / windows / updates）
.github/workflows/      # Release CI
```

详细编码规范见 [AGENTS.md](./AGENTS.md)。
