<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import ParticleBackground from './components/ParticleBackground.vue';
import UsageLine from './components/UsageLine.vue';
import SettingsDialog from './components/SettingsDialog.vue';

const weeklyUsed = ref('--%');
const weeklyRemaining = ref('--%');
const weeklyResetTime = ref('--');
const hourlyUsed = ref('--%');
const hourlyRemaining = ref('--%');
const hourlyResetTime = ref('--');
const lastUpdateTime = ref('--:--:--');
const isRefreshing = ref(false);
const settingsDialog = ref<InstanceType<typeof SettingsDialog>>();

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

const updateUsage = (data: any) => {
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

const formatResetTime = (isoString: string) => {
  const date = new Date(isoString);
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${month}-${day} ${hours}:${minutes}:${seconds}`;
};

let unlistenUsage: (() => void) | undefined;
let unlistenSettings: (() => void) | undefined;
let unlistenRefresh: (() => void) | undefined;

onMounted(async () => {
  // 监听后端推送的用量更新事件
  unlistenUsage = await listen('usage-updated', (event: any) => {
    updateUsage(event.payload);
  });

  // 监听打开设置事件
  unlistenSettings = await listen('open-settings', () => {
    settingsDialog.value?.open();
  });

  // 监听手动刷新事件
  unlistenRefresh = await listen('manual-refresh', () => {
    handleRefresh();
  });

  // 初始加载
  handleRefresh();
});

onUnmounted(() => {
  if (unlistenUsage) unlistenUsage();
  if (unlistenSettings) unlistenSettings();
  if (unlistenRefresh) unlistenRefresh();
});
</script>

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

    <SettingsDialog ref="settingsDialog" />
  </div>
</template>

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
