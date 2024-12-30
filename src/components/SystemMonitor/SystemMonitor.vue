<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import CpuCard from './CpuCard.vue';
import GpuCard from './GpuCard.vue';
import StressTestCard from './StressTestCard.vue';

interface SystemStats {
  cpuUsage: number;
  gpuUsage: number;
  cpuTemp: number;
  gpuTemp: number;
  cpuCores: Array<{
    usage: number;
    temperature: number;
  }>;
  fanSpeed: number;
  cpuInfo: {
    vendor: string;
    model: string;
    cores: number;
    threads: number;
  };
  gpuFanSpeed: number;
}

const UPDATE_INTERVAL = 1000;
const TEMP_THRESHOLD = 100;

const stats = ref<SystemStats>({
  cpuUsage: 0,
  gpuUsage: 0,
  cpuTemp: 0,
  gpuTemp: 0,
  cpuCores: [],
  fanSpeed: 0,
  cpuInfo: {
    vendor: '',
    model: '',
    cores: 0,
    threads: 0
  },
  gpuFanSpeed: 0
});

const isTestRunning = ref(false);
const showWarning = ref(false);

const initCpuInfo = async () => {
  try {
    const cpuInfo = await invoke<{
      vendor: string;
      model: string;
      cores: number;
      threads: number;
    }>('get_cpu_info');
    
    stats.value.cpuInfo = cpuInfo;
  } catch (error) {
    console.error('Failed to get CPU info:', error);
  }
};

const updateStats = async () => {
  try {
    const update = async () => {
      try {
        const [
          cpuData,
          cpuTemp,
          coreTemps,
          fanSpeeds,
          gpuData
        ] = await Promise.all([
          invoke<[number[], number[], number]>('get_cpu_usage'),
          invoke<number>('get_cpu_temp'),
          invoke<Array<[number, number]>>('get_all_core_temps'),
          invoke<Array<[number, number]>>('get_all_fan_speeds'),
          invoke<[number, number, number]>('get_actual_gpu_stats')
        ]);

        const [_, coreUsage, totalUsage] = cpuData;
        const [gpuUsage, gpuTemp, gpuFan] = gpuData;

        if (isTestRunning.value && (Math.round(cpuTemp) >= TEMP_THRESHOLD || gpuTemp >= TEMP_THRESHOLD)) {
          await stopTest();
          showWarning.value = true;
        }

        requestAnimationFrame(() => {
          stats.value = {
            ...stats.value,
            cpuUsage: totalUsage,
            cpuTemp: Math.round(cpuTemp),
            cpuCores: coreTemps.map((temp, index) => ({
              usage: coreUsage[index] || 0,
              temperature: temp[1]
            })),
            fanSpeed: fanSpeeds[0]?.[1] || 0,
            gpuUsage,
            gpuTemp,
            gpuFanSpeed: gpuFan
          };
        });
      } catch (error) {
        console.error('Failed to update stats:', error);
      }
      
      updateTimeout = setTimeout(() => {
        requestAnimationFrame(update);
      }, UPDATE_INTERVAL);
    };

    update();
  } catch (error) {
    console.error('Failed to start update loop:', error);
    updateTimeout = setTimeout(updateStats, UPDATE_INTERVAL);
  }
};

const startTest = async (type: 'cpu' | 'gpu' | 'both') => {
  isTestRunning.value = true;
  showWarning.value = false;
  
  try {
    await invoke('start_stress_test', { testType: type });
  } catch (error) {
    console.error('Failed to start stress test:', error);
    stopTest();
  }
};

const stopTest = async () => {
  isTestRunning.value = false;
  try {
    await Promise.all([
      invoke('stop_stress_test'),
      invoke('stop_gpu_stress_test')
    ]);
  } catch (error) {
    console.error('Failed to stop stress test:', error);
  }
};

watch([() => stats.value.cpuTemp, () => stats.value.gpuTemp], ([cpuTemp, gpuTemp]) => {
  if (isTestRunning.value && (cpuTemp >= TEMP_THRESHOLD || gpuTemp >= TEMP_THRESHOLD)) {
    stopTest();
    showWarning.value = true;
  }
});

let updateTimeout: ReturnType<typeof setTimeout>;

onMounted(async () => {
  await initCpuInfo();
  updateStats();
});

onUnmounted(() => {
  clearTimeout(updateTimeout);
});
</script>

<template>
  <div class="monitor-container">
    <Transition name="slide-fade">
      <div class="warning-alert" v-if="showWarning">
        <div class="warning-content">
          <span class="warning-icon">⚠️</span>
          <span>温度已达到 100°C，测试已自动停止</span>
        </div>
        <button class="close-button" @click="showWarning = false">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>
    </Transition>
    
    <div class="content-wrapper">
      <div class="cards-grid">
        <CpuCard 
          title="处理器"
          :usage="stats.cpuUsage"
          :temperature="stats.cpuTemp"
          :fan-speed="stats.fanSpeed"
          :cpu-info="stats.cpuInfo"
          :cpu-cores="stats.cpuCores"
        />
        <GpuCard 
          title="显卡"
          :usage="stats.gpuUsage"
          :temperature="stats.gpuTemp"
          :fan-speed="stats.gpuFanSpeed"
        />
        <StressTestCard 
          @start-test="startTest"
          @stop-test="stopTest"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.monitor-container {
  width: 100%;
  min-height: 100%;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  box-sizing: border-box;
  padding-top: 20px;
  padding-bottom: 32px;
}

.content-wrapper {
  width: 100%;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  padding: 0 8px;
  box-sizing: border-box;
}

.cards-grid {
  display: flex;
  flex-direction: column;
  gap: 20px;
  width: 100%;
  align-items: center;
}

/* 基础卡片样式 */
:deep(.stat-card),
:deep(.stress-test-card) {
  width: 100%;
  max-width: 384px;
  box-sizing: border-box;
}

/* 所有卡片的过渡效果 */
:deep(.stat-card),
:deep(.stress-test-card),
:deep(.stat-card-wrapper) {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 当CPU卡片展开时，移动后续元素 */
:deep(.stat-card-wrapper:has(.core-info-card.expanded)) ~ * {
  transform: translateY(var(--content-height));
  transition: transform 0.3s ease;
}

.warning-alert {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(220, 38, 38, 0.2);
  color: #fca5a5;
  padding: 12px 16px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(220, 38, 38, 0.1);
  z-index: 100;
  display: flex;
  align-items: center;
  gap: 12px;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(220, 38, 38, 0.3);
}

.warning-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.close-button {
  background: none;
  border: none;
  padding: 4px;
  color: #fca5a5;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.close-button:hover {
  background: rgba(255, 255, 255, 0.1);
}

/* 过渡动画 */
.slide-fade-enter-active {
  transition: all 0.3s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.2s cubic-bezier(0, 1, 0.5, 1);
}

.slide-fade-enter-from {
  transform: translate(-50%, -100%);
  opacity: 0;
}

.slide-fade-leave-to {
  transform: translate(-50%, -100%);
  opacity: 0;
}

/* 移除之前的特殊间距处理 */
:deep(.stat-card-wrapper:nth-child(2)) {
  margin-top: 0;
}

:deep(.stress-test-card) {
  margin-top: 0;
}

/* 移除 GpuCard.vue 中的额外边距 */
:deep(.stat-card-wrapper + .stat-card-wrapper) {
  margin-top: 0;
}

@media (max-width: 768px) {
  .monitor-container {
    padding: 16px;
    padding-top: 32px;
    padding-bottom: 32px;
  }
  
  .cards-grid {
    gap: 20px;
  }
}
</style> 