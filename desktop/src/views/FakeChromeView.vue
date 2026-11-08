<template>
  <div class="flex flex-col h-screen w-screen bg-[#202124] text-white select-none font-sans overflow-hidden">
    
    <!-- Chrome Header -->
    <div ref="headerWrapper" class="flex flex-col w-full shrink-0 z-30">
      
      <!-- 1. Window Controls & Tabs Bar -->
      <div class="flex items-end bg-[#1f2020] h-[40px] pl-0 relative" @mousedown="startDragging">
        
        <!-- Left scroll arrow -->
        <button 
          v-show="showScrollArrows" 
          @mousedown.stop 
          @click="scrollTabs(-200)" 
          class="z-20 flex items-center justify-center w-7 h-full bg-gradient-to-r from-[#1f2020] via-[#1f2020] to-transparent pr-2 text-zinc-500 hover:text-zinc-200 shrink-0 self-stretch"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
        </button>

        <div ref="tabsContainerRef" class="flex items-end h-full overflow-x-auto scrollbar-hide flex-1 mr-[140px] pr-4" style="scrollbar-width: none; -ms-overflow-style: none; scroll-behavior: smooth;">
          <!-- Tabs loop -->
            <div 
              v-for="(tab, index) in tabs" 
              :key="tab.id" 
              class="flex items-end h-full relative tab-item mr-1 shrink"
              :class="[{ 'tab-entering': tab._animating }, index === 0 ? 'ml-3' : '']"
              style="width: 232px; min-width: 56px;"
            >
              <div 
                class="tab-body relative flex items-center justify-between pl-3 pr-1.5 py-1 w-full h-[34px] text-[12px] rounded-t-[10px] group cursor-default" 
                :class="activeTabId === tab.id ? 'bg-[#3c3c3c] text-zinc-200 z-[5]' : 'hover:bg-[#282a2d] text-zinc-400 z-[1]'"
                @mousedown.stop="setActiveTab(tab.id)"
              >
                <!-- Inverted curves (съезды) at the bottom for active tab -->
                <template v-if="activeTabId === tab.id">
                  <div class="absolute -left-3 bottom-0 w-3 h-3 bg-transparent rounded-br-xl shadow-[6px_6px_0_4px_#3c3c3c] z-[6] pointer-events-none"></div>
                  <div class="absolute -right-3 bottom-0 w-3 h-3 bg-transparent rounded-bl-xl shadow-[-6px_6px_0_4px_#3c3c3c] z-[6] pointer-events-none"></div>
                </template>
                
                <div class="flex items-center space-x-2 truncate pl-0 pr-1.5 -translate-x-[4px] -translate-y-[3px] z-[7] relative overflow-hidden">
                  <div v-if="tab.isLoading" class="w-4 h-4 border-2 border-zinc-400 border-t-transparent rounded-full animate-spin shrink-0"></div>
                  
                  <svg v-else-if="tab.isNewTab" class="w-4 h-4 shrink-0 object-contain" viewBox="0 0 24 24" fill="#c7c7c7" :stroke="activeTabId === tab.id ? '#3c3c3c' : '#1f2020'" stroke-width="0.5" stroke-linejoin="round">
                    <path d="M12 0C8.21 0 4.831 1.757 2.632 4.501l3.953 6.848A5.454 5.454 0 0 1 12 6.545h10.691A12 12 0 0 0 12 0zM1.931 5.47A11.943 11.943 0 0 0 0 12c0 6.012 4.42 10.991 10.189 11.864l3.953-6.847a5.45 5.45 0 0 1-6.865-2.29zm13.342 2.166a5.446 5.446 0 0 1 1.45 7.09l.002.001h-.002l-5.344 9.257c.206.01.413.016.621.016 6.627 0 12-5.373 12-12 0-1.54-.29-3.011-.818-4.364zM12 16.364a4.364 4.364 0 1 1 0-8.728 4.364 4.364 0 0 1 0 8.728Z"/>
                  </svg>
                  <img v-else-if="getTabFavicon(tab)" :src="getTabFavicon(tab)" class="w-4 h-4 shrink-0 object-contain" @error="(e) => onFaviconError(tab)" />
                  <svg v-else class="w-4 h-4 text-zinc-500 shrink-0 object-contain" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="2" y1="12" x2="22" y2="12"></line>
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                  </svg>

                  <span class="truncate min-w-0 flex-1">{{ tab.title || 'Новая вкладка' }}</span>
                </div>
                <button 
                  v-if="tabs.length > 1"
                  @mousedown.stop="closeTab(tab.id)"
                  class="text-[#e8eaed] hover:bg-[#ffffff]/20 rounded-full w-[20px] h-[20px] flex items-center justify-center transition-colors z-[7] relative -translate-y-[3px] shrink-0 ml-1 opacity-0 group-hover:opacity-100"
                  :class="activeTabId === tab.id ? '!opacity-100' : ''"
                >
                  <svg class="w-[14px] h-[14px]" fill="none" stroke="currentColor" stroke-width="2.35" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"></path></svg>
                </button>
              </div>
              <!-- Separator between inactive tabs -->
              <div v-if="index < tabs.length - 1 && activeTabId !== tab.id && activeTabId !== tabs[index+1].id" class="h-[20px] w-px bg-[#4a4d51] mb-1.5 shrink-0 absolute right-0 bottom-0 z-[2]"></div>
            </div>
          
          <!-- New tab button -->
          <button @click="addTab" class="ml-[6px] -translate-y-[2px] text-[#e8eaed] hover:bg-[#545454] w-7 h-7 flex items-center justify-center rounded-full transition-colors self-end mb-1 shrink-0" @mousedown.stop>
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
              <path d="M12 5v14M5 12h14"/>
            </svg>
          </button>
        </div>

        <!-- Right scroll arrow -->
        <button 
          v-show="showScrollArrows" 
          @mousedown.stop 
          @click="scrollTabs(200)" 
          class="z-20 flex items-center justify-center w-7 h-full bg-gradient-to-l from-[#1f2020] via-[#1f2020] to-transparent pl-2 text-zinc-500 hover:text-zinc-200 shrink-0 self-stretch absolute right-[138px] top-0"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
        </button>
        
        <!-- Window Controls — always absolutely positioned, never overlapped -->
        <div class="flex items-center h-full absolute right-0 top-0 text-[#8e8e8e] z-30">
          <button @mousedown.stop @click="minimizeWindow" class="w-[46px] h-full flex items-center justify-center hover:bg-[#545454] hover:text-white transition-colors">
            <svg class="w-2.5 h-2.5" viewBox="0 0 10 10" fill="currentColor"><rect y="4" width="10" height="2"></rect></svg>
          </button>
          <button @mousedown.stop @click="maximizeWindow" class="w-[46px] h-full flex items-center justify-center hover:bg-[#545454] hover:text-white transition-colors">
            <svg class="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
              <rect x="1.5" y="3" width="5.5" height="5.5" rx="0.75" />
              <path d="M3.5 3V2a0.75 0.75 0 0 1 0.75-0.75h4a0.75 0.75 0 0 1 0.75 0.75v4a0.75 0.75 0 0 1-0.75 0.75H7" stroke-linejoin="round" />
            </svg>
          </button>
          <button @mousedown.stop @click="closeWindow" class="w-[46px] h-full flex items-center justify-center hover:bg-[#e81123] hover:text-white transition-colors">
            <svg class="w-2.5 h-2.5" viewBox="0 0 10 10" stroke="currentColor" stroke-width="2"><path d="M0 0 L10 10 M10 0 L0 10"></path></svg>
          </button>
        </div>
      </div>

      <!-- 2. Address Bar Row -->
      <div class="flex items-center bg-[#3c3c3c] pt-[6px] pb-[7px] pl-2 pr-1.5 border-b border-[#303236] relative">
        <div class="flex items-center shrink-0 translate-x-[1px] -translate-y-[1px]">
          <button @click="goBack" class="w-7 h-7 flex items-center justify-center text-[#e8eaed] hover:bg-[#545454] rounded-full transition-colors">
            <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><path d="M12 19l-7-7 7-7"/></svg>
          </button>
          <button @click="goForward" class="w-7 h-7 ml-[8px] flex items-center justify-center text-zinc-500 rounded-full transition-colors">
            <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5l7 7-7 7"/></svg>
          </button>
          <button @click="reloadTab" class="w-7 h-7 ml-[8px] translate-y-[1px] flex items-center justify-center text-[#e8eaed] hover:bg-[#545454] rounded-full transition-colors">
            <svg class="w-[13.5px] h-[13.5px]" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M21 3v6h-6" stroke-width="2.5" stroke-linejoin="miter" stroke-linecap="square"/>
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L21 9" stroke-width="2.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <form @submit.prevent="navigate" class="flex-1 flex items-center ml-[13px] mr-[13px]">
          <div class="relative w-full flex items-center bg-[#282828] border border-transparent focus-within:border-[#3b82f6] rounded-full h-[34px] pl-[5px] pr-3 transition-colors">
            <!-- Settings/Tune Icon inside the URL bar -->
            <div class="flex items-center justify-center bg-[#3c3c3c] hover:bg-[#4a4d51] rounded-full w-[24px] h-[24px] shrink-0 transition-colors -ml-[1px]">
              <button type="button" class="text-white flex items-center justify-center w-full h-full">
                <svg class="w-[13.5px] h-[13.5px]" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-linecap="round">
                  <circle cx="3.8" cy="4.5" r="1.9" stroke-width="1.6" />
                  <line x1="8.5" y1="4.5" x2="13.8" y2="4.5" stroke-width="1.5" />
                  <line x1="2.2" y1="12.2" x2="7.2" y2="12.2" stroke-width="1.5" />
                  <circle cx="12.2" cy="12.2" r="1.9" stroke-width="1.6" />
                </svg>
              </button>
            </div>
            <input 
              v-model="inputUrl" 
              type="text" 
              class="w-full bg-transparent border-none outline-none text-zinc-100 text-[14px] placeholder-zinc-500 leading-[34px] ml-2 pr-7 -translate-y-[0px]"
            />
            <button type="button" class="text-[#e8eaed] hover:bg-[#545454] p-1.5 rounded-full absolute right-[8px] transition-colors">
              <svg class="w-[18.5px] h-[18.5px]" viewBox="0 0 24 24" fill="currentColor">
                <path d="M22 9.24l-7.19-.62L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21 12 17.27 18.18 21l-1.63-7.03L22 9.24zM12 15.4l-3.76 2.27 1-4.28-3.32-2.88 4.38-.38L12 6.1l1.71 4.04 4.38.38-3.32 2.88 1 4.28L12 15.4z"/>
              </svg>
            </button>
          </div>
        </form>

        <div class="flex items-center space-x-0.5 shrink-0 relative">
          <!-- Profile -->
          <button class="w-8 h-8 flex items-center justify-center text-[#bdc1c6] hover:bg-[#545454] rounded-full transition-colors -translate-x-[3px] -translate-y-[0px]">
            <svg class="w-[19.5px] h-[19.5px]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="10.4" r="2.8" />
              <path d="M 5.9 18.7 C 7.2 15.2, 16.8 15.2, 18.1 18.7" />
              <circle cx="12" cy="12" r="9" />
            </svg>
          </button>
          <!-- Menu (Three dots) -->
          <button 
            ref="menuBtnRef"
            @click="toggleMenuWidget" 
            class="w-8 h-8 flex items-center justify-center text-[#e8eaed] rounded-full transition-colors" 
            :class="isMenuOpen ? 'bg-[#545454]' : 'hover:bg-[#545454]'"
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/></svg>
          </button>
        </div>

        <!-- Loading Progress Bar -->
        <div 
          v-if="authStore.transitionPhase !== 'idle'"
          class="absolute bottom-0 left-0 h-[2px] bg-[#3b82f6] transition-all duration-300 ease-out z-50"
          :style="{ width: progressWidth, opacity: authStore.transitionPhase === 'done' ? 0 : 1 }"
        ></div>
      </div>
    </div>
    
    <!-- Area for the stealth Webview -->
    <div class="flex-1 bg-[#202124] relative">
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAuthStore } from '../stores/auth.store';
import api from '../services/api';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { getCurrentWindow } from '@tauri-apps/api/window';

