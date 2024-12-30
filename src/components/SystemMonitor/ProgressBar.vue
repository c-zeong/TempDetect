<script setup lang="ts">
defineProps<{
  percentage: number;
  type?: string;
  showTemp?: boolean;
  temperature?: number;
}>();
</script>

<template>
  <div class="progress-wrapper">
    <div class="progress-container">
      <div class="progress-bar" 
           :class="type"
           :style="{ width: `${Math.max(percentage, 0)}%` }">
      </div>
    </div>
    <div class="value" v-if="!showTemp">{{ Math.round(percentage) }}%</div>
    <div class="temp-value" v-if="showTemp">{{ Math.round(temperature || 0) }}°C</div>
  </div>
</template>

<style scoped>
.progress-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.progress-container {
  position: relative;
  background: rgba(148, 163, 184, 0.1);
  border-radius: 8px;
  height: 14px;
  overflow: hidden;
  flex: 1;
  width: 100%;
}

.progress-bar {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  border-radius: 8px;
  transition: width 0.3s ease;
  will-change: width;
}

/* 使用率数值 */
.value {
  color: white;
  font-size: 0.875rem;
  font-weight: 600;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  font-variant-numeric: tabular-nums;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  min-width: 42px;
  text-align: right;
  flex-shrink: 0;
}

/* 温度数值 */
.temp-value {
  color: white;
  font-size: 0.875rem;
  font-weight: 600;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  font-variant-numeric: tabular-nums;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  min-width: 42px;
  text-align: right;
  flex-shrink: 0;
}

/* 温度颜色定义 */
.progress-bar.success {
  background: linear-gradient(90deg, #22c55e, #16a34a);
}

.progress-bar.warning {
  background: linear-gradient(90deg, #f59e0b, #d97706);
}

.progress-bar.danger {
  background: linear-gradient(90deg, #ef4444, #dc2626);
}

/* GPU 颜色 */
.progress-bar.gpu {
  background: linear-gradient(90deg, #9333ea, #7e22ce);
}

/* CPU 颜色 */
.progress-bar.cpu {
  background: linear-gradient(90deg, #3b82f6, #2563eb);
}
</style> 