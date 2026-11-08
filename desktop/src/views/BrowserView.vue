<template>
  <div @mousemove="onGlobalMouseMove" class="flex flex-col h-screen w-screen bg-[#08080a] text-zinc-200 select-none font-sans overflow-hidden">
    
    <!-- HEADER WRAPPER ДЛЯ ИЗМЕРЕНИЯ ВЫСОТЫ -->
    <div 
      ref="headerWrapper" 
      v-show="!isFullscreen || isHeaderRevealed"
      @mouseleave="onHeaderMouseLeave"
      class="flex flex-col w-full shrink-0 z-30 shadow-md"
    >
      <!-- 1. ПАНЕЛЬ ВКЛАДОК -->
    <div @mousedown="startDragging" class="flex items-center bg-[#0d0d10] px-2 pt-1.5 border-b border-zinc-800/80 relative z-20 shrink-0 w-full h-11">
      <!-- Left scroll arrow -->
      <button @mousedown.stop v-show="showScrollArrows" @click="scrollTabs(-200)" class="z-10 bg-gradient-to-r from-[#0d0d10] via-[#0d0d10] to-transparent pr-4 text-zinc-500 hover:text-zinc-200">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
      </button>

      <div ref="tabsContainer" class="flex items-center overflow-x-auto scrollbar-hide flex-1 scroll-smooth" style="scrollbar-width: none; -ms-overflow-style: none;">
      <div 
        v-for="tab in browserStore.tabs" 
        :key="tab.id"
        @click="setActiveTab(tab.id)"
        @mousedown.stop
        class="flex items-center justify-between px-3 py-1.5 min-w-[64px] w-[192px] shrink text-xs cursor-pointer rounded-t-xl transition-all mr-1.5 group relative border-t border-x"
        :class="tab.id === browserStore.activeTabId 
          ? 'bg-[#15151b] text-zinc-100 border-zinc-700/80 shadow-sm font-medium' 
          : 'bg-[#0d0d10] text-zinc-500 border-transparent hover:bg-[#121217] hover:text-zinc-300'"
      >
        <div class="flex items-center space-x-2 truncate pr-2">
          <div v-if="tab.isLoading" class="w-3.5 h-3.5 border-2 border-zinc-400 border-t-transparent rounded-full animate-spin flex-shrink-0"></div>
          
          <img 
            v-else-if="getTabFavicon(tab)" 
            :src="getTabFavicon(tab)" 
            class="w-3.5 h-3.5 rounded-sm flex-shrink-0 object-contain"
            @error="(e) => onFaviconError(tab)"
          />
          
          <svg v-else class="w-3.5 h-3.5 text-zinc-500 flex-shrink-0 object-contain" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="2" y1="12" x2="22" y2="12"></line>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
          </svg>
          
          <span class="truncate">{{ getTabTitle(tab) }}</span>
        </div>

        <button 
          @click.stop="closeTab(tab.id)" 
          class="text-zinc-500 hover:text-zinc-200 font-medium rounded-md w-4 h-4 flex items-center justify-center hover:bg-zinc-700/50 transition-colors"
        >
          ✕
        </button>
      </div>
      <!-- Новая вкладка (внутри контейнера скролла) -->
      <button 
        @mousedown.stop
        @click="addNewTab" 
        class="ml-0.5 text-zinc-500 hover:text-zinc-200 text-sm w-6 h-6 flex items-center justify-center rounded-lg hover:bg-zinc-800 transition-colors shrink-0"
        title="Новая вкладка"
      >
        +
      </button>
      </div>
      
      <!-- Right scroll arrow -->
      <button @mousedown.stop v-show="showScrollArrows" @click="scrollTabs(200)" class="z-10 bg-gradient-to-l from-[#0d0d10] via-[#0d0d10] to-transparent pl-4 text-zinc-500 hover:text-zinc-200">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
      </button>

      <!-- Оконные кнопки справа -->
      <div class="ml-auto flex items-center h-full mb-1">
        <button @mousedown.stop @click="minimizeWindow" class="w-9 h-full flex items-center justify-center text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors rounded-md mx-0.5">
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" /></svg>
        </button>
        <button @mousedown.stop @click="maximizeWindow" class="w-9 h-full flex items-center justify-center text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors rounded-md mx-0.5">
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" /></svg>
        </button>
        <button @mousedown.stop @click="closeWindow" class="w-9 h-full flex items-center justify-center text-zinc-400 hover:text-white hover:bg-red-500 transition-colors rounded-md ml-0.5">
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
        </button>
      </div>
    </div>

    <!-- 2. АДРЕСНАЯ СТРОКА -->
    <div class="flex items-center space-x-2 bg-[#15151b] p-2 border-b border-zinc-800/80 shrink-0">
      
      <div class="flex items-center space-x-1">
        <button @click="goBack" class="p-1.5 text-zinc-400 hover:text-zinc-100 rounded-lg hover:bg-zinc-800 transition-colors" title="Назад">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><path d="M12 19l-7-7 7-7"/></svg>
        </button>

        <button @click="goForward" class="p-1.5 text-zinc-400 hover:text-zinc-100 rounded-lg hover:bg-zinc-800 transition-colors" title="Вперед">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5l7 7-7 7"/></svg>
        </button>

        <button @click="reloadCurrentTab" class="p-1.5 text-zinc-400 hover:text-zinc-100 rounded-lg hover:bg-zinc-800 transition-colors" title="Обновить">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
        </button>
      </div>
      
      <form @submit.prevent="navigate" class="flex-1 flex">
        <div class="relative w-full flex items-center">
          <span class="absolute left-3 text-emerald-400">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </span>
          <input 
            v-model="inputUrl" 
            type="text" 
            placeholder="Введите URL-адрес или поисковый запрос..."
            class="w-full bg-[#0d0d11] text-zinc-200 border border-zinc-800 rounded-xl pl-9 pr-24 py-1.5 focus:outline-none focus:border-zinc-600 text-xs transition-all placeholder-zinc-600 shadow-inner"
          >
          <span class="absolute right-3 text-[10px] font-mono text-emerald-500/80 select-none hidden sm:inline">
            CHROMIUM ENGINE
          </span>
        </div>
      </form>

      <!-- Дополнительные элементы управления: Масштаб и Загрузки -->
      <div class="flex items-center space-x-1 pl-1">
        
        <!-- Кнопки масштаба (Zoom) -->
        <div class="flex items-center bg-[#101015] border border-zinc-800/80 rounded-lg p-0.5 text-zinc-400">
          <button 
            @click="zoomOut" 
            class="w-6 h-6 flex items-center justify-center hover:text-zinc-100 hover:bg-zinc-800/60 rounded text-xs transition-colors"
            title="Уменьшить масштаб (Ctrl -)"
          >
            -
          </button>
          
          <button 
            @click="zoomReset" 
            class="px-1.5 h-6 flex items-center justify-center text-[11px] font-mono hover:text-zinc-100 rounded transition-colors"
            :class="activeZoom !== 1.0 ? 'text-emerald-400 font-semibold' : 'text-zinc-400'"
            title="Сбросить масштаб (Ctrl 0)"
          >
            {{ Math.round(activeZoom * 100) }}%
          </button>

          <button 
            @click="zoomIn" 
            class="w-6 h-6 flex items-center justify-center hover:text-zinc-100 hover:bg-zinc-800/60 rounded text-xs transition-colors"
            title="Увеличить масштаб (Ctrl +)"
          >
            +
          </button>
        </div>

        <!-- Кнопка Полноэкранного режима (F11) -->
        <button 
          @click="toggleFullscreen" 
          class="p-1.5 rounded-lg hover:bg-zinc-800 transition-colors flex items-center justify-center"
          :class="isFullscreen ? 'text-emerald-400 bg-zinc-800/60' : 'text-zinc-400 hover:text-zinc-100'"
          :title="isFullscreen ? 'Выйти из полноэкранного режима (F11)' : 'Полноэкранный режим (F11)'"
        >
          <svg v-if="!isFullscreen" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
          <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 4v4H5m4 0L4 3m11 1v4h4m-4 0l5-5m-5 11v4h4m-4-4l5 5M9 15v4H5m4-4l-5 5" />
          </svg>
        </button>

        <!-- Кнопка Загрузок (Downloads) -->
        <button 
          ref="downloadBtnRef"
          @click="toggleDownloadsWidget" 
          class="p-1.5 rounded-lg hover:bg-zinc-800 transition-colors relative flex items-center justify-center"
          :class="isDownloadsOpen || sessionDownloads.length > 0 ? 'text-emerald-400 bg-zinc-800/40' : 'text-zinc-400 hover:text-zinc-100'"
          title="Загрузки текущей сессии"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          <span 
            v-if="sessionDownloads.length > 0" 
            class="absolute -top-1 -right-1 bg-emerald-500 text-zinc-950 font-bold text-[9px] min-w-4 h-4 px-1 rounded-full flex items-center justify-center shadow-lg animate-pulse"
          >
            {{ sessionDownloads.length }}
          </span>
        </button>

        <!-- Кнопка Выхода (Вернуться в обычный браузер) -->
        <button 
          @click="handleLogout" 
          class="ml-2 pl-2.5 pr-3 py-1.5 flex items-center justify-center space-x-1.5 bg-gradient-to-r from-red-500/10 to-orange-500/10 hover:from-red-500/20 hover:to-orange-500/20 border border-red-500/20 hover:border-red-500/40 text-red-400 hover:text-red-300 rounded-lg transition-all shadow-sm group"
          title="Вернуться в обычный браузер"
        >
          <svg class="w-3.5 h-3.5 group-hover:-translate-x-0.5 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
          </svg>
          <span class="text-[11px] font-medium tracking-wide">ВЫЙТИ</span>
        </button>

      </div>
    </div>

    <!-- ВЫЕЗЖАЮЩАЯ ПАНЕЛЬ ЗАГРУЗОК УДАЛЕНА И ПЕРЕНЕСЕНА В ВИДЖЕТ -->
    <!-- ВСПЛЫВАЮЩИЙ БАННЕР СКАЧИВАНИЯ УДАЛЕН И ПЕРЕНЕСЕН В ВИДЖЕТ -->

    <!-- АНИМИРОВАННЫЙ БАННЕР УСПЕШНОГО СКАЧИВАНИЯ -->
    <div 
      v-if="flyingItem" 
      class="bg-gradient-to-r from-emerald-600 via-emerald-500 to-teal-500 text-zinc-950 px-4 py-1.5 text-xs font-bold flex items-center justify-between shadow-lg transition-all duration-300"
    >
      <div class="flex items-center space-x-2">
        <div class="w-4 h-4 rounded-full bg-zinc-950/20 flex items-center justify-center animate-bounce">
          <svg class="w-2.5 h-2.5 text-zinc-950" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </div>
        <span>Файл сохранен: <span class="font-mono text-zinc-950 font-extrabold">{{ flyingItem.filename }}</span></span>
      </div>
      <button @click="toggleDownloadsWidget" class="underline text-[11px] hover:text-white transition-colors">
        Посмотреть в загрузках →
      </button>
    </div>

    </div> <!-- END HEADER WRAPPER -->

    <!-- 3. СТАРТОВАЯ СТРАНИЦА (Отображается когда вкладка пустая) -->
    <div v-show="isCurrentStartPage" class="flex-1 bg-[#08080a] relative overflow-hidden flex flex-col items-center justify-center px-4">
      
      <div class="w-full max-w-2xl flex flex-col items-center py-6">
        
        <div class="flex flex-col items-center mb-7 text-center">
          <div class="w-12 h-12 bg-[#111115] border border-zinc-700/80 rounded-2xl flex items-center justify-center text-zinc-100 mb-3 shadow-xl">
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.75" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h1 class="text-2xl font-bold text-zinc-100 tracking-tight">Chabupelik Browser</h1>
        </div>

        <!-- Поисковая строка -->
        <form @submit.prevent="handleStartPageSearch" class="w-full flex items-center bg-[#111116] border border-zinc-700/80 rounded-2xl p-1.5 shadow-2xl focus-within:border-zinc-500 transition-all mb-8">
          <span class="pl-3.5 pr-2 text-zinc-500">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </span>
          <input v-model="startPageInput" type="text" placeholder="Поиск в интернете или ввод URL..." class="w-full bg-transparent text-zinc-100 placeholder-zinc-500 text-xs focus:outline-none px-2 py-2" autofocus />
          
          <button type="submit" class="bg-zinc-100 hover:bg-white text-zinc-950 p-2.5 rounded-xl transition-all shadow-md active:scale-95 shrink-0 flex items-center justify-center ml-1" title="Искать">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
        </form>



      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAuthStore } from '../stores/auth.store';