const updateAppTitle = (title) => {
  invoke('native_set_window_title', { title: `${title || 'Новая вкладка'} - Google Chrome` })
    .catch((e) => alert(`Error setting title: ${e}`));
};

const headerWrapper = ref(null);
const tabsContainerRef = ref(null);
let resizeObserver = null;
let tabsResizeObserver = null;
const showScrollArrows = ref(false);
const inputUrl = ref('google.com');

const progressWidth = computed(() => {
  switch (authStore.transitionPhase) {
    case 'authenticating': return '30%';
    case 'connecting': return '70%';
    case 'morphing': return '100%';
    case 'done': return '100%';
    default: return '0%';
  }
});

// Menu widget state
const isMenuOpen = ref(false);
const menuBtnRef = ref(null);
const isDownloadsOpen = ref(false);

// Favicon fallback
const failedFavicons = ref(new Set());

const getTabFavicon = (tab) => {
  if (tab.isNewTab) return null;
  if (failedFavicons.value.has(tab.id)) return null;
  // Prefer native favicon from WebView2
  if (tab.favicon) return tab.favicon;
  // Fallback: Google's favicon service
  if (tab.url && !tab.url.startsWith('about:') && !tab.url.startsWith('data:')) {
    try {
      const url = new URL(tab.url);
      return `https://www.google.com/s2/favicons?domain=${url.hostname}&sz=128`;
    } catch (e) {
      return null;
    }
  }
  return null;
};

