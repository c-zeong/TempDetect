<script setup lang="ts">
defineEmits(['stress-test']);
</script>

<template>
  <button 
    @click="$emit('stress-test')" 
    class="stress-btn"
  >
    满载测试
  </button>
</template>

<style scoped>
.stress-btn {
  display: block;
  margin: 2rem auto;
  padding: 0.8rem 2rem;
  background: linear-gradient(45deg, #ff416c, #ff4b2b);
  border: none;
  border-radius: 25px;
  color: white;
  font-weight: bold;
  cursor: pointer;
  transition: transform 0.2s;
}

.stress-btn:hover {
  transform: scale(1.05);
}
</style> 