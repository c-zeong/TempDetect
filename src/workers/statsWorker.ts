import { invoke } from '@tauri-apps/api/core';

let updateInterval: number | null = null;

self.onmessage = async (e: MessageEvent) => {
  if (e.data === 'start') {
    updateInterval = self.setInterval(async () => {
      try {
        const [cpuStats, gpuStats] = await Promise.all([
          invoke('get_cpu_usage'),
          invoke('get_gpu_stats')
        ]);
        self.postMessage({ cpuStats, gpuStats });
      } catch (error) {
        console.error('Failed to get stats:', error);
      }
    }, 2000);
  } else if (e.data === 'stop') {
    if (updateInterval !== null) {
      clearInterval(updateInterval);
    }
  }
}; 