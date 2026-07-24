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
  // 加载当前设置
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
  <div v-if="visible" class="settings-overlay" @click.self="close">
    <div class="settings-dialog">
      <div class="settings-header">
        <span class="settings-title">⚙️ 设置</span>
        <button class="close-btn" @click="close">✕</button>
      </div>

      <div class="settings-body">
        <div class="setting-item">
          <label class="setting-label-text">API Token</label>
          <input
            type="text"
            class="token-input"
            v-model="token"
            @blur="handleTokenChange"
            placeholder="sk-kimi-..."
          />
          <p class="setting-desc">Kimi Code Plan 的 API Token</p>
        </div>

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
  -webkit-app-region: no-drag;
}

.settings-dialog {
  background: #1a1a2e;
  border-radius: 12px;
  width: 400px;
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
  margin-bottom: 20px;
}

.setting-label-text {
  display: block;
  color: #e0e0ff;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 8px;
}

.token-input {
  width: 100%;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: #e0e0ff;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  outline: none;
  transition: border-color 0.2s;
}

.token-input:focus {
  border-color: #00d4ff;
}

.token-input::placeholder {
  color: #4a4a6a;
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
