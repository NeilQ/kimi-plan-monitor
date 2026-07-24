<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  icon: string;
  label: string;
  used: string;
  remaining: string;
  resetTime: string;
}>();

const isWarning = computed(() => {
  const percent = parseInt(props.remaining);
  return !isNaN(percent) && percent < 30;
});
</script>

<template>
  <div class="usage-line flex items-center gap-2 font-mono text-xs px-2 py-1.5 rounded-lg backdrop-blur-md">
    <span class="text-sm">{{ icon }}</span>
    <span class="text-text-primary font-medium min-w-[50px]">{{ label }}</span>
    <span class="text-accent-blue font-semibold min-w-[40px] text-right">{{ used }}</span>
    <span class="text-text-muted">│</span>
    <span
      class="font-semibold min-w-[40px] text-right transition-colors"
      :class="isWarning ? 'text-danger animate-warning-pulse' : 'text-success'"
    >
      {{ remaining }}
    </span>
    <span class="text-text-secondary ml-auto text-[11px]">{{ resetTime }}</span>
  </div>
</template>

<style scoped>
@keyframes warning-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.animate-warning-pulse {
  animation: warning-pulse 1s ease-in-out infinite;
}

.usage-line {
  background: var(--color-bg-line);
}
</style>
