<template>
  <div class="w-full h-full bg-[#282a2d]/95 backdrop-blur-2xl flex flex-col font-sans select-none overflow-hidden rounded-lg border border-[#3b3e41] shadow-2xl text-zinc-300">
    <!-- Header -->
    <div class="flex items-center justify-between px-3.5 py-3 border-b border-[#3b3e41] bg-[#2c2e31]/90" @mousedown="startDragging">
      <div class="flex items-center space-x-2 pointer-events-none">
        <div class="w-6 h-6 rounded-lg bg-[#3b3e41] flex items-center justify-center text-zinc-400">
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </div>
        <span class="font-medium text-xs text-zinc-200">Загрузки</span>
        <span 
          v-if="downloads.length > 0"
          class="text-[10px] font-bold px-1.5 py-0.5 rounded-full bg-[#3b3e41] text-zinc-400"
        >
          {{ downloads.length }}
        </span>
      </div>

      <div class="flex items-center space-x-1.5">
        <button 
          v-if="downloads.length > 0"
          @click="clearDownloads" 
          class="flex items-center space-x-1 text-[11px] text-zinc-400 hover:text-red-400 hover:bg-red-500/10 px-2 py-1 rounded-lg transition-all"
          title="Очистить список"
        >
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          <span>Очистить</span>
        </button>
        <button 
          @click="closeWidget" 
          class="w-6 h-6 flex items-center justify-center text-zinc-400 hover:text-zinc-100 hover:bg-[#545454] rounded-lg transition-colors"
          title="Закрыть"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Список файлов -->
    <div class="flex-1 overflow-y-auto space-y-1 p-2 bg-[#282a2d]/50 custom-scrollbar">
      <!-- Empty State -->
      <div v-if="downloads.length === 0" class="h-full min-h-[260px] flex flex-col items-center justify-center text-center p-6 space-y-3">
        <div class="w-12 h-12 rounded-2xl bg-[#1f2020] border border-[#3b3e41] flex items-center justify-center text-zinc-500 shadow-inner">
          <svg class="w-6 h-6 stroke-[1.5]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
          </svg>
        </div>
        <div>
          <p class="text-xs font-medium text-zinc-300">Нет загрузок</p>
          <p class="text-[11px] text-zinc-500 mt-0.5 max-w-[200px]">Скачанные файлы появятся здесь</p>
        </div>
      </div>

      <!-- File item card -->
      <div 
        v-for="item in downloads" 
        :key="item.id"
        class="p-2.5 bg-[#323437] hover:bg-[#3b3e41] border border-[#3b3e41] hover:border-[#4a4d51] rounded-lg transition-all duration-150 flex items-center justify-between group"
      >
        <div class="flex items-center space-x-2.5 min-w-0 pr-2">
          <!-- File Category Icon -->
          <div 
            class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 border"
            :class="getFileTheme(item.filename)"
          >
            <!-- Executable / App -->
            <svg v-if="getFileType(item.filename) === 'app'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
            </svg>
            <!-- Archive -->
            <svg v-else-if="getFileType(item.filename) === 'archive'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
            </svg>
            <!-- Media / Video / Audio -->
            <svg v-else-if="getFileType(item.filename) === 'media'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <!-- Image -->
            <svg v-else-if="getFileType(item.filename) === 'image'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            <!-- Document / PDF -->
            <svg v-else-if="getFileType(item.filename) === 'doc'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <!-- Generic File -->
            <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
            </svg>
          </div>

          <div class="min-w-0">
            <p class="font-medium text-zinc-200 text-xs truncate max-w-[170px]" :title="item.filename">
              {{ item.filename }}
            </p>
            <div class="flex items-center space-x-1.5 text-[10px] text-zinc-500 mt-0.5">
              <span>{{ item.time }}</span>
              <span>•</span>
              <span class="text-[#8ab4f8] font-medium">{{ item.size || 'Завершено' }}</span>
            </div>
          </div>
        </div>

        <!-- Action buttons -->
        <div class="flex items-center space-x-1 shrink-0">
          <button 
            @click="openDownloadedFile(item.path)" 
            class="flex items-center space-x-1 px-2.5 py-1 bg-[#3b3e41] hover:bg-[#8ab4f8] hover:text-[#202124] text-zinc-300 rounded-lg text-[11px] font-medium transition-all duration-150"
            title="Открыть файл"
          >
            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            </svg>
            <span>Открыть</span>
          </button>
          <button 
            @click="showDownloadedFileInFolder(item.path)" 
            class="p-1.5 bg-[#3b3e41]/60 hover:bg-[#4a4d51] text-zinc-400 hover:text-zinc-100 rounded-lg transition-colors"
            title="Показать в папке"
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const startDragging = () => invoke('native_start_dragging').catch(() => {});

