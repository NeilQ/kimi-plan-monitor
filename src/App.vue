<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import UsageLine from './components/UsageLine.vue';

const weeklyUsed = ref('--%');
const weeklyRemaining = ref('--%');
const weeklyResetTime = ref('--');
const hourlyUsed = ref('--%');
const hourlyRemaining = ref('--%');
const hourlyResetTime = ref('--');
const lastUpdateTime = ref('--:--:--');
const isRefreshing = ref(false);

// 主题
const theme = ref<'dark' | 'light'>('light');

const applyTheme = (t: 'dark' | 'light') => {
  document.documentElement.setAttribute('data-theme', t);
  localStorage.setItem('theme', t);
};

const loadTheme = async () => {
  try {
    const saved = await invoke<'dark' | 'light'>('get_theme');
    theme.value = saved || 'light';
    applyTheme(theme.value);
  } catch (e) {
    console.error('Failed to load theme:', e);
    const fallback = (localStorage.getItem('theme') as 'dark' | 'light') || 'light';
    theme.value = fallback;
    applyTheme(fallback);
  }
};

const toggleTheme = async () => {
  const newTheme = theme.value === 'dark' ? 'light' : 'dark';
  theme.value = newTheme;
  applyTheme(newTheme);
  try {
    await invoke('set_theme', { theme: newTheme });
  } catch (e) {
    console.error('Failed to set theme:', e);
  }
};

const handleRefresh = async () => {
  isRefreshing.value = true;
  try {
    await invoke('refresh_usage');
  } catch (e) {
    // 后端已清空数据并发出 usage-cleared，这里不额外处理显示
    console.error('Refresh failed:', e);
  } finally {
    setTimeout(() => {
      isRefreshing.value = false;
    }, 600);
  }
};

const clearUsage = () => {
  weeklyUsed.value = '--%';
  weeklyRemaining.value = '--%';
  weeklyResetTime.value = '--';
  hourlyUsed.value = '--%';
  hourlyRemaining.value = '--%';
  hourlyResetTime.value = '--';
};

const updateUsage = (data: any) => {
  const weeklyLimit = parseInt(data.usage.limit);
  const weeklyRemainingCount = parseInt(data.usage.remaining);
  const weeklyUsedCount = weeklyLimit - weeklyRemainingCount;
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
let unlistenRefresh: (() => void) | undefined;
let unlistenTheme: (() => void) | undefined;
let unlistenCleared: (() => void) | undefined;

onMounted(async () => {
  await loadTheme();

  unlistenUsage = await listen('usage-updated', (event: any) => {
    updateUsage(event.payload);
  });

  unlistenCleared = await listen('usage-cleared', () => {
    clearUsage();
  });

  unlistenRefresh = await listen('manual-refresh', () => {
    handleRefresh();
  });

  unlistenTheme = await listen('theme-changed', (event: any) => {
    const t = event.payload as 'dark' | 'light';
    if (t === 'dark' || t === 'light') {
      theme.value = t;
      applyTheme(t);
    }
  });

  handleRefresh();
});

onUnmounted(() => {
  if (unlistenUsage) unlistenUsage();
  if (unlistenCleared) unlistenCleared();
  if (unlistenRefresh) unlistenRefresh();
  if (unlistenTheme) unlistenTheme();
});
</script>

<template>
  <div class="relative overflow-hidden" :style="{ background: 'var(--color-bg-base)' }">
    <div class="relative z-10 h-full flex flex-col gap-1 min-w-[250px] min-h-[90px]"
                        style="-webkit-app-region: drag">
      <div class="header flex justify-between items-center">
        <span class="text-sm font-semibold text-text-primary flex items-center gap-1.5">
          <span class="inline-flex items-center justify-center w-4 h-4 mt-1 ml-1">
            <svg class="w-full h-full" viewBox="0 0 1024 1024" fill="none" stroke="currentColor" stroke-width="60" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="460" cy="230" r="78" />
              <path d="M460 308 L560 470" />
              <path d="M508 390 L330 330 L240 400" />
              <path d="M520 400 L660 340 L740 260" />
              <path d="M560 470 L500 720 L420 820" />
              <path d="M560 470 L720 560 L860 520" />
            </svg>
          </span>
          Kimi Plan
        </span>
        <span class="text-[11px] text-text-secondary">{{ lastUpdateTime }}</span>
        <div class="flex items-center" style="-webkit-app-region: no-drag">
          <button class="icon-btn bg-transparent border-none text-base cursor-pointer rounded-md transition-colors"
                  :title="theme === 'dark' ? '切换到浅色主题' : '切换到深色主题'"
                  @click="toggleTheme">
            {{ theme === 'dark' ? '☀️' : '🌙' }}
          </button>
          <button class="icon-btn bg-transparent border-none text-base cursor-pointer px-2 rounded-md transition-colors"
                  :class="{ 'animate-spin-once': isRefreshing }"
                  title="刷新"
                  @click="handleRefresh">
            🔄
          </button>
        </div>
      </div>

      <UsageLine icon="📅"
                 label="周用量"
                 :used="weeklyUsed"
                 :remaining="weeklyRemaining"
                 :reset-time="weeklyResetTime" />

      <UsageLine icon="⏱️"
                 label="5小时"
                 :used="hourlyUsed"
                 :remaining="hourlyRemaining"
                 :reset-time="hourlyResetTime" />
  </div>
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

.header {
  border-bottom: 1px solid var(--color-border);
}

.icon-btn:hover {
  background-color: var(--color-hover);
}
</style>