const onFaviconError = (tab) => {
  if (tab && tab.id) failedFavicons.value.add(tab.id);
};

let lastMenuWidgetCloseTime = 0;
let lastDownloadsWidgetCloseTime = 0;

// Menu widget channel
const menuChannel = new BroadcastChannel('menu-widget-channel');
menuChannel.onmessage = (e) => {
  if (e.data.type === 'action') {
    handleMenuAction(e.data.action);
  } else if (e.data.type === 'widget-closed') {
    isMenuOpen.value = false;
    lastMenuWidgetCloseTime = Date.now();
  }
};

// Downloads widget channel (receive close events only — no button in FakeChromeView)
const downloadsChannel = new BroadcastChannel('downloads-channel');
downloadsChannel.onmessage = (e) => {
  if (e.data.type === 'widget-closed') {
    isDownloadsOpen.value = false;
    lastDownloadsWidgetCloseTime = Date.now();
  }
};

const handleMenuAction = (action) => {
  switch (action) {
    case 'new-tab':
      addTab();
      break;
    case 'downloads':
      toggleDownloadsWidget();
      break;
  }
};

const toggleMenuWidget = async () => {
  if (Date.now() - lastMenuWidgetCloseTime < 200) return;
  try {
    const widget = await WebviewWindow.getByLabel('menu-widget');
    if (widget) {
      const isVisible = await widget.isVisible();
      if (isVisible) {
        await widget.hide();
        isMenuOpen.value = false;
      } else {
        const mainWindow = getCurrentWindow();
        const pos = await mainWindow.outerPosition();
        const size = await mainWindow.outerSize();
        
        // Position at top-right near the 3 dots button
        await widget.setPosition(new PhysicalPosition(pos.x + size.width - 320 - 10, pos.y + 85));
        
        await widget.show();
        await widget.setFocus();
        isMenuOpen.value = true;
      }
    }
  } catch (err) {
    console.error('Ошибка переключения виджета меню:', err);
  }
};