import { useBrowserStore } from '../stores/browser.store';
import api from '../services/api';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { getCurrentWindow } from '@tauri-apps/api/window';

const updateAppTitle = (title) => {
  invoke('native_set_window_title', { title: `${title || 'Новая вкладка'} - Google Chrome` })
    .catch((e) => alert(`Error setting title: ${e}`));
};

const headerWrapper = ref(null);
let resizeObserver = null;
let tabResizeObserver = null;

const minimizeWindow = () => invoke('native_minimize_window');
const maximizeWindow = () => invoke('native_maximize_window');
const closeWindow = () => invoke('native_close_window');
const startDragging = () => invoke('native_start_dragging');

const authStore = useAuthStore();
const browserStore = useBrowserStore();
const tabsContainer = ref(null);
const showScrollArrows = ref(false);

const checkScrollArrows = () => {
  if (tabsContainer.value) {
    showScrollArrows.value = tabsContainer.value.scrollWidth > tabsContainer.value.clientWidth;
  }
};

// Zoom State & Controls
const tabZooms = ref({});

const activeZoom = computed(() => {
  const tabId = browserStore.activeTabId;
  return tabZooms.value[tabId] || 1.0;
});

const setZoom = async (factor) => {
  const tabId = browserStore.activeTabId;
  if (!tabId) return;
  const clamped = Math.round(Math.min(2.5, Math.max(0.5, factor)) * 10) / 10;
  tabZooms.value[tabId] = clamped;
  try {
    await invoke('native_set_zoom', { tabId, zoomFactor: clamped });
  } catch (err) {
    console.error('Error setting zoom:', err);
  }
};

