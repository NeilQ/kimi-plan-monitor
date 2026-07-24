<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  icon: string;
  label: string;
  used: string;
  remaining: string;
  resetTime: string;
}>();

// 判断是否预警（剩余 < 30%）
const isWarning = computed(() => {
  const percent = parseInt(props.remaining);
  return !isNaN(percent) && percent < 30;
});
</script>

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