const toggleDownloadsWidget = async () => {
  if (Date.now() - lastDownloadsWidgetCloseTime < 200) return;
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

const closeWidgetExternally = async (widgetLabel) => {
  try {
    const widget = await WebviewWindow.getByLabel(widgetLabel);
    if (widget) {
      const isVisible = await widget.isVisible();
      if (isVisible) {
        await widget.hide();
      }
    }
  } catch (e) {
    console.error(e);
  }
};

const closeAllWidgets = async () => {
  if (!isMenuOpen.value && !isDownloadsOpen.value) return; // Prevent IPC flood
  isMenuOpen.value = false;
  isDownloadsOpen.value = false;
  await closeWidgetExternally('menu-widget');
  await closeWidgetExternally('downloads-widget');
};

const checkScrollArrows = () => {
  if (tabsContainerRef.value) {
    showScrollArrows.value = tabsContainerRef.value.scrollWidth > tabsContainerRef.value.clientWidth;
  }
};
const scrollTabs = (delta) => {
  if (tabsContainerRef.value) {
    tabsContainerRef.value.scrollLeft += delta;
  }
};

const tabs = ref([{ id: 'fake_tab_1', title: 'Новая вкладка', url: 'https://google.com', isLoading: true, isNewTab: true, favicon: '', _animating: false }]);
const activeTabId = ref('fake_tab_1');


const formatDisplayUrl = (url) => {
  if (!url) return '';
  let displayUrl = url;
  if (displayUrl.startsWith('https://www.')) displayUrl = displayUrl.replace('https://www.', '');
  else if (displayUrl.startsWith('https://')) displayUrl = displayUrl.replace('https://', '');
  else if (displayUrl.startsWith('http://www.')) displayUrl = displayUrl.replace('http://www.', '');
  else if (displayUrl.startsWith('http://')) displayUrl = displayUrl.replace('http://', '');
  if (displayUrl.endsWith('/')) displayUrl = displayUrl.slice(0, -1);
  return displayUrl;
};

const addTab = () => {
  const newId = `fake_tab_${Date.now()}`;
  const newTab = { id: newId, title: 'Новая вкладка', url: 'https://google.com', isLoading: true, isNewTab: true, favicon: '', _animating: true };
  
  // Set active tab BEFORE pushing to avoid flicker: the new tab is already active when it appears
  activeTabId.value = newId;
  tabs.value.push(newTab);
  
  nextTick(() => {
    // Remove animation flag after animation completes
    setTimeout(() => {
      newTab._animating = false;
    }, 300);
    // Scroll new tab into view
    if (tabsContainerRef.value) {
      tabsContainerRef.value.scrollTo({ left: tabsContainerRef.value.scrollWidth, behavior: 'smooth' });
    }
    checkScrollArrows();
  });
};

const closeTab = async (id) => {
  if (tabs.value.length === 1) return; // don't close last tab
  const index = tabs.value.findIndex(t => t.id === id);
  if (index !== -1) {
    try { await invoke('native_close_tab', { tabId: id }); } catch (_) {}
    
    // If closing the active tab, switch first to prevent flicker
    if (activeTabId.value === id) {
      // Prefer next tab, fallback to previous
      const newActive = tabs.value[index + 1] || tabs.value[Math.max(0, index - 1)];
      if (newActive && newActive.id !== id) {
        activeTabId.value = newActive.id;
      }
    }
    
    tabs.value.splice(index, 1);
    nextTick(() => checkScrollArrows());
  }
};

const setActiveTab = (id) => {
  activeTabId.value = id;
};

const minimizeWindow = () => invoke('native_minimize_window').catch(() => {});
const maximizeWindow = () => invoke('native_maximize_window').catch(() => {});
const closeWindow = () => invoke('native_close_window').catch(() => {});
const startDragging = () => invoke('native_start_dragging').catch(() => {});

const authStore = useAuthStore();

const goBack = () => invoke('native_go_back', { tabId: activeTabId.value }).catch(() => {});
const goForward = () => invoke('native_go_forward', { tabId: activeTabId.value }).catch(() => {});
const reloadTab = () => invoke('native_reload', { tabId: activeTabId.value }).catch(() => {});

watch(() => activeTabId.value, async (newId, oldId) => {
  // Skip if same tab (prevents flicker on initial load and redundant switches)
  if (newId === oldId) return;
  
  const activeTab = tabs.value.find(t => t.id === newId);
  if (activeTab) {
    updateAppTitle(activeTab.title);
    inputUrl.value = activeTab.isNewTab ? '' : formatDisplayUrl(activeTab.url || '');
    
    // Background load new tabs to avoid empty background flash
    if (activeTab.isNewTab && activeTab.isLoading) {
      invoke('native_navigate_tab', { tabId: newId, url: activeTab.url }).catch(() => {
        activeTab.isLoading = false;
      });
      return; 
    }

    try {
      await invoke('native_switch_tab', { tabId: newId });
    } catch (e) {
      if (e === 'NOT_FOUND' && activeTab.url) {
        activeTab.isLoading = true;
        invoke('native_navigate_tab', { tabId: newId, url: activeTab.url }).catch(() => {
          activeTab.isLoading = false;
        });
      }
    }
  }
}, { immediate: true });

const navigate = async () => {
  let url = inputUrl.value.trim();
  
  if (url.startsWith('372://')) {
    const code = url.replace('372://', '');
    const currentTab = tabs.value.find(t => t.id === activeTabId.value);
    if (currentTab) {
      currentTab.isLoading = true;
      currentTab.title = 'Подключение...';
    }
    inputUrl.value = 'Установка защищенного соединения...';
    
    const success = await authStore.loginWithOtp(code);
    if (!success) {
      if (currentTab) {
        currentTab.isLoading = false;
        currentTab.title = 'Ошибка';
      }
      inputUrl.value = url;
    }
    return;
  }

  let searchUrl = url;
  if (!url.startsWith('http://') && !url.startsWith('https://') && !url.startsWith('data:')) {
    if (url.includes('.') && !url.includes(' ')) {
      searchUrl = `https://${url}`;
    } else {
      searchUrl = `https://www.google.com/search?q=${encodeURIComponent(url)}`;
    }
  } else if (url.startsWith('http://') || url.startsWith('https://')) {
    searchUrl = url;
  }
  
  const currentTab = tabs.value.find(t => t.id === activeTabId.value);
  if (currentTab) {
    currentTab.url = searchUrl;
    currentTab.isLoading = true;
    currentTab.isNewTab = false;
    // Clear failed favicons for this tab since URL changed
    failedFavicons.value.delete(currentTab.id);
  }
  
  inputUrl.value = formatDisplayUrl(searchUrl);
  invoke('native_navigate_tab', { tabId: activeTabId.value, url: searchUrl }).catch(() => {});
};

let unlistenListeners = [];

const handleMainWindowMousedown = (e) => {
  // If click is on menu button, let toggleMenuWidget handle it
  if (menuBtnRef.value && menuBtnRef.value.contains(e.target)) return;
  if (isMenuOpen.value || isDownloadsOpen.value) {
    closeAllWidgets();
  }
};

onMounted(async () => {
  try {
    if (!authStore.deviceId) {
      await authStore.initDeviceId();
    }
    const deviceId = authStore.deviceId || 'unknown';
    await api.post('/api/device/event', {
      device_id: deviceId,
      event_type: 'open',
      browser_type: 'normal'
    });
  } catch (err) {
    alert('Failed to log FakeChrome open: ' + err.message + ' ' + JSON.stringify(err.response?.data || {}));
    console.error('Failed to log device event:', err);
  }

  resizeObserver = new ResizeObserver((entries) => {
    for (let entry of entries) {
      if (entry.target === headerWrapper.value) {
        let height = entry.contentRect.height;
        invoke('native_update_top_bar_height', { height }).catch(() => {});
      }
    }
  });
  if (headerWrapper.value) {
    resizeObserver.observe(headerWrapper.value);
  }

  if (tabsContainerRef.value) {
    tabsResizeObserver = new ResizeObserver(() => {
      checkScrollArrows();
    });
    tabsResizeObserver.observe(tabsContainerRef.value);
  }

  // Close widgets on main window move/resize/blur
  const mainWindow = getCurrentWindow();
  mainWindow.onMoved(closeAllWidgets);
  mainWindow.onResized(closeAllWidgets);
  mainWindow.onFocusChanged(({ payload: focused }) => {
    closeAllWidgets();
  });

  window.addEventListener('mousedown', handleMainWindowMousedown);

  unlistenListeners.push(await listen('webview-loading', (event) => {
    const { tabId } = event.payload || {};
    const tab = tabs.value.find(t => t.id === tabId);
    if (tab) tab.isLoading = true;
  }));

  unlistenListeners.push(await listen('webview-loaded', (event) => {
    const p = event.payload || {};
    const tId = p.tabId || p.tab_id;
    const tab = tabs.value.find(t => t.id === tId);
    if (tab) {
      tab.isLoading = false;
      if (p.url && !p.url.startsWith('data:')) {
        tab.url = p.url;
        
        // Если это больше не главная страница Google (например, пошел поиск или другой сайт)
        const isGoogleHome = p.url.match(/^https?:\/\/(www\.)?google\.[a-z.]{2,6}\/?$/);
        if (tab.isNewTab && !isGoogleHome) {
          tab.isNewTab = false;
        }

        if (activeTabId.value === tId) {
          inputUrl.value = tab.isNewTab ? '' : formatDisplayUrl(p.url);
          // Only switch native tab once fully loaded to prevent flash
          invoke('native_switch_tab', { tabId: tId }).catch(() => {});
        }
      } else if (activeTabId.value === tId) {
         invoke('native_switch_tab', { tabId: tId }).catch(() => {});
      }
    }
  }));

  unlistenListeners.push(await listen('webview-title-changed', (event) => {
    const { tabId, tab_id, title } = event.payload || {};
    const tId = tabId || tab_id;
    const tab = tabs.value.find(t => t.id === tId);
    if (tab && title) {
      if (!tab.isNewTab) {
        tab.title = title.trim();
        if (activeTabId.value === tId) {
          updateAppTitle(tab.title);
        }
      }
    }
  }));

  unlistenListeners.push(await listen('webview-favicon-changed', (event) => {
    const { tabId, tab_id, favicon } = event.payload || {};
    const tId = tabId || tab_id;
    const tab = tabs.value.find(t => t.id === tId);
    if (tab && favicon) {
      tab.favicon = favicon;
      // Clear failed status since we got a real favicon
      failedFavicons.value.delete(tab.id);
    }
  }));

  unlistenListeners.push(await listen('webview-error', (event) => {
    const err = event.payload;
    console.warn(`[WebView2 Error]`, err);
    const currentTab = tabs.value.find(t => t.id === activeTabId.value);
    if (currentTab) currentTab.isLoading = false;
  }));

  unlistenListeners.push(await listen('open-new-tab', (event) => {
    const { url } = event.payload || {};
    if (!url) return;
    const newId = `fake_tab_${Date.now()}`;
    const newTab = { id: newId, title: 'Загрузка...', url: url, isLoading: true, isNewTab: false, favicon: '', _animating: true };
    activeTabId.value = newId;
    tabs.value.push(newTab);
  }));
});

onUnmounted(() => {
  if (resizeObserver && headerWrapper.value) resizeObserver.unobserve(headerWrapper.value);
  if (tabsResizeObserver) tabsResizeObserver.disconnect();
  unlistenListeners.forEach(u => u());
  window.removeEventListener('mousedown', handleMainWindowMousedown);
  tabs.value.forEach(t => invoke('native_close_tab', { tabId: t.id }).catch(() => {}));
  menuChannel.close();
  downloadsChannel.close();
});

// Watch tab count changes to update scroll arrows
watch(() => tabs.value.length, () => {
  nextTick(() => checkScrollArrows());
});
</script>

<style scoped>
/* Chrome-like tab entry animation */
@keyframes tab-slide-in {
  0% {
    max-width: 0;
    opacity: 0;
  }
  60% {
    opacity: 0.7;
  }
  100% {
    max-width: 300px;
    opacity: 1;
  }
}

.tab-entering {
  animation: tab-slide-in 0.25s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.tab-item {
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Active tab body needs higher z-index so curves don't get clipped by neighbor tabs */
.tab-body {
  position: relative;
}
</style>
