<script setup lang="ts">
import { useTheme } from './utils/theme';
import SystemMonitor from './components/SystemMonitor/SystemMonitor.vue';

const { initTheme } = useTheme();
initTheme();
</script>

<template>
  <main class="app-container">
    <SystemMonitor />
  </main>
</template>

<style>
@import './styles/global.css';

.app-container {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background-color: #111827;
}
</style>