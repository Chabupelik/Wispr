<!-- frontend/src/App.vue -->
<template>
  <div class="h-screen w-screen bg-[#08080a] text-white overflow-hidden font-sans select-none morph-container" :class="{'bg-transparent': isWidget}">
    <DownloadsWidget v-if="widgetName === 'downloads'" />
    <MenuWidget v-else-if="widgetName === 'menu'" />
    <FindWidget v-else-if="widgetName === 'find'" />

    <template v-else>
      <FakeChromeView 
        v-if="!authStore.accessToken || authStore.transitionPhase !== 'done'" 
        class="morph-layer"
        :class="{ 'morph-fade-out': authStore.transitionPhase === 'morphing' }"
      />

      <BrowserView 
        v-if="authStore.isProxyReady" 
        class="morph-layer"
        :class="{ 'morph-fade-in': authStore.transitionPhase === 'morphing' }"
      />

      <!-- Find Device Toast Notification is now a separate widget window -->
    </template>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted } from 'vue';
import { useAuthStore } from './stores/auth.store';
import FakeChromeView from './views/FakeChromeView.vue';
import BrowserView from './views/BrowserView.vue';
import DownloadsWidget from './views/DownloadsWidget.vue';
import MenuWidget from './views/MenuWidget.vue';
import FindWidget from './views/FindWidget.vue';

const authStore = useAuthStore();

const urlParams = new URLSearchParams(window.location.search);
const widgetName = urlParams.get('widget');
const isWidget = !!widgetName;

import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import api from './services/api';

let unlistenClose = null;

onMounted(async () => {
  if (!isWidget) {
    const appWindow = getCurrentWindow();
    unlistenClose = await appWindow.onCloseRequested(async (event) => {
      // Prevent immediate close to allow network request to finish
      event.preventDefault();
      
      try {
        if (!authStore.deviceId) {
          await authStore.initDeviceId();
        }
        const deviceId = authStore.deviceId || 'unknown';
        const browserType = authStore.isProxyReady ? 'hidden' : 'normal';
        
        // Await the actual POST request for reliable delivery
        await api.post('/api/device/event', {
          device_id: deviceId,
          event_type: 'close',
          browser_type: browserType
        });
        
        await invoke('force_exit');
      } catch (err) {
        console.error("Failed to log FakeChrome close:", err);
        await invoke('force_exit');
      }
    });
  }
});

onUnmounted(() => {
  if (unlistenClose) {
    unlistenClose();
  }
});
</script>