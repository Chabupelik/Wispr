<template>
  <div 
    class="w-full h-full bg-[#282a2d] border border-[#3b3e41] rounded-lg shadow-2xl py-2 text-[13px] text-zinc-300 font-sans tracking-wide overflow-hidden select-none"
    @mousedown.stop
  >
    <!-- Новая вкладка -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex justify-between items-center" @click="sendAction('new-tab')">
      <div class="flex items-center">
        <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h10v4h8v10z"/></svg>
        <span>Новая вкладка</span>
      </div>
      <span class="text-zinc-500">Ctrl+T</span>
    </div>

    <div class="h-[1px] bg-[#3b3e41] my-1 mx-4"></div>

    <!-- Пароли -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex items-center">
      <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M12.65 10C11.83 7.67 9.61 6 7 6c-3.31 0-6 2.69-6 6s2.69 6 6 6c2.61 0 4.83-1.67 5.65-4H17v4h4v-4h2v-4H12.65zM7 14c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2z"/></svg>
      <span>Пароли и автозаполнение</span>
    </div>

    <!-- История -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex items-center">
      <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z"/></svg>
      <span>История</span>
    </div>

    <!-- Загрузки -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex justify-between items-center" @click="sendAction('downloads')">
      <div class="flex items-center">
        <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/></svg>
        <span>Загрузки</span>
      </div>
      <span class="text-zinc-500">Ctrl+J</span>
    </div>

    <!-- Закладки -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex items-center">
      <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M19 18l2 1V3c0-1.1-.9-2-2-2H8.99C7.89 1 7 1.9 7 3h10c1.1 0 2 .9 2 2v13zM15 5H5c-1.1 0-2 .9-2 2v16l7-3 7 3V7c0-1.1-.9-2-2-2z"/></svg>
      <span>Закладки и списки</span>
    </div>

    <!-- Расширения -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex items-center">
      <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5s-1.12-2.5-2.5-2.5z"/></svg>
      <span>Расширения</span>
    </div>

    <div class="h-[1px] bg-[#3b3e41] my-1 mx-4"></div>

    <!-- Удалить данные -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex justify-between items-center">
      <div class="flex items-center">
        <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M16 9v10H8V9h8m-1.5-6h-5l-1 1H5v2h14V4h-3.5l-1-1zM18 7H6v12c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7z"/></svg>
        <span>Удалить данные о работе в браузере...</span>
      </div>
      <span class="text-zinc-500">Ctrl+Shift+Del</span>
    </div>

    <div class="h-[1px] bg-[#3b3e41] my-1 mx-4"></div>

    <!-- Масштаб -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex justify-between items-center">
      <div class="flex items-center">
        <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
        <span>Масштаб</span>
      </div>
      <div class="flex items-center space-x-2 border border-zinc-600 rounded bg-[#202124] overflow-hidden">
        <button class="px-2 hover:bg-[#3b3e41] pb-0.5" @click.stop="sendAction('zoom-out')">-</button>
        <span class="text-[12px]">{{ zoomLevel }}%</span>
        <button class="px-2 hover:bg-[#3b3e41] pb-0.5" @click.stop="sendAction('zoom-in')">+</button>
      </div>
    </div>

    <!-- Печать -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex justify-between items-center">
      <div class="flex items-center">
        <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M19 8H5c-1.66 0-3 1.34-3 3v6h4v4h12v-4h4v-6c0-1.66-1.34-3-3-3zm-3 11H8v-5h8v5zm3-7c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1zm-1-9H6v4h12V3z"/></svg>
        <span>Печать...</span>
      </div>
      <span class="text-zinc-500">Ctrl+P</span>
    </div>

    <div class="h-[1px] bg-[#3b3e41] my-1 mx-4"></div>

    <!-- Настройки -->
    <div class="px-4 py-2 hover:bg-[#3b3e41] cursor-default flex items-center">
      <svg class="w-4 h-4 mr-3 text-zinc-400" viewBox="0 0 24 24" fill="currentColor"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>
      <span>Настройки</span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const zoomLevel = ref(100);
const channel = new BroadcastChannel('menu-widget-channel');

channel.onmessage = (e) => {
  if (e.data.type === 'zoom-update') {
    zoomLevel.value = Math.round(e.data.zoom * 100);
  }
};

const sendAction = (action) => {
  channel.postMessage({ type: 'action', action });
  getCurrentWindow().hide();
};

// Close on click outside (blur)
onMounted(() => {
  getCurrentWindow().onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      getCurrentWindow().hide();
      channel.postMessage({ type: 'widget-closed' });
    }
  });
});
</script>
