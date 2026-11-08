// frontend/src/stores/browser.store.js
import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useBrowserStore = defineStore('browser', () => {
  let tabIdCounter = 0;
  const generateTabId = () => `${Date.now()}_${tabIdCounter++}`;

  const loadSavedTabs = () => {
    return [{ id: generateTabId(), url: '', title: 'Новая вкладка', isLoading: false }];
  };

  const tabs = ref(loadSavedTabs());
  const activeTabId = ref(tabs.value[0].id);

  const addTab = (url = '') => {
    const newTab = { id: generateTabId(), url, title: 'Новая вкладка', isLoading: !!(url && url !== 'about:blank') };
    tabs.value.push(newTab);
    activeTabId.value = newTab.id;
  };

  const closeTab = (id) => {
    const index = tabs.value.findIndex(t => t.id === id);
    if (index === -1) return;

    tabs.value.splice(index, 1);

    if (tabs.value.length === 0) {
      addTab('');
    } else if (activeTabId.value === id) {
      activeTabId.value = tabs.value[Math.max(0, index - 1)].id;
    }
  };

  const setActiveTab = (id) => {
    activeTabId.value = id;
  };

  // Сбрасывает вкладки в памяти при выходе из аккаунта.
  const clearUserSession = async () => {
    tabs.value = [{ id: generateTabId(), url: '', title: 'Новая вкладка', isLoading: false }];
    activeTabId.value = tabs.value[0].id;
  };

  const restoreTabs = (savedTabs) => {
    if (!Array.isArray(savedTabs) || savedTabs.length === 0) return;
    tabs.value = savedTabs.map(t => ({
      id: generateTabId(),
      url: t.url || '',
      title: t.title || 'Новая вкладка',
      isLoading: false,
    }));
    activeTabId.value = tabs.value[0].id;
  };

  return { tabs, activeTabId, addTab, closeTab, setActiveTab, clearUserSession, restoreTabs };
});