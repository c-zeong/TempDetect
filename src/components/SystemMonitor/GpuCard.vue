<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ProgressBar from './ProgressBar.vue';

// 添加 GpuInfo 接口
interface GpuInfo {
  vendor: string;
  model: string;
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
  }
});

// 硬件信息（只在加载时获取一次）
const gpuInfo = ref<GpuInfo>({
  vendor: '',
  model: ''
});

// 获取硬件信息（只在加载时执行一次）
const loadHardwareInfo = async () => {
  try {
    gpuInfo.value = await invoke('get_gpu_info');
  } catch (error) {
    console.error('Failed to get GPU info:', error);
  }
};

// 组件挂载时加载硬件信息
loadHardwareInfo();

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
</script>

<template>
  <div class="stat-card-wrapper">
    <div class="stat-card">
      <!-- 卡片头部 -->
      <div class="card-header">
        <div class="title-section">
          <div class="title-row">
            <div class="model-info">
              <div class="model-details">
                <div class="brand-badge gpu">
                  {{ gpuInfo.vendor }}
                </div>
                <span class="model-series">{{ gpuInfo.model }}</span>
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
          <ProgressBar :percentage="usage" type="gpu" />
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

.brand-badge.gpu {
  background: linear-gradient(135deg, #ed1c24, #ff4654);
}

.model-series {
  color: #9ca3af;
  font-size: 0.7rem;
  font-weight: 500;
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

.stat-card-wrapper + .stat-card-wrapper {
  margin-top: 24px;
}
</style> 