const zoomIn = () => setZoom(activeZoom.value + 0.1);
const zoomOut = () => setZoom(activeZoom.value - 0.1);
const zoomReset = () => setZoom(1.0);

const isFullscreen = ref(false);
const isHeaderRevealed = ref(false);
const measuredHeaderHeight = ref(84);

const setHeaderVisible = async (visible) => {
  if (!isFullscreen.value) return;
  isHeaderRevealed.value = visible;
  const height = visible ? (measuredHeaderHeight.value || 84) : 0;
  try {
    await invoke('native_update_top_bar_height', { height });
  } catch (err) {
    console.error('Error updating top bar height:', err);
  }
};

const toggleFullscreen = async () => {
  try {
    const nextState = await invoke('native_toggle_fullscreen');
    isFullscreen.value = nextState;
    if (nextState) {
      isHeaderRevealed.value = false;
      await invoke('native_update_top_bar_height', { height: 0 }).catch(() => {});
    } else {
      isHeaderRevealed.value = true;
      await invoke('native_update_top_bar_height', { height: measuredHeaderHeight.value || 84 }).catch(() => {});
    }
  } catch (err) {
    console.error('Error toggling fullscreen:', err);
  }
};

const onHeaderMouseLeave = () => {
  if (isFullscreen.value && !isDownloadsOpen.value) {
    setHeaderVisible(false);
  }
};

