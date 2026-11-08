<template>
  <div class="w-full h-full bg-blue-600/90 backdrop-blur-md text-white rounded-2xl shadow-[0_0_40px_rgba(37,99,235,0.5)] border border-blue-400/30 flex items-center justify-center gap-3 select-none overflow-hidden" data-tauri-drag-region>
    <div class="animate-pulse bg-blue-400 w-4 h-4 rounded-full pointer-events-none"></div>
    <span class="font-medium text-xl tracking-wide pointer-events-none">🔍 Это устройство активно</span>
  </div>
</template>

<script setup>
import { onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

onMounted(() => {
  // Window visibility is now managed by auth.store.js
});
</script>
