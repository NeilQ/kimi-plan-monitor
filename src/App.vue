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
  const weeklyLimit = parseInt(data.usage.limit);
  const weeklyUsedCount = parseInt(data.usage.used);
  const weeklyUsedPercent = Math.round((weeklyUsedCount / weeklyLimit) * 100);
  const weeklyRemainingPercent = 100 - weeklyUsedPercent;

  weeklyUsed.value = `${weeklyUsedPercent}%`;
  weeklyRemaining.value = `${weeklyRemainingPercent}%`;
  weeklyResetTime.value = formatResetTime(data.usage.resetTime);

  const hourlyLimit = data.limits[0];
  const hourlyLimitCount = parseInt(hourlyLimit.detail.limit);
  const hourlyRemainingCount = parseInt(hourlyLimit.detail.remaining);
  const hourlyUsedCount = hourlyLimitCount - hourlyRemainingCount;
  const hourlyUsedPercent = Math.round((hourlyUsedCount / hourlyLimitCount) * 100);
  const hourlyRemainingPercent = 100 - hourlyUsedPercent;

  hourlyUsed.value = `${hourlyUsedPercent}%`;
  hourlyRemaining.value = `${hourlyRemainingPercent}%`;
  hourlyResetTime.value = formatResetTime(hourlyLimit.detail.resetTime);

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
  unlistenUsage = await listen('usage-updated', (event: any) => {
    updateUsage(event.payload);
  });

  unlistenSettings = await listen('open-settings', () => {
    settingsDialog.value?.open();
  });

  unlistenRefresh = await listen('manual-refresh', () => {
    handleRefresh();
  });

  handleRefresh();
});

onUnmounted(() => {
  if (unlistenUsage) unlistenUsage();
  if (unlistenSettings) unlistenSettings();
  if (unlistenRefresh) unlistenRefresh();
});
</script>

<template>
  <div class="w-screen h-screen relative overflow-hidden">
    <ParticleBackground />

    <div
      class="relative z-10 h-full flex flex-col gap-2 p-3"
      style="-webkit-app-region: drag"
    >
      <div class="flex justify-between items-center pb-2 border-b border-white/10">
        <span class="text-sm font-semibold text-text-primary">⚡ Kimi Plan</span>
        <button
          class="bg-transparent border-none text-base cursor-pointer px-2 py-1 rounded-md transition-colors hover:bg-white/10"
          :class="{ 'animate-spin-once': isRefreshing }"
          style="-webkit-app-region: no-drag"
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

      <div class="mt-auto pt-2 border-t border-white/10">
        <span class="text-[11px] text-text-secondary">✨ {{ lastUpdateTime }}</span>
      </div>
    </div>

    <SettingsDialog ref="settingsDialog" />
  </div>
</template>

<style scoped>
@keyframes spin-once {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin-once {
  animation: spin-once 0.6s linear;
}
</style>