const onGlobalMouseMove = (e) => {
  if (isFullscreen.value && !isHeaderRevealed.value && e.clientY <= 3) {
    setHeaderVisible(true);
  }
};

const handleGlobalKeydown = (e) => {
  if (e.key === 'F11') {
    e.preventDefault();
    toggleFullscreen();
    return;
  }
  if (e.key === 'Escape' && isFullscreen.value) {
    e.preventDefault();
    invoke('native_exit_fullscreen_if_active').then(() => {
      isFullscreen.value = false;
      isHeaderRevealed.value = true;
    }).catch(console.error);
    return;
  }
  if (e.ctrlKey || e.metaKey) {
    if (e.key === '=' || e.key === '+') {
      e.preventDefault();
      zoomIn();
    } else if (e.key === '-' || e.key === '_') {
      e.preventDefault();
      zoomOut();
    } else if (e.key === '0') {
      e.preventDefault();
      zoomReset();
    }
  }
};

const sessionDownloads = ref([]);
const isDownloadsOpen = ref(false); // Used just for button active state
const downloadBtnRef = ref(null);
const flyingItem = ref(null);

let lastWidgetCloseTime = 0;

const syncDownloadsToWidget = () => {
  const channel = new BroadcastChannel('downloads-channel');
  channel.postMessage({ type: 'sync', data: JSON.parse(JSON.stringify(sessionDownloads.value)) });
};

