# Kimi Code Plan 用量监控 - Windows 桌面小工具

## Context

用户需要一个 Windows 桌面小工具，用于实时监控 Kimi Code Plan 的用量情况。核心需求：
- 置顶小窗口，无边框，可拖动，可调节大小
- 显示每周用量和 5 小时用量（已用百分比、剩余百分比、重置时间）
- 右下角托盘图标常驻，包含退出菜单
- 界面精美，带粒子背景动画
- 手动刷新按钮
- 定时自动轮询 API

**API 信息：**
- Endpoint: `GET https://api.kimi.com/coding/v1/usages`
- Auth: Bearer Token (`***REDACTED-TOKEN***`)
- 响应包含 `usage`（周用量）和 `limits[0]`（5小时用量）

## 技术选型

**Tauri 2.x** —— 理由：
- 体积超小（~3-8MB），适合小工具
- 前端 HTML/CSS/JS，UI 天花板极高
- Rust 后端原生支持置顶、托盘、窗口管理
- 内置 HTTP client，定时轮询顺手
- 跨平台（未来可扩展到 macOS/Linux）

**对比其他方案：**
- WinForms：托盘/置顶更原生，但 UI 想做精美太费劲
- Electron：体积太大（100MB+），小工具不该这么重
- Flutter Desktop：桌面端支持不够成熟

## 架构设计

```
┌─────────────────────────────────────────┐
│           Kimi Plan Monitor             │
│  ┌─────────────┐    ┌─────────────────┐ │
│  │  Frontend   │◄──►│  Rust Backend   │ │
│  │  (HTML/JS)  │    │  (Tauri Core)   │ │
│  └─────────────┘    └─────────────────┘ │
│         │                    │          │
│    渲染 UI 界面         定时轮询 API      │
│    处理用户交互         管理托盘/窗口      │
└─────────────────────────────────────────┘
```

**技术栈：**
- 后端：Rust + Tauri 2.x
- 前端：Vue 3 + Vite（组件化开发）
- 存储：`tauri-plugin-store`（保存 Token、窗口位置）
- HTTP：`reqwest`

## UI 布局

```
┌─────────────────────────────────┐
│  ⚡ Kimi Plan          [🔄]    │  ← 标题栏（可拖动）+ 刷新按钮
│─────────────────────────────────│
│  📅 周用量  22% │ 78%  07-28 02:29:57 │
│  ⏱️ 5小时   0%  │100%  07-24 03:29:57 │
│─────────────────────────────────│
│  ✨ 12:34:56                     │
└─────────────────────────────────┘
```

**窗口属性：**
- 尺寸：默认 320×120，可拖拽调整
- 无边框（`decorations: false`）
- 置顶（`alwaysOnTop: true`）
- 跳过任务栏（`skipTaskbar: true`）
- 透明背景（`transparent: true`）

**数据展示：**
- 周用量：`usage.used / usage.limit` 算百分比，`usage.resetTime` 格式化
- 5小时用量：`limits[0]`（`window.duration=300`），算百分比
- 重置时间格式：`MM-DD HH:mm:ss`

**配色方案：**
- 背景：`#0a0a12`（深黑）+ 粒子动画
- 文字：`#e0e0ff`（淡紫白）
- 已用百分比：`#ff6b6b`（红色）
- 剩余百分比：`#51cf66`（绿色），**剩余 < 30% 时变为 `#ff6b6b`（红色）预警**
- 分隔符：`#4a4a6a`（暗紫）
- 日期时间：`#8a8aaa`（灰色）
- 粒子：青/蓝/紫随机，透明度 0.3-0.6

**交互：**
- 左键拖动：移动窗口
- 点击刷新按钮：立即调用 API，按钮旋转动画
- 双击托盘图标：显示/隐藏窗口
- 右键托盘图标：菜单（显示/隐藏、立即刷新、设置、退出）
- **贴边自动收起**：窗口贴近屏幕边缘时自动收起，只显示一条小边，鼠标靠近时展开（可在设置中开关）

**用量预警：**
- 当剩余百分比 < 30% 时，剩余百分比数字变为红色（`#ff6b6b`）
- 可以添加闪烁动画或图标提醒（⚠️）

