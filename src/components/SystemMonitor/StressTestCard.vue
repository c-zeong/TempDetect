<template>
  <div class="stress-test-card" :class="{ 'testing': isRunning }">
    <div class="card-header">
      <div class="title-section">
        <div class="title-row">
          <div class="title-group">
            <span class="title">压力测试</span>
            <div v-if="isRunning" class="test-duration">
              <svg class="timer-icon" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor">
                <circle cx="12" cy="12" r="9" stroke-width="2"/>
                <path d="M12 7v5l3 3" stroke-width="2" stroke-linecap="round"/>
              </svg>
              <span>{{ formatDuration(testDuration) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="test-options">
      <button 
        v-for="type in testTypes"
        :key="type.value"
        class="test-type-button"
        :class="{ 
          'active': testType === type.value,
          'disabled': isRunning,
          'active-disabled': isRunning && testType === type.value
        }"
        @click="!isRunning && (testType = type.value)"
      >
        <span class="icon">{{ type.icon }}</span>
        <span class="label">{{ type.label }}</span>
      </button>
    </div>

    <button 
      class="control-button"
      :class="{ 'running': isRunning }"
      @click="isRunning ? stopTest() : startTest()"
    >
      <div class="button-content">
        <span class="control-icon">
          <svg v-if="!isRunning" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14H9V8h2v8zm4 0h-2V8h2v8z"/>
          </svg>
        </span>
        <span class="button-text">{{ isRunning ? '停止测试' : '开始测试' }}</span>
      </div>
      <div class="button-glow"></div>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits(['start-test', 'stop-test']);
const isRunning = ref(false);
const testType = ref<'cpu' | 'gpu' | 'both'>('both');
const testDuration = ref(0);
const timerInterval = ref<number | null>(null);

const formatDuration = (seconds: number) => {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
};

const startTest = async () => {
  try {
    isRunning.value = true;
    testDuration.value = 0;
    
    timerInterval.value = window.setInterval(() => {
      testDuration.value++;
    }, 1000);
    
    switch (testType.value) {
      case 'cpu':
        await invoke('start_stress_test');
        break;
      case 'gpu':
        await invoke('start_gpu_stress_test');
        break;
      case 'both':
        await invoke('start_stress_test');
        await invoke('start_gpu_stress_test');
        break;
    }
  } catch (error) {
    console.error('Failed to start test:', error);
    isRunning.value = false;
    if (timerInterval.value) {
      clearInterval(timerInterval.value);
      timerInterval.value = null;
    }
  }
};

const stopTest = async () => {
  try {
    if (timerInterval.value) {
      clearInterval(timerInterval.value);
      timerInterval.value = null;
    }
    
    switch (testType.value) {
      case 'cpu':
        await invoke('stop_stress_test');
        break;
      case 'gpu':
        await invoke('stop_gpu_stress_test');
        break;
      case 'both':
        await invoke('stop_stress_test');
        await invoke('stop_gpu_stress_test');
        break;
    }
  } catch (error) {
    console.error('Failed to stop test:', error);
  } finally {
    isRunning.value = false;
    testDuration.value = 0;
  }
};

const testTypes = [
  { value: 'cpu' as const, icon: '💻', label: 'CPU' },
  { value: 'gpu' as const, icon: '🎮', label: 'GPU' },
  { value: 'both' as const, icon: '🔥', label: 'CPU+GPU' }
];

onUnmounted(async () => {
  if (isRunning.value) {
    await stopTest();
  }
  if (timerInterval.value) {
    clearInterval(timerInterval.value);
    timerInterval.value = null;
  }
});
</script>

<style scoped>
.stress-test-card {
  background: rgba(30, 41, 59, 0.9);
  border-radius: 16px;
  padding: 16px;
  position: relative;
  border: 1px solid transparent;
  transition: all 0.3s ease;
}

.stress-test-card.testing {
  border-color: rgba(239, 68, 68, 0.2);
  animation: innerGlow 2s ease-in-out infinite;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.title-group {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 12px;
}

.title {
  font-size: 0.875rem;
  font-weight: 600;
  color: #e5e7eb;
  letter-spacing: 0.02em;
}

.test-duration {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  font-size: 0.75rem;
  color: #9ca3af;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  margin-left: auto;
}

.test-options {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-bottom: 16px;
  overflow: hidden;
}

.test-type-button {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 8px;
  color: #e5e7eb;
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
}

.test-type-button .icon {
  font-size: 1.125rem;
}

.test-type-button .label {
  font-size: 0.75rem;
  font-weight: 500;
}

.test-type-button:hover:not(.disabled) {
  background: rgba(255, 255, 255, 0.15);
}

.test-type-button.active:not(.disabled) {
  background: rgba(59, 130, 246, 0.5);
  color: white;
}

.test-type-button.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.test-type-button.active-disabled {
  background: rgba(239, 68, 68, 0.25);
  color: rgba(255, 255, 255, 0.9);
}

.control-button {
  position: relative;
  width: 100%;
  padding: 0;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  overflow: hidden;
  transition: all 0.3s ease;
}

.button-content {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px;
  color: white;
  font-size: 0.8rem;
  font-weight: 500;
  background: linear-gradient(135deg, 
    rgba(59, 130, 246, 0.8),
    rgba(96, 165, 250, 0.8)
  );
  border-radius: 8px;
  transition: all 0.3s ease;
}

.control-button:hover .button-content {
  background: linear-gradient(135deg, 
    rgba(37, 99, 235, 0.8),
    rgba(59, 130, 246, 0.8)
  );
}

.control-button.running .button-content {
  background: linear-gradient(135deg, 
    rgba(239, 68, 68, 0.8),
    rgba(248, 113, 113, 0.8)
  );
}

.control-button.running:hover .button-content {
  background: linear-gradient(135deg, 
    rgba(220, 38, 38, 0.8),
    rgba(239, 68, 68, 0.8)
  );
}

.button-glow {
  position: absolute;
  top: 0;
  left: -100%;
  width: 50%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.2),
    transparent
  );
  animation: buttonGlow 2s ease-in-out infinite;
  will-change: transform;
  contain: paint;
}

.control-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon {
  font-size: 1.25rem;
}

.timer-icon {
  color: #9ca3af;
}

@keyframes innerGlow {
  0% {
    box-shadow: 
      inset 0 0 20px rgba(239, 68, 68, 0.2),
      inset 0 0 40px rgba(239, 68, 68, 0.1);
  }
  50% {
    box-shadow: 
      inset 0 0 30px rgba(239, 68, 68, 0.3),
      inset 0 0 60px rgba(239, 68, 68, 0.2);
  }
  100% {
    box-shadow: 
      inset 0 0 20px rgba(239, 68, 68, 0.2),
      inset 0 0 40px rgba(239, 68, 68, 0.1);
  }
}

@keyframes buttonGlow {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(200%); }
}

@keyframes activeButtonPulse {
  0% {
    background: rgba(239, 68, 68, 0.4);
  }
  50% {
    background: rgba(239, 68, 68, 0.6);
  }
  100% {
    background: rgba(239, 68, 68, 0.4);
  }
}
</style> 