const toggleDownloadsWidget = async () => {
  if (Date.now() - lastWidgetCloseTime < 200) {
    return; // Prevent immediate reopen if closed by blur
  }
  try {
    const widget = await WebviewWindow.getByLabel('downloads-widget');
    if (widget) {
      const isVisible = await widget.isVisible();
      if (isVisible) {
        await widget.hide();
        isDownloadsOpen.value = false;
      } else {
        const mainWindow = getCurrentWindow();
        const pos = await mainWindow.outerPosition();
        const size = await mainWindow.outerSize();
        
        // Move widget to top right corner of main window
        await widget.setPosition(new PhysicalPosition(pos.x + size.width - 320 - 10, pos.y + 85));
        
        await widget.show();
        await widget.setFocus();
        isDownloadsOpen.value = true;
      }
    }
  } catch (err) {
    console.error('Ошибка переключения виджета загрузок:', err);
  }
};

const triggerFlyingAnimation = (filename) => {
  flyingItem.value = { filename };
  setTimeout(() => {
    flyingItem.value = null;
  }, 4000);
};

const handleNewDownload = async (data) => {
  const defaultDir = await invoke('native_get_downloads_dir').catch(() => '');
  const fileName = data.filename || 'download';
  const fullPath = defaultDir ? `${defaultDir}\\${fileName}` : fileName;

  const item = {
    id: Date.now().toString(),
    filename: fileName,
    url: data.url,
    path: fullPath,
    time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    size: 'Загружается...'
  };
  
  sessionDownloads.value.unshift(item);
  syncDownloadsToWidget();
  triggerFlyingAnimation(item.filename);
};

const channel = new BroadcastChannel('downloads-channel');
channel.onmessage = (e) => {
  if (e.data.type === 'request-sync') {
    syncDownloadsToWidget();
  } else if (e.data.type === 'clear') {
    sessionDownloads.value = [];
  } else if (e.data.type === 'widget-closed') {
    isDownloadsOpen.value = false;
    lastWidgetCloseTime = Date.now();
  }
};

const closeWidgetExternally = async () => {
  if (!isDownloadsOpen.value) return; // CRITICAL: Prevents IPC flood during resize!
  
  isDownloadsOpen.value = false;
  try {
    const widget = await WebviewWindow.getByLabel('downloads-widget');
    if (widget) {
      await widget.hide();
    }
  } catch (e) {
    console.error(e);
  }
};

const handleMainWindowMousedown = (e) => {
  if (isDownloadsOpen.value && downloadBtnRef.value && !downloadBtnRef.value.contains(e.target)) {
    closeWidgetExternally();
  }
};