**贴边自动收起：**
- 检测窗口位置，当贴近屏幕边缘（上下左右）时触发
- 收起时窗口缩小为一条 4px 宽的小边，显示在屏幕边缘
- 鼠标靠近小边时，窗口自动展开
- 鼠标离开窗口后，延迟 1 秒自动收起
- 可在设置中开启/关闭此功能

## 数据流

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│   前端 UI   │ ◄──► │  Tauri IPC   │ ◄──► │  Rust 后端  │
│  (HTML/JS)  │      │  (invoke)    │      │  (轮询器)   │
└─────────────┘      └──────────────┘      └─────────────┘
                                                   │
                                                   ▼
                                          ┌─────────────┐
                                          │  Kimi API   │
                                          │  (HTTPS)    │
                                          └─────────────┘
```

**轮询逻辑：**
1. 启动时：立即调用一次 `fetch_usage()`
2. 定时器：每 60 秒轮询一次 API
3. 缓存：将最新数据存储在 `Arc<Mutex<UsageData>>` 中
4. 通知：数据更新后通过 `emit` 事件推送到前端

**错误处理：**
- 网络失败：显示 `⚠️ 网络错误`，30 秒后重试
- Token 失效：显示 `❌ Token 无效`，不自动重试
- 数据解析失败：显示 `⚠️ 数据异常`，继续用缓存数据

## 项目结构

```
kimi-plan-monitor/
├── src-tauri/                 # Rust 后端
│   ├── src/
│   │   ├── main.rs            # 入口，初始化 Tauri
│   │   ├── api.rs             # API 调用逻辑
│   │   ├── tray.rs            # 托盘图标和菜单
│   │   └── state.rs           # 全局状态管理
│   ├── Cargo.toml             # Rust 依赖
│   └── tauri.conf.json        # Tauri 配置
├── src/                       # Vue 前端
│   ├── components/
│   │   ├── ParticleBackground.vue  # 粒子背景组件
│   │   └── UsageLine.vue           # 用量行组件
│   ├── App.vue                # 主组件
│   ├── main.js                # Vue 入口
│   └── style.css              # 全局样式
├── index.html                 # HTML 模板
├── package.json               # 前端依赖
├── vite.config.js             # Vite 配置
└── README.md
```

## 关键实现

### 1. Tauri 配置（`src-tauri/tauri.conf.json`）

```json
{
  "tauri": {
    "windows": [{
      "title": "Kimi Plan Monitor",
      "width": 320,
      "height": 120,
      "decorations": false,
      "alwaysOnTop": true,
      "resizable": true,
      "skipTaskbar": true,
      "transparent": true
    }],
    "systemTray": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```

### 2. API 调用（`src-tauri/src/api.rs`）

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageData {
    pub usage: Usage,
    pub limits: Vec<Limit>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Usage {
    pub limit: String,
    pub used: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Limit {
    pub window: Window,
    pub detail: Detail,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Window {
    pub duration: u32,
    #[serde(rename = "timeUnit")]
    pub time_unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Detail {
    pub limit: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}

pub async fn fetch_usage(token: &str) -> Result<UsageData, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.kimi.com/coding/v1/usages")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    response.json::<UsageData>().await
}
```

### 3. Vue 主组件（`src/App.vue`）

```vue
<template>
  <div class="app">
    <ParticleBackground />
    
    <div class="container">
      <div class="header">
        <span class="title">⚡ Kimi Plan</span>
        <button 
          class="refresh-btn" 
          :class="{ spinning: isRefreshing }"
          @click="handleRefresh"
        >
          🔄
        </button>
      </div>
      
      <UsageLine
        icon="📅"
        label="周用量"
        :used="weeklyUsed"
        :remaining="weeklyRemaining"
        :reset-time="weeklyResetTime"
      />
      
      <UsageLine
        icon="⏱️"
        label="5小时"
        :used="hourlyUsed"
        :remaining="hourlyRemaining"
        :reset-time="hourlyResetTime"
      />
      
      <div class="footer">
        <span class="update-time">✨ {{ lastUpdateTime }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import ParticleBackground from './components/ParticleBackground.vue';
import UsageLine from './components/UsageLine.vue';

const weeklyUsed = ref('--%');
const weeklyRemaining = ref('--%');
const weeklyResetTime = ref('--');
const hourlyUsed = ref('--%');
const hourlyRemaining = ref('--%');
const hourlyResetTime = ref('--');
const lastUpdateTime = ref('--:--:--');
const isRefreshing = ref(false);

const handleRefresh = async () => {
  isRefreshing.value = true;
  try {
    await invoke('refresh_usage');
  } finally {
    setTimeout(() => {
      isRefreshing.value = false;
    }, 600);
  }
};

const updateUsage = (data) => {
  // 计算周用量百分比
  const weeklyLimit = parseInt(data.usage.limit);
  const weeklyUsedCount = parseInt(data.usage.used);
  const weeklyUsedPercent = Math.round((weeklyUsedCount / weeklyLimit) * 100);
  const weeklyRemainingPercent = 100 - weeklyUsedPercent;
  
  weeklyUsed.value = `${weeklyUsedPercent}%`;
  weeklyRemaining.value = `${weeklyRemainingPercent}%`;
  weeklyResetTime.value = formatResetTime(data.usage.resetTime);
  
  // 计算5小时用量百分比
  const hourlyLimit = data.limits[0];
  const hourlyLimitCount = parseInt(hourlyLimit.detail.limit);
  const hourlyRemainingCount = parseInt(hourlyLimit.detail.remaining);
  const hourlyUsedCount = hourlyLimitCount - hourlyRemainingCount;
  const hourlyUsedPercent = Math.round((hourlyUsedCount / hourlyLimitCount) * 100);
  const hourlyRemainingPercent = 100 - hourlyUsedPercent;
  
  hourlyUsed.value = `${hourlyUsedPercent}%`;
  hourlyRemaining.value = `${hourlyRemainingPercent}%`;
  hourlyResetTime.value = formatResetTime(hourlyLimit.detail.resetTime);
  
  // 更新最后刷新时间
  const now = new Date();
  lastUpdateTime.value = now.toLocaleTimeString('zh-CN', { hour12: false });
};

const formatResetTime = (isoString) => {
  const date = new Date(isoString);
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${month}-${day} ${hours}:${minutes}:${seconds}`;
};

let unlisten;

onMounted(async () => {
  // 监听后端推送的用量更新事件
  unlisten = await listen('usage-updated', (event) => {
    updateUsage(event.payload);
  });
  
  // 初始加载
  handleRefresh();
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});
</script>

<style scoped>
.app {
  width: 100vw;
  height: 100vh;
  position: relative;
  overflow: hidden;
}

.container {
  position: relative;
  z-index: 1;
  padding: 12px;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  -webkit-app-region: drag;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.title {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0ff;
}

.refresh-btn {
  background: none;
  border: none;
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: background-color 0.2s;
  -webkit-app-region: no-drag;
}

.refresh-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.refresh-btn.spinning {
  animation: spin 0.6s linear;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.footer {
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.update-time {
  font-size: 11px;
  color: #8a8aaa;
}
</style>
```

### 4. 用量行组件（`src/components/UsageLine.vue`）

```vue
<template>
  <div class="usage-line">
    <span class="icon">{{ icon }}</span>
    <span class="label">{{ label }}</span>
    <span class="used">{{ used }}</span>
    <span class="separator">│</span>
    <span class="remaining" :class="{ warning: isWarning }">{{ remaining }}</span>
    <span class="reset">{{ resetTime }}</span>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  icon: String,
  label: String,
  used: String,
  remaining: String,
  resetTime: String,
});

// 判断是否预警（剩余 < 30%）
const isWarning = computed(() => {
  const percent = parseInt(props.remaining);
  return !isNaN(percent) && percent < 30;
});
</script>

<style scoped>
.usage-line {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  backdrop-filter: blur(10px);
}

.icon {
  font-size: 14px;
}

.label {
  color: #e0e0ff;
  font-weight: 500;
  min-width: 50px;
}

.used {
  color: #ff6b6b;
  font-weight: 600;
  min-width: 40px;
  text-align: right;
}

.separator {
  color: #4a4a6a;
}

.remaining {
  color: #51cf66;
  font-weight: 600;
  min-width: 40px;
  text-align: right;
  transition: color 0.3s;
}

.remaining.warning {
  color: #ff6b6b;
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.reset {
  color: #8a8aaa;
  margin-left: auto;
  font-size: 11px;
}
</style>
```

### 5. 贴边收起逻辑（`src-tauri/src/main.rs`）

```rust
use tauri::{Manager, Window, PhysicalPosition, PhysicalSize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// 贴边收起状态
pub struct AutoHideState {
    pub enabled: bool,
    pub is_hidden: bool,
    pub edge: Option<Edge>,
    pub original_size: Option<PhysicalSize<u32>>,
    pub original_position: Option<PhysicalPosition<i32>>,
}

#[derive(Clone, Copy)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl AutoHideState {
    pub fn new() -> Self {
        Self {
            enabled: true,
            is_hidden: false,
            edge: None,
            original_size: None,
            original_position: None,
        }
    }
}

// 检测窗口是否贴边
pub fn check_edge(window: &Window) -> Option<Edge> {
    let monitor = window.current_monitor().ok()??;
    let screen_size = monitor.size();
    let screen_pos = monitor.position();
    
    let win_pos = window.outer_position().ok()?;
    let win_size = window.outer_size().ok()?;
    
    const THRESHOLD: i32 = 10; // 贴边阈值（像素）
    
    // 检测左边缘
    if (win_pos.x - screen_pos.x).abs() < THRESHOLD {
        return Some(Edge::Left);
    }
    
    // 检测右边缘
    if ((win_pos.x + win_size.width as i32) - (screen_pos.x + screen_size.width as i32)).abs() < THRESHOLD {
        return Some(Edge::Right);
    }
    
    // 检测上边缘
    if (win_pos.y - screen_pos.y).abs() < THRESHOLD {
        return Some(Edge::Top);
    }
    
    // 检测下边缘
    if ((win_pos.y + win_size.height as i32) - (screen_pos.y + screen_size.height as i32)).abs() < THRESHOLD {
        return Some(Edge::Bottom);
    }
    
    None
}

// 收起窗口
pub async fn hide_window(window: &Window, edge: Edge, state: Arc<Mutex<AutoHideState>>) {
    let mut state = state.lock().unwrap();
    
    if state.is_hidden {
        return;
    }
    
    // 保存原始尺寸和位置
    if let (Ok(size), Ok(pos)) = (window.outer_size(), window.outer_position()) {
        state.original_size = Some(size);
        state.original_position = Some(pos);
        state.is_hidden = true;
        state.edge = Some(edge);
        
        // 根据边缘方向收起
        match edge {
            Edge::Left => {
                let _ = window.set_size(PhysicalSize::new(4, size.height));
            }
            Edge::Right => {
                let monitor = window.current_monitor().unwrap().unwrap();
                let screen_width = monitor.size().width;
                let _ = window.set_size(PhysicalSize::new(4, size.height));
                let _ = window.set_position(PhysicalPosition::new(
                    screen_width as i32 - 4,
                    pos.y,
                ));
            }
            Edge::Top => {
                let _ = window.set_size(PhysicalSize::new(size.width, 4));
            }
            Edge::Bottom => {
                let monitor = window.current_monitor().unwrap().unwrap();
                let screen_height = monitor.size().height;
                let _ = window.set_size(PhysicalSize::new(size.width, 4));
                let _ = window.set_position(PhysicalPosition::new(
                    pos.x,
                    screen_height as i32 - 4,
                ));
            }
        }
    }
}

// 展开窗口
pub async fn show_window(window: &Window, state: Arc<Mutex<AutoHideState>>) {
    let mut state = state.lock().unwrap();
    
    if !state.is_hidden {
        return;
    }
    
    if let (Some(size), Some(pos)) = (state.original_size, state.original_position) {
        let _ = window.set_size(size);
        let _ = window.set_position(pos);
        state.is_hidden = false;
        state.edge = None;
    }
}

// 启动贴边检测任务
pub fn start_auto_hide_task(window: Window, state: Arc<Mutex<AutoHideState>>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            let enabled = {
                let state = state.lock().unwrap();
                state.enabled
            };
            
            if !enabled {
                continue;
            }
            
            let is_hidden = {
                let state = state.lock().unwrap();
                state.is_hidden
            };
            
            if !is_hidden {
                // 检测是否贴边
                if let Some(edge) = check_edge(&window) {
                    // 延迟 1 秒后收起
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    
                    // 再次检查是否仍然贴边
                    if let Some(current_edge) = check_edge(&window) {
                        if matches!(current_edge, edge) {
                            hide_window(&window, edge, state.clone()).await;
                        }
                    }
                }
            }
        }
    });
}
```

### 6. 设置窗口（`src/components/SettingsDialog.vue`）

```vue
<template>
  <div v-if="visible" class="settings-overlay" @click.self="close">
    <div class="settings-dialog">
      <div class="settings-header">
        <span class="settings-title">⚙️ 设置</span>
        <button class="close-btn" @click="close">✕</button>
      </div>
      
      <div class="settings-body">
        <div class="setting-item">
          <label class="setting-label">
            <input 
              type="checkbox" 
              v-model="autoHideEnabled"
              @change="handleAutoHideChange"
            />
            <span>贴边自动收起</span>
          </label>
          <p class="setting-desc">窗口贴近屏幕边缘时自动收起，鼠标靠近时展开</p>
        </div>
      </div>
      
      <div class="settings-footer">
        <button class="btn-primary" @click="close">确定</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

const visible = ref(false);
const autoHideEnabled = ref(true);

const emit = defineEmits(['close']);

const open = async () => {
  visible.value = true;
  // 加载当前设置
  try {
    const enabled = await invoke('get_auto_hide_enabled');
    autoHideEnabled.value = enabled;
  } catch (e) {
    console.error('Failed to load auto-hide setting:', e);
  }
};

const close = () => {
  visible.value = false;
  emit('close');
};

const handleAutoHideChange = async () => {
  try {
    await invoke('set_auto_hide_enabled', { enabled: autoHideEnabled.value });
  } catch (e) {
    console.error('Failed to save auto-hide setting:', e);
  }
};

defineExpose({ open });
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-dialog {
  background: #1a1a2e;
  border-radius: 12px;
  width: 360px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.settings-title {
  font-size: 16px;
  font-weight: 600;
  color: #e0e0ff;
}

.close-btn {
  background: none;
  border: none;
  color: #8a8aaa;
  font-size: 20px;
  cursor: pointer;
  padding: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.close-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: #e0e0ff;
}

.settings-body {
  padding: 20px;
}

.setting-item {
  margin-bottom: 16px;
}

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #e0e0ff;
  font-size: 14px;
  cursor: pointer;
  user-select: none;
}

.setting-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: #00d4ff;
}

