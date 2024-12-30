<script setup lang="ts">
import { ref, computed } from 'vue';
import ProgressBar from './ProgressBar.vue';

// 添加 CoreInfo 接口
interface CoreInfo {
  usage: number;
  temperature: number;
}

// 添加 CpuInfo 接口
interface CpuInfo {
  vendor: string;
  model: string;
  cores: number;
  threads: number;
}

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  usage: {
    type: Number,
    required: true
  },
  temperature: {
    type: Number,
    required: true
  },
  fanSpeed: {
    type: Number,
    default: 0
  },
  cpuInfo: {
    type: Object as () => CpuInfo,
    default: () => ({
      vendor: '',
      model: '',
      cores: 0,
      threads: 0
    })
  },
  cpuCores: {
    type: Array as () => CoreInfo[],
    default: () => []
  }
});

const showCoreInfo = ref(false);

// 添加切换函数
const toggleCoreInfo = () => {
  showCoreInfo.value = !showCoreInfo.value;
};

// 计算温度百分比
const tempPercentage = computed(() => {
  return Math.min(100, props.temperature);
});

// 根据温度返回进度条颜色
const getTempColor = computed(() => {
  if (props.temperature < 60) return 'success';
  if (props.temperature < 80) return 'warning';
  return 'danger';
});

// 添加计算属性来计算核心信息的高度
const coreInfoHeight = computed(() => {
  const coresCount = props.cpuCores.length;
  const rowCount = Math.ceil(coresCount / 4);  // 每行4个核心
  const rowHeight = 72;  // 每行的高度
  const rowGap = 16;    // 行间距
  const verticalPadding = 16;  // 上下内边距
  
  return rowCount * rowHeight + (rowCount - 1) * rowGap + verticalPadding;
});
</script>

<template>
  <div class="stat-card-wrapper">
    <div class="stat-card" :class="{ 'expanded': showCoreInfo }">
      <!-- 卡片头部 -->
      <div class="card-header">
        <div class="title-section">
          <div class="title-row">
            <div class="model-info">
              <div class="model-details">
                <div class="brand-badge cpu">
                  {{ cpuInfo.vendor }}
                </div>
                <span class="model-series">{{ cpuInfo.model }}</span>
                <button 
                  class="core-info-trigger"
                  @click="toggleCoreInfo"
                  :class="{ 'active': showCoreInfo }"
                >
                  <span class="core-count">{{ cpuInfo.cores }}核{{ cpuInfo.threads }}线程</span>
                  <span class="trigger-icon">
                    <svg 
                      viewBox="0 0 24 24" 
                      width="16" 
                      height="16" 
                      fill="none" 
                      stroke="currentColor" 
                      stroke-width="2"
                      :class="{ 'rotated': showCoreInfo }"
                    >
                      <path d="M19 9l-7 7-7-7" />
                    </svg>
                  </span>
                </button>
              </div>
            </div>
            <div class="fan-speed">
              <div class="fan-speed-wrapper">
                <svg class="fan-icon" :class="{ 'rotating': fanSpeed > 0 }" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                  <path d="M12,11A1,1 0 0,0 11,12A1,1 0 0,0 12,13A1,1 0 0,0 13,12A1,1 0 0,0 12,11M12.5,2C17,2 17.11,5.57 14.75,6.75C13.76,7.24 13.32,8.29 13.13,9.22C13.61,9.42 14.03,9.73 14.35,10.13C18.05,8.13 22.03,8.92 22.03,12.5C22.03,17 18.46,17.1 17.28,14.73C16.78,13.74 15.72,13.3 14.79,13.11C14.59,13.59 14.28,14 13.88,14.34C15.87,18.03 15.08,22 11.5,22C7,22 6.91,18.42 9.27,17.24C10.25,16.75 10.69,15.71 10.89,14.79C10.4,14.59 9.97,14.27 9.65,13.87C5.96,15.85 2,15.07 2,11.5C2,7 5.56,6.89 6.74,9.26C7.24,10.25 8.29,10.68 9.22,10.87C9.41,10.39 9.73,9.97 10.14,9.66C8.15,5.96 8.94,2 12.5,2Z"/>
                </svg>
                <span>{{ Math.round(fanSpeed) }} RPM</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 主要指标 -->
      <div class="metrics">
        <div class="metric-row">
          <span class="metric-label">使用率</span>
          <ProgressBar :percentage="usage" type="cpu" />
        </div>
        <div class="metric-row">
          <span class="metric-label">温度</span>
          <ProgressBar 
            :percentage="tempPercentage" 
            :type="getTempColor" 
            :temperature="temperature"
            show-temp
          />
        </div>
      </div>

      <!-- 核心信息区域 -->
      <div v-if="cpuCores.length > 0" 
           class="core-info-section" 
           :class="{ 'expanded': showCoreInfo }"
           :style="{ '--content-height': `${coreInfoHeight}px` }">
        <div class="cores-grid">
          <div v-for="(core, index) in cpuCores" :key="index" class="core-item">
            <div class="usage-section">
              <div class="ring-container">
                <svg class="progress-ring" width="42" height="42" viewBox="0 0 42 42">
                  <defs>
                    <linearGradient id="progressGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                      <stop offset="0%" style="stop-color:#0ea5e9;stop-opacity:1" />
                      <stop offset="100%" style="stop-color:#38bdf8;stop-opacity:1" />
                    </linearGradient>
                  </defs>
                  <!-- 外层装饰圆环 -->
                  <circle
                    class="progress-ring-outer"
                    cx="21"
                    cy="21"
                    r="20"
                    fill="none"
                    stroke-width="0.75"
                  />
                  <!-- 背景圆环 -->
                  <circle
                    class="progress-ring-bg"
                    cx="21"
                    cy="21"
                    r="18"
                    fill="none"
                    stroke-width="6"
                    stroke-linecap="round"
                  />
                  <!-- 进度圆环 -->
                  <circle
                    class="progress-ring-value"
                    cx="21"
                    cy="21"
                    r="18"
                    fill="none"
                    stroke-width="6"
                    stroke="url(#progressGradient)"
                    stroke-linecap="round"
                    :stroke-dasharray="113.1"
                    :stroke-dashoffset="113.1 - (113.1 * core.usage) / 100"
                  />
                </svg>
                <div class="usage-value">{{ Math.round(core.usage) }}%</div>
              </div>
              <div class="temp-value">{{ Math.round(core.temperature) }}°C</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stat-card-wrapper {
  width: 100%;
}

