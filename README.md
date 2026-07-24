# Kimi Plan Monitor

一个基于 **Tauri 2.x + Vue 3 + Vite** 的 Windows 桌面小组件，用于实时监控 Kimi Code Plan 的用量情况。

它可以常驻系统托盘、置顶显示周用量和 5 小时窗口用量，支持贴边自动收起、深色/浅色主题切换，以及本地配置文件持久化。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.x-blue)
![Vue](https://img.shields.io/badge/Vue-3.x-green)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

## ✨ 功能特性

- **实时用量监控**：显示周用量和 5 小时窗口用量的百分比、剩余量和重置时间
- **自动轮询**：每 60 秒自动刷新一次数据，也可手动刷新
- **系统托盘常驻**：最小化到托盘，支持显示/隐藏、刷新、设置、退出
- **贴边自动收起**：窗口拖到屏幕边缘（或超出屏幕）时自动收起为一条细边，鼠标靠近时平滑展开
- **窗口位置记忆**：关闭或退出时保存窗口位置，下次启动自动恢复到原位
- **设置窗口独立**：Token、贴边收起开关都在独立设置窗口中配置，排版工整

## 📦 快速开始

### 环境要求

- Node.js 22.23+
- Rust（Tauri 构建需要）
- Windows 10/11

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 打包绿色 exe

```bash
npm run pack
```

输出位置：`src-tauri/target/release/kimi-plan-monitor.exe`

### 类型检查

```bash
npm run typecheck
```

## ⚙️ 配置说明

应用配置保存在本地 `config.json` 中，路径通常为：

```
%APPDATA%/com.kimi-plan-monitor/config.json
```

包含以下字段：

| 字段 | 说明 |
|------|------|
| `token` | Kimi Code Plan 的 API Token |
| `auto_hide_enabled` | 是否启用贴边自动收起 |
| `theme` | 主题（`dark` / `light`） |
| `window_position` | 主窗口上次关闭时的位置 |

### 获取 API Token

1. 登录 Kimi Code Plan 控制台
2. 生成 API Token
3. 在应用设置窗口中粘贴并保存

> 首次启动未配置 Token 时，会自动打开设置窗口。

## 🎨 界面预览

### 主窗口

- 顶部显示应用图标和名称、上次刷新时间
- 两行用量条：周用量（📅）和 5 小时用量（⏱️）
- 右上角支持切换主题和手动刷新

### 设置窗口

- API Token 输入框
- 贴边自动收起开关
- 主题切换按钮
- 配置文件路径显示

## 🛠️ 技术栈

| 前端 | 后端 | 工具链 |
|------|------|--------|
| Vue 3 + `<script setup>` | Rust + Tauri 2.x | Vite 8 |
| Tailwind CSS 4 | tokio + reqwest | TypeScript 7 |
| Tauri API (window/tray/event) | serde + tauri-plugin-store | vue-tsc |

## 📁 项目结构

```
kimi-plan-monitor/
├── src/                    # Vue 前端源码
│   ├── App.vue             # 主窗口组件
│   ├── SettingsApp.vue     # 设置窗口组件
│   ├── components/
│   │   └── UsageLine.vue   # 用量显示行组件
│   ├── settings.ts         # 设置窗口入口
│   └── style.css           # 全局样式 + 主题变量
├── src-tauri/              # Rust 后端源码
│   ├── src/
│   │   ├── lib.rs          # Tauri 入口、命令、托盘、事件
│   │   ├── api.rs          # Kimi API 请求与数据结构
│   │   └── autohide.rs     # 贴边自动收起逻辑
│   ├── icons/              # 应用图标（SVG + 生成的 PNG/ICO）
│   ├── capabilities/       # Tauri 权限配置
│   └── tauri.conf.json     # Tauri 应用配置
├── index.html              # 主窗口入口
├── settings.html           # 设置窗口入口
└── package.json
```

## 🔒 安全说明

- API Token 仅保存在本地 `config.json` 中，不会提交到 git 仓库
- 项目 `.gitignore` 已包含 `config.json` 和 `*.token` 等敏感文件
- 如果你曾将 Token 提交到历史记录，请执行 `git filter-branch` 或 `git filter-repo` 清理

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 开源。

## 🙏 致谢

- [Tauri](https://tauri.app/) - 轻量跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Tailwind CSS](https://tailwindcss.com/) - 实用优先的 CSS 框架

---

如果这个项目对你有帮助，欢迎 Star ⭐ 或提交 Issue / PR！