onMounted(async () => {
  try {
    if (!authStore.deviceId) {
      await authStore.initDeviceId();
    }
  } catch (err) {
    console.error("Failed to init device ID in BrowserView:", err);
  }

  if (tabsContainer.value) {
    tabResizeObserver = new ResizeObserver(() => checkScrollArrows());
    tabResizeObserver.observe(tabsContainer.value);
  }
  
  window.addEventListener('resize', checkScrollArrows);
  window.addEventListener('keydown', handleGlobalKeydown);
  window.addEventListener('mousedown', handleMainWindowMousedown);

  const mainWindow = getCurrentWindow();
  mainWindow.onMoved(closeWidgetExternally);
  mainWindow.onResized(closeWidgetExternally);
  mainWindow.onFocusChanged(({ payload: focused }) => {
    // Если главное окно получает фокус (например клик по нативному webview страницы)
    // или теряет фокус (альт-таб), закрываем виджет.
    // Виджет имеет свое окно, поэтому когда он открыт - главное окно не в фокусе.
    closeWidgetExternally();
  });

  const activeTab = browserStore.tabs.find(t => t.id === browserStore.activeTabId);
  if (activeTab && activeTab.url && activeTab.url !== 'about:blank') {
    invoke('native_navigate_tab', { tabId: activeTab.id, url: activeTab.url }).catch(console.error);
  }
  
  // Инициализация ResizeObserver для передачи высоты Header'а в Rust
  resizeObserver = new ResizeObserver((entries) => {
    for (let entry of entries) {
      if (entry.target === headerWrapper.value) {
        const height = entry.contentRect.height;
        if (height > 0) {
          measuredHeaderHeight.value = height;
        }
        if (!isFullscreen.value || isHeaderRevealed.value) {
          invoke('native_update_top_bar_height', { height }).catch(console.error);
        } else {
          invoke('native_update_top_bar_height', { height: 0 }).catch(console.error);
        }
      }
    }
  });
  if (headerWrapper.value) {
    resizeObserver.observe(headerWrapper.value);
  }

  invoke('native_is_fullscreen').then((fs) => {
    isFullscreen.value = !!fs;
  }).catch(() => {});

  listen('fullscreen-changed', (event) => {
    isFullscreen.value = !!event.payload;
    if (isFullscreen.value) {
      isHeaderRevealed.value = false;
      invoke('native_update_top_bar_height', { height: 0 }).catch(() => {});
    } else {
      isHeaderRevealed.value = true;
      invoke('native_update_top_bar_height', { height: measuredHeaderHeight.value || 84 }).catch(() => {});
    }
  });

  listen('show-fullscreen-header', (event) => {
    if (isFullscreen.value && !isHeaderRevealed.value) {
      setHeaderVisible(true);
    }
  });

  // Подписка на события загрузки страниц из Rust
  listen('webview-loading', (event) => {
    const { tabId, url } = event.payload;
    const tab = browserStore.tabs.find(t => t.id === tabId);
    if (tab) {
      tab.isLoading = true;
      if (browserStore.activeTabId === tabId && url) {
        inputUrl.value = formatDisplayUrl(url);
      }
    }
  });

  listen('webview-loaded', (event) => {
    const { tabId, url } = event.payload;
    const tab = browserStore.tabs.find(t => t.id === tabId);
    if (tab) {
      tab.isLoading = false;
      tab.url = url;
      if (browserStore.activeTabId === tabId) {
        inputUrl.value = formatDisplayUrl(url);
        // Force show the webview once it's loaded to fix the issue where it stays hidden
        invoke('native_switch_tab', { tabId }).catch(() => {});
      }
    }
  });

  listen('webview-title-changed', (event) => {
    const { tabId, tab_id, title } = event.payload || {};
    const tId = tabId || tab_id;
    const tab = browserStore.tabs.find(t => t.id === tId);
    if (tab && title && title.trim()) {
      tab.title = title.trim();
      if (browserStore.activeTabId === tId) {
        updateAppTitle(tab.title);
      }
    }
  });

  listen('webview-favicon-changed', (event) => {
    const { tab_id, favicon } = event.payload || {};
    if (tab_id && favicon) {
      const tab = browserStore.tabs.find(t => t.id === tab_id);
      if (tab) {
        tab.favicon = favicon;
      }
    }
  });

  listen('webview-download-started', (event) => {
    const { tabId, url, filename } = event.payload || {};
    handleNewDownload({ url, filename: filename || 'file' });
  });

  // Открытие новой вкладки по запросу из WebView (window.open / target=_blank)
  listen('open-new-tab', (event) => {
    const { url } = event.payload || {};
    if (!url) return;
    browserStore.addTab(url);
  });
});

onUnmounted(() => {
  if (tabResizeObserver) {
    tabResizeObserver.disconnect();
  }
  window.removeEventListener('resize', checkScrollArrows);
  window.removeEventListener('keydown', handleGlobalKeydown);

  if (resizeObserver && headerWrapper.value) {
    resizeObserver.unobserve(headerWrapper.value);
  }
});

watch(() => browserStore.tabs.length, () => {
  setTimeout(checkScrollArrows, 50);
});

watch(() => browserStore.activeTabId, (newId) => {
  const activeTab = browserStore.tabs.find(t => t.id === newId);
  if (activeTab) {
    updateAppTitle(activeTab.title);
  }
}, { immediate: true });

const scrollTabs = (offset) => {
  if (tabsContainer.value) {
    tabsContainer.value.scrollBy({ left: offset, behavior: 'smooth' });
  }
};
const formatDisplayUrl = (url) => {
  if (!url) return '';
  let display = url;
  if (display.startsWith('http://')) display = display.substring(7);
  if (display.startsWith('https://')) display = display.substring(8);
  if (display.startsWith('www.')) display = display.substring(4);
  if (display.endsWith('/')) display = display.substring(0, display.length - 1);
  return display;
};
const inputUrl = ref('');
const startPageInput = ref('');
const failedFavicons = ref(new Set());

const isCurrentStartPage = computed(() => {
  const activeTab = browserStore.tabs.find(t => t.id === browserStore.activeTabId);
  return !activeTab || !activeTab.url || activeTab.url === '' || activeTab.url === 'about:blank';
});