.stat-card {
  background: rgba(30, 41, 59, 0.9);
  border-radius: 16px;
  padding: 20px;
  position: relative;
  transition: all 0.3s ease;
}

.card-header {
  margin-bottom: 16px;
}

.title-section {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  width: 100%;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.model-info {
  display: flex;
  flex-direction: column;
}

.model-details {
  display: flex;
  align-items: center;
  gap: 8px;
}

.brand-badge {
  color: white;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.05em;
}

.brand-badge.cpu {
  background: linear-gradient(135deg, #0095ff, #00c3ff);
}

.model-series {
  color: #9ca3af;
  font-size: 0.7rem;
  font-weight: 500;
}

.core-info-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  transition: transform 0.2s ease;
}

.core-count {
  font-size: 0.7rem;
  color: #9ca3af;
  font-weight: 500;
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.core-info-trigger:hover .core-count {
  background: rgba(255, 255, 255, 0.15);
}

.trigger-icon {
  display: flex;
  align-items: center;
  color: #9ca3af;
  transition: transform 0.2s ease;
}

.trigger-icon svg {
  transform: rotate(0deg);
  transition: transform 0.2s ease;
  will-change: transform;
}

.trigger-icon svg.rotated {
  transform: rotate(180deg);
}

.core-info-trigger.active .core-count {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
}

.core-info-trigger.active .trigger-icon {
  color: #60a5fa;
}

.fan-speed {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #9ca3af;
  font-size: 0.7rem;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.fan-icon {
  color: #9ca3af;
}

.fan-icon.rotating {
  animation: rotate 2s linear infinite;
}

.metrics {
  display: flex;
  flex-direction: column;
  gap: 20px;
  margin-top: 24px;
}

.metric-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.metric-label {
  color: #9ca3af;
  font-size: 0.875rem;
  font-weight: 500;
  min-width: 48px;
}

.core-info-section {
  max-height: 0;
  opacity: 0;
  overflow: hidden;
  transform-origin: top;
  transition: all 0.3s linear;
  margin-top: 0;
  padding: 0 8px;
}

.core-info-section.expanded {
  max-height: var(--content-height);
  opacity: 1;
  margin-top: 20px;
  transform: translateY(0);
}

.cores-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px 8px;
  width: 100%;
  padding: 0 4px;
}

.core-item {
  height: 64px;
  opacity: 0;
  transform: translateY(8px);
  transition: all 0.3s linear;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.expanded .core-item {
  opacity: 1;
  transform: translateY(0);
}

.usage-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
}

.ring-container {
  position: relative;
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.progress-ring {
  position: absolute;
  transform: rotate(-90deg);
  width: 42px;
  height: 42px;
}

.progress-ring-outer {
  stroke: rgba(255, 255, 255, 0.05);
  stroke-width: 0.75;
}

.progress-ring-bg {
  stroke: rgba(148, 163, 184, 0.2);
  stroke-linecap: round;
  stroke-width: 6;
}

.progress-ring-value {
  stroke-width: 6;
  transition: stroke-dashoffset 0.3s ease;
  filter: drop-shadow(0 0 2px rgba(14, 165, 233, 0.3));
}

.usage-value {
  position: absolute;
  font-size: 0.75rem;
  font-weight: 600;
  color: #f8fafc;
  font-variant-numeric: tabular-nums;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  text-align: center;
  z-index: 1;
  width: 100%;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
}

.temp-value {
  font-size: 0.7rem;
  font-weight: 500;
  color: #94a3b8;
  font-variant-numeric: tabular-nums;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  text-align: center;
  margin-top: 4px;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.fan-speed-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  transition: background-color 0.2s ease;
}

.fan-speed-wrapper:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style> 