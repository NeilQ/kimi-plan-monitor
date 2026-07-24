<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    theme?: 'dark' | 'light';
  }>(),
  { theme: 'dark' }
);

const canvasRef = ref<HTMLCanvasElement>();
let animationId: number;
let particles: Particle[] = [];

// 深色主题粒子：青/蓝/紫 高亮
const DARK_COLORS = ['#00d4ff', '#7b2cbf', '#4a90e2'];
// 浅色主题粒子：低饱和度深色调，避免在白底上看不清
const LIGHT_COLORS = ['#0891b2', '#6d28d9', '#2563eb'];

class Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  opacity: number;
  color: string;
  canvas: HTMLCanvasElement;

  constructor(canvas: HTMLCanvasElement, colors: string[]) {
    this.canvas = canvas;
    this.x = Math.random() * canvas.width;
    this.y = Math.random() * canvas.height;
    this.vx = (Math.random() - 0.5) * 0.3;
    this.vy = (Math.random() - 0.5) * 0.3;
    this.radius = Math.random() * 2 + 1;
    this.opacity = Math.random() * 0.3 + 0.3;
    this.color = colors[Math.floor(Math.random() * colors.length)];
  }

  update() {
    this.x += this.vx;
    this.y += this.vy;

    if (this.x < 0 || this.x > this.canvas.width) this.vx *= -1;
    if (this.y < 0 || this.y > this.canvas.height) this.vy *= -1;
  }

  draw(ctx: CanvasRenderingContext2D) {
    ctx.beginPath();
    ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
    ctx.fillStyle = this.color + Math.floor(this.opacity * 255).toString(16).padStart(2, '0');
    ctx.fill();
  }
}

const currentColors = () => (props.theme === 'light' ? LIGHT_COLORS : DARK_COLORS);

const initParticles = (canvas: HTMLCanvasElement) => {
  particles = [];
  const particleCount = 25;
  for (let i = 0; i < particleCount; i++) {
    particles.push(new Particle(canvas, currentColors()));
  }
};

const animate = (canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) => {
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  particles.forEach(p => {
    p.update();
    p.draw(ctx);
  });

  animationId = requestAnimationFrame(() => animate(canvas, ctx));
};

watch(
  () => props.theme,
  () => {
    // 主题切换时重建粒子颜色
    const canvas = canvasRef.value;
    if (canvas) {
      particles.forEach(p => {
        p.color = currentColors()[Math.floor(Math.random() * currentColors().length)];
      });
    }
  }
);

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

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

<template>
  <canvas ref="canvasRef" class="absolute top-0 left-0 w-full h-full z-0"></canvas>
</template>