const downloads = ref([]);
const appWindow = getCurrentWindow();
let channel = null;
let unlistenBlur = null;

onMounted(async () => {
  channel = new BroadcastChannel('downloads-channel');
  channel.onmessage = (e) => {
    if (e.data.type === 'sync') {
      downloads.value = e.data.data || [];
    } else if (e.data.type === 'close-widget') {
      closeWidget();
    }
  };

  // Запрашиваем актуальные данные при старте
  channel.postMessage({ type: 'request-sync' });
  
  // Закрытие при потере фокуса (клик вне виджета)
  window.addEventListener('blur', handleBlur);
  window.addEventListener('keydown', handleKeydown);

  try {
    appWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) closeWidget();
    });
  } catch (err) {
    console.error('Ошибка подписки на focus:', err);
  }
});

onUnmounted(() => {
  if (channel) {
    channel.close();
  }
  window.removeEventListener('blur', handleBlur);
  window.removeEventListener('keydown', handleKeydown);
});

const handleBlur = () => {
  closeWidget();
};

const handleKeydown = (e) => {
  if (e.key === 'Escape') {
    closeWidget();
  }
};

const closeWidget = () => {
  appWindow.hide();
  if (channel) {
    channel.postMessage({ type: 'widget-closed' });
  }
};

const clearDownloads = () => {
  downloads.value = [];
  if (channel) {
    channel.postMessage({ type: 'clear' });
  }
};

const getFileType = (filename) => {
  const ext = filename?.split('.').pop()?.toLowerCase();
  if (['exe', 'msi', 'bat', 'cmd'].includes(ext)) return 'app';
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return 'archive';
  if (['mp4', 'mkv', 'avi', 'mov', 'webm', 'mp3', 'wav', 'flac'].includes(ext)) return 'media';
  if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'ico'].includes(ext)) return 'image';
  if (['pdf', 'doc', 'docx', 'txt', 'md', 'json', 'csv', 'xlsx'].includes(ext)) return 'doc';
  return 'file';
};

const getFileTheme = (filename) => {
  const type = getFileType(filename);
  switch (type) {
    case 'app':
      return 'bg-[#394b3e] border-[#4a6e52] text-[#81c995]';
    case 'archive':
      return 'bg-[#4b4430] border-[#6e6340] text-[#fdd663]';
    case 'media':
      return 'bg-[#3d3552] border-[#5c4d7a] text-[#c58af9]';
    case 'image':
      return 'bg-[#2e3e50] border-[#3e5a7a] text-[#8ab4f8]';
    case 'doc':
      return 'bg-[#2e4450] border-[#3e647a] text-[#78d9ec]';
    default:
      return 'bg-[#3b3e41] border-[#4a4d51] text-zinc-400';
  }
};

const openDownloadedFile = async (path) => {
  try {
    await invoke('native_open_path', { path });
  } catch (e) {
    console.error('Ошибка открытия файла:', e);
  }
};

const showDownloadedFileInFolder = async (path) => {
  try {
    await invoke('native_show_in_folder', { path });
  } catch (e) {
    console.error('Ошибка показа файла в папке:', e);
  }
};
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 5px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #3b3e41;
  border-radius: 9999px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #4a4d51;
}
</style>