watch(() => browserStore.activeTabId, async (newId) => {
  const activeTab = browserStore.tabs.find(t => t.id === newId);
  if (activeTab) {
    inputUrl.value = formatDisplayUrl(activeTab.url || '');
    startPageInput.value = '';
    try {
      await invoke('native_switch_tab', { tabId: newId });
      // Восстановить масштаб вкладки
      const zoom = tabZooms.value[newId] || 1.0;
      await invoke('native_set_zoom', { tabId: newId, zoomFactor: zoom });
    } catch (e) {
      if (e === 'NOT_FOUND' && activeTab.url && activeTab.url !== 'about:blank') {
        activeTab.isLoading = true;
        invoke('native_navigate_tab', { tabId: newId, url: activeTab.url }).catch(() => {
          activeTab.isLoading = false;
        });
      }
    }
  }
}, { immediate: true });

const navigate = async () => {
  processNavigation(browserStore.activeTabId, inputUrl.value);
};

const handleStartPageSearch = () => {
  processNavigation(browserStore.activeTabId, startPageInput.value);
};

const openUrlInActiveTab = (url) => {
  processNavigation(browserStore.activeTabId, url);
};

const processNavigation = async (tabId, targetUrl) => {
  if (!targetUrl.trim()) return;
  
  let url = targetUrl.trim();
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    if (url.includes('.') && !url.includes(' ')) {
      url = `https://${url}`;
    } else {
      url = `https://duckduckgo.com/?q=${encodeURIComponent(url)}`;
    }
  }

  const tab = browserStore.tabs.find(t => t.id === tabId);
  if (tab) {
    tab.isLoading = true;
    tab.url = url;
  }
  
  if (tabId === browserStore.activeTabId) {
    inputUrl.value = formatDisplayUrl(url);
  }
  
  try {
    await invoke('native_navigate_tab', { tabId, url });
    
    setTimeout(() => {
      const currentTab = browserStore.tabs.find(t => t.id === tabId);
      if (currentTab && currentTab.isLoading) {
        currentTab.isLoading = false;
      }
    }, 15000);
    
  } catch (err) {
    console.error('Ошибка навигации нативного Webview:', err);
    if (tab) tab.isLoading = false;
  }
};

const setActiveTab = async (id) => {
  browserStore.setActiveTab(id);
  try {
    await invoke('native_switch_tab', { tabId: id });
  } catch (_) {}
};

const closeTab = async (id) => {
  try {
    await invoke('native_close_tab', { tabId: id });
  } catch (_) {}
  browserStore.closeTab(id);
  const active = browserStore.tabs.find(t => t.id === browserStore.activeTabId);
  if (active) {
    if (!active.url || active.url === 'about:blank') {
      try { await invoke('native_navigate_tab', { tabId: active.id, url: '' }); } catch (_) {}
    } else {
      try { await invoke('native_switch_tab', { tabId: active.id }); } catch (_) {}
    }
  }
};

const goBack = () => {
  invoke('native_go_back', { tabId: browserStore.activeTabId }).catch(console.error);
};

const goForward = () => {
  invoke('native_go_forward', { tabId: browserStore.activeTabId }).catch(console.error);
};

const reloadCurrentTab = () => {
  invoke('native_reload', { tabId: browserStore.activeTabId }).catch(console.error);
};

const addNewTab = () => {
  browserStore.addTab('');
};

const getTabFavicon = (tab) => {
  if (failedFavicons.value.has(tab.id) || !tab.url || tab.url.startsWith('about:')) return null;
  if (tab.favicon) return tab.favicon; // Использовать нативную иконку, если есть
  
  try {
    const url = new URL(tab.url);
    return `https://www.google.com/s2/favicons?domain=${url.hostname}&sz=128`;
  } catch (e) {
    return null;
  }
};

const onFaviconError = (tab) => {
  if (tab && tab.id) failedFavicons.value.add(tab.id);
};

const getTabTitle = (tab) => {
  if (tab.title && tab.title.trim() !== '' && tab.title !== 'Новая вкладка') return tab.title;
  if (!tab.url || tab.url === 'about:blank') return 'Новая вкладка';
  return extractHostname(tab.url);
};

const extractHostname = (urlStr) => {
  try {
    const u = new URL(urlStr);
    return u.hostname.replace('www.', '');
  } catch (e) {
    return urlStr;
  }
};

const handleLogout = async () => {
  await authStore.logout();
};
</script>