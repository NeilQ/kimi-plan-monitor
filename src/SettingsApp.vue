<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const autoHideEnabled = ref(true);
const token = ref('');
const saving = ref(false);
const message = ref('');

const load = async () => {
  try {
    autoHideEnabled.value = await invoke<boolean>('get_auto_hide_enabled');
    token.value = await invoke<string>('get_token');
  } catch (e) {
    console.error('Failed to load settings:', e);
  }
};

const save = async () => {
  saving.value = true;
  message.value = '';
  try {
    await invoke('set_auto_hide_enabled', { enabled: autoHideEnabled.value });
    await invoke('set_token', { token: token.value });
    message.value = '保存成功';
    setTimeout(() => {
      message.value = '';
    }, 2000);
  } catch (e) {
    console.error('Failed to save settings:', e);
    message.value = '保存失败';
  } finally {
    saving.value = false;
  }
};

const close = async () => {
  await getCurrentWindow().close();
};

onMounted(load);
</script>

<template>
  <div class="w-screen h-screen flex flex-col text-sm" :style="{ background: 'var(--color-bg-base)' }">
    <!-- 标题栏 -->
    <div
      class="h-12 flex items-center justify-between px-5 border-b shrink-0"
      style="-webkit-app-region: drag; border-color: var(--color-border)"
    >
      <span class="font-semibold text-text-primary">⚙️ 设置</span>
      <button
        class="w-7 h-7 flex items-center justify-center rounded-md text-text-secondary hover:text-text-primary transition-colors"
        style="-webkit-app-region: no-drag"
        :style="{ '--tw-hover-bg': 'var(--color-hover)' }"
        title="关闭"
        @click="close"
      >
        ✕
      </button>
    </div>

    <!-- 表单 -->
    <div class="flex-1 p-6 flex flex-col gap-5 overflow-auto">
      <!-- API Token -->
      <div class="flex flex-col gap-2">
        <label class="font-medium text-text-primary">API Token</label>
        <input
          v-model="token"
          type="text"
          placeholder="sk-kimi-..."
          class="w-full px-3 py-2 rounded-md text-[13px] font-mono outline-none transition-colors"
          :style="{
            background: 'var(--color-bg-line)',
            border: '1px solid var(--color-border)',
            color: 'var(--color-text-primary)',
          }"
          style="--tw-focus-border: var(--color-accent-cyan)"
        />
        <p class="text-xs leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">
          Kimi Code Plan 的 API Token，仅保存在本地配置文件中。
        </p>
      </div>

      <!-- 贴边自动收起 -->
      <div class="flex flex-col gap-2">
        <label class="flex items-center gap-3 cursor-pointer select-none"
        >
          <input
            v-model="autoHideEnabled"
            type="checkbox"
            class="w-[18px] h-[18px] cursor-pointer"
            :style="{ accentColor: 'var(--color-accent-cyan)' }"
          />
          <span class="font-medium text-text-primary">贴边自动收起</span>
        </label>
        <p class="text-xs leading-relaxed ml-[26px]" :style="{ color: 'var(--color-text-secondary)' }">
          窗口贴近屏幕边缘时自动收起，鼠标靠近时展开。
        </p>
      </div>
    </div>

    <!-- 底部按钮 -->
    <div
      class="h-16 flex items-center justify-between px-5 border-t shrink-0"
      :style="{ borderColor: 'var(--color-border)' }"
    >
      <span class="text-xs" :style="{ color: 'var(--color-success)' }">{{ message }}</span>
      <div class="flex items-center gap-3">
        <button
          class="px-4 py-2 rounded-md text-text-secondary hover:text-text-primary transition-colors"
          :style="{ background: 'var(--color-hover)' }"
          @click="close"
        >
          取消
        </button>
        <button
          class="px-5 py-2 rounded-md text-white font-medium transition-all hover:-translate-y-px hover:shadow-lg active:translate-y-0 disabled:opacity-50"
          :class="{ 'opacity-70 cursor-not-allowed': saving }"
          :style="{
            background: 'linear-gradient(135deg, var(--color-accent-cyan), var(--color-accent-purple))',
          }"
          :disabled="saving"
          @click="save"
        >
          {{ saving ? '保存中...' : '保存' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
button:hover {
  background-color: var(--color-hover);
}

input:focus {
  border-color: var(--color-accent-cyan) !important;
}
</style>