.setting-desc {
  margin-top: 6px;
  margin-left: 26px;
  font-size: 12px;
  color: #8a8aaa;
  line-height: 1.5;
}

.settings-footer {
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  justify-content: flex-end;
}

.btn-primary {
  background: linear-gradient(135deg, #00d4ff, #7b2cbf);
  border: none;
  color: white;
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 212, 255, 0.3);
}

.btn-primary:active {
  transform: translateY(0);
}
</style>
```

### 7. 托盘菜单（`src-tauri/src/tray.rs`）

```rust
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTrayEvent, Manager};

pub fn create_tray_menu() -> SystemTrayMenu {
    let show = CustomMenuItem::new("show".to_string(), "显示/隐藏");
    let refresh = CustomMenuItem::new("refresh".to_string(), "立即刷新");
    let settings = CustomMenuItem::new("settings".to_string(), "设置");
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    
    SystemTrayMenu::new()
        .add_item(show)
        .add_item(refresh)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit)
}

pub fn handle_tray_event(app: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            let window = app.get_window("main").unwrap();
            if window.is_visible().unwrap() {
                window.hide().unwrap();
            } else {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                "refresh" => {
                    let _ = app.emit_all("manual-refresh", ());
                }
                "settings" => {
                    let _ = app.emit_all("open-settings", ());
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

### 8. 粒子背景组件（`src/components/ParticleBackground.vue`）

```vue
<template>
  <canvas ref="canvasRef" class="particle-canvas"></canvas>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

const canvasRef = ref(null);
let animationId;
let particles = [];

class Particle {
  constructor(canvas) {
    this.canvas = canvas;
    this.x = Math.random() * canvas.width;
    this.y = Math.random() * canvas.height;
    this.vx = (Math.random() - 0.5) * 0.3;
    this.vy = (Math.random() - 0.5) * 0.3;
    this.radius = Math.random() * 2 + 1;
    this.opacity = Math.random() * 0.3 + 0.3;
    this.color = ['#00d4ff', '#7b2cbf', '#4a90e2'][Math.floor(Math.random() * 3)];
  }
  
  update() {
    this.x += this.vx;
    this.y += this.vy;
    
    if (this.x < 0 || this.x > this.canvas.width) this.vx *= -1;
    if (this.y < 0 || this.y > this.canvas.height) this.vy *= -1;
  }
  
  draw(ctx) {
    ctx.beginPath();
    ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
    ctx.fillStyle = this.color + Math.floor(this.opacity * 255).toString(16).padStart(2, '0');
    ctx.fill();
  }
}

const initParticles = (canvas) => {
  particles = [];
  const particleCount = 25;
  for (let i = 0; i < particleCount; i++) {
    particles.push(new Particle(canvas));
  }
};

const animate = (canvas, ctx) => {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  
  particles.forEach(p => {
    p.update();
    p.draw(ctx);
  });
  
  animationId = requestAnimationFrame(() => animate(canvas, ctx));
};

onMounted(() => {
  const canvas = canvasRef.value;
  const ctx = canvas.getContext('2d');
  
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  
  initParticles(canvas);
  animate(canvas, ctx);
  
  window.addEventListener('resize', () => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    initParticles(canvas);
  });
});

onUnmounted(() => {
  if (animationId) {
    cancelAnimationFrame(animationId);
  }
});
</script>

<style scoped>
.particle-canvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
}
</style>
```

### 9. 全局样式（`src/style.css`）

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #0a0a12;
  color: #e0e0ff;
  overflow: hidden;
  user-select: none;
}

#app {
  width: 100vw;
  height: 100vh;
}
```

## 实现步骤

1. **进入项目目录并初始化 Git**
   - 项目已初始化，直接打开 `D:\repos\kimi-plan-monitor`
   - 如果是从 Tauri 模板创建，确认 Vue 依赖已安装
   - **将本 Plan 文件复制到项目根目录**（`D:\repos\kimi-plan-monitor\PLAN.md`），方便后续查阅
   - **初始化 Git 仓库**：`git init`
   - **创建 `.gitignore` 文件**（见下方内容）
   - **首次提交**：`git add . && git commit -m "chore: 初始化项目"`

2. **配置窗口和托盘**
   - 修改 `tauri.conf.json` 设置窗口属性
   - 实现托盘图标和右键菜单（含设置选项）
   - **提交**：`git add . && git commit -m "feat: 配置窗口属性和托盘菜单"`

3. **实现 API 调用**
   - 编写 `api.rs` 定义数据结构和请求函数
   - 在 `main.rs` 中设置定时器轮询
   - **提交**：`git add . && git commit -m "feat: 实现 Kimi API 调用和定时轮询"`

4. **实现贴边自动收起**
   - 编写 `main.rs` 中的贴边检测逻辑
   - 实现收起/展开动画
   - 添加设置开关
   - **提交**：`git add . && git commit -m "feat: 实现贴边自动收起功能"`

5. **实现 Vue 组件**
   - 编写 `App.vue` 主组件
   - 编写 `UsageLine.vue` 用量行组件（含预警逻辑）
   - 编写 `ParticleBackground.vue` 粒子背景组件
   - 编写 `SettingsDialog.vue` 设置对话框
   - 编写 `style.css` 全局样式
   - **提交**：`git add . && git commit -m "feat: 实现 Vue 组件和 UI 界面"`

6. **实现前后端通信**
   - Rust 端：定义 `refresh_usage`、`get_auto_hide_enabled`、`set_auto_hide_enabled` commands
   - 前端：监听 `usage-updated`、`open-settings` 事件
   - **提交**：`git add . && git commit -m "feat: 实现前后端通信"`

7. **实现 Token 和设置存储**
   - 使用 `tauri-plugin-store` 保存 Token 和设置
   - 首次启动时弹出输入框
   - **提交**：`git add . && git commit -m "feat: 实现 Token 和设置存储"`

8. **测试和打包**
   - 开发模式：`npm run tauri dev`
   - 打包：`npm run tauri build`
   - **提交**：`git add . && git commit -m "chore: 完成开发和测试"`

## 验证方式

1. **功能验证：**
   - 启动应用，窗口置顶显示
   - 拖动窗口，位置可调整
   - 调整窗口大小，UI 自适应
   - 点击刷新按钮，数据更新
   - 等待 60 秒，数据自动更新
   - 右键托盘图标，显示菜单
   - 点击退出，应用关闭

2. **数据验证：**
   - 周用量百分比计算正确
   - 5小时用量百分比计算正确
   - 重置时间格式正确（MM-DD HH:mm:ss）
   - 最后更新时间显示正确

3. **UI 验证：**
   - 粒子背景动画流畅
   - 刷新按钮旋转动画正常
   - 配色方案符合设计
   - 字体对齐（等宽字体）

4. **预警验证：**
   - 剩余 ≥ 30% 时，剩余百分比显示绿色（`#51cf66`）
   - 剩余 < 30% 时，剩余百分比显示红色（`#ff6b6b`）并带脉冲动画
   - 预警状态切换流畅，无延迟

5. **贴边收起验证：**
   - 拖动窗口到屏幕边缘，窗口自动收起为一条小边
   - 鼠标靠近小边，窗口自动展开
   - 鼠标离开窗口，延迟 1 秒后自动收起
   - 在设置中关闭贴边收起，窗口不再自动收起
   - 设置保存后重启应用，设置仍然生效

## 依赖清单

**Rust 依赖（`Cargo.toml`）：**
```toml
[dependencies]
tauri = { version = "2.0", features = ["system-tray"] }
tauri-plugin-store = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
```

**前端依赖（`package.json`）：**
```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

## Git 配置

### `.gitignore` 文件

```gitignore
# 依赖
node_modules/
src-tauri/target/

# 构建产物
dist/
src-tauri/gen/

# 配置文件（包含敏感信息）
config.json
*.local

# 日志
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# 编辑器
.vscode/
.idea/
*.swp
*.swo
*~

# 操作系统
.DS_Store
Thumbs.db

# 环境变量
.env
.env.local
.env.*.local

# Tauri
src-tauri/Cargo.lock
# 注意：Cargo.lock 可以选择提交或不提交，团队项目建议提交
```

### Commit 规范

遵循 Conventional Commits 规范，使用中文描述：

- `chore: 初始化项目` - 项目初始化、配置文件
- `feat: 添加 xxx 功能` - 新功能
- `fix: 修复 xxx 问题` - Bug 修复
- `refactor: 重构 xxx` - 代码重构
- `docs: 更新文档` - 文档更新
- `style: 调整样式` - 样式调整
- `test: 添加测试` - 测试相关

### 分阶段提交

每个实现步骤完成后自动 commit，提交信息遵循上述规范。这样：
- 每次 commit 都是一个完整的功能点
- 方便回溯和定位问题
- 便于 code review

## 注意事项

1. **Token 安全：**
   - Token 存储在本地配置文件，不要提交到 Git
   - 建议在 `.gitignore` 中添加 `config.json`

2. **API 限流：**
   - 60 秒轮询一次应该足够，不要过于频繁
   - 如果遇到 429 错误，增加轮询间隔

3. **窗口位置：**
   - 可以使用 `tauri-plugin-positioner` 保存和恢复窗口位置
   - 默认显示在屏幕右下角

4. **打包优化：**
   - 使用 `tauri build --release` 减小体积
   - 可以考虑使用 UPX 压缩可执行文件

## 后续扩展（可选）

- 支持多个 API Token 切换
- 添加用量历史图表
- 支持自定义轮询间隔
- ~~添加用量预警（比如剩余 < 30% 时变红）~~ ✅ 已实现
- 支持自定义预警阈值（比如 20%、30%、50% 可选）
- 预警时播放提示音或系统通知
- 支持 macOS/Linux
