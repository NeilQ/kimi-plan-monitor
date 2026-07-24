<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const visible = ref(false);
const autoHideEnabled = ref(true);
const token = ref('');

const emit = defineEmits<{
  close: [];
}>();

const open = async () => {
  visible.value = true;
  try {
    const enabled = await invoke<boolean>('get_auto_hide_enabled');
    autoHideEnabled.value = enabled;

    const savedToken = await invoke<string>('get_token');
    token.value = savedToken;
  } catch (e) {
    console.error('Failed to load settings:', e);
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

const handleTokenChange = async () => {
  try {
    await invoke('set_token', { token: token.value });
  } catch (e) {
    console.error('Failed to save token:', e);
  }
};

defineExpose({ open });
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 w-screen h-screen bg-black/50 flex items-center justify-center z-[1000]"
    style="-webkit-app-region: no-drag"
    @click.self="close"
  >
    <div class="bg-bg-card rounded-xl w-[400px] shadow-2xl border border-white/10">
      <div class="flex justify-between items-center px-5 py-4 border-b border-white/10">
        <span class="text-base font-semibold text-text-primary">⚙️ 设置</span>
        <button
          class="bg-transparent border-none text-text-secondary text-xl cursor-pointer w-7 h-7 flex items-center justify-center rounded-md transition-colors hover:bg-white/10 hover:text-text-primary"
          @click="close"
        >
          ✕
        </button>
      </div>

      <div class="p-5">
        <div class="mb-5">
          <label class="block text-text-primary text-sm font-medium mb-2">API Token</label>
          <input
            type="text"
            v-model="token"
            @blur="handleTokenChange"
            placeholder="sk-kimi-..."
            class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-md text-text-primary text-[13px] font-mono outline-none transition-colors focus:border-accent-cyan placeholder:text-text-muted"
          />
          <p class="mt-1.5 text-xs text-text-secondary leading-relaxed">Kimi Code Plan 的 API Token</p>
        </div>

        <div class="mb-5">
          <label class="flex items-center gap-2 text-text-primary text-sm cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="autoHideEnabled"
              @change="handleAutoHideChange"
              class="w-[18px] h-[18px] cursor-pointer accent-accent-cyan"
            />
            <span>贴边自动收起</span>
          </label>
          <p class="mt-1.5 ml-[26px] text-xs text-text-secondary leading-relaxed">窗口贴近屏幕边缘时自动收起，鼠标靠近时展开</p>
        </div>
      </div>

      <div class="px-5 py-4 border-t border-white/10 flex justify-end">
        <button
          class="bg-gradient-to-br from-accent-cyan to-accent-purple border-none text-white px-5 py-2 rounded-lg text-sm font-medium cursor-pointer transition-all hover:-translate-y-px hover:shadow-[0_4px_12px_rgba(0,212,255,0.3)] active:translate-y-0"
          @click="close"
        >
          确定
        </button>
      </div>
    </div>
  </div>
</template>
