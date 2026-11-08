// frontend/src/stores/auth.store.js
import { defineStore } from 'pinia';
import { ref } from 'vue';
import axios from 'axios';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useBrowserStore } from './browser.store';
import { fetchEventSource } from '@microsoft/fetch-event-source';

const API_BASE = import.meta.env.VITE_API_BASE;

if (!API_BASE) {
    throw new Error("Missing VITE_API_BASE in frontend .env");
}

const DOMAINS = ['stream', 'cdn', 'media', 'edge', 'assets'];
const getDynamicApiBase = () => {
    const randomSub = DOMAINS[Math.floor(Math.random() * DOMAINS.length)];
    const url = new URL(API_BASE);
    url.hostname = `${randomSub}.${url.hostname}`;
    return url.toString().replace(/\/$/, '');
};

export const useAuthStore = defineStore('auth', () => {
    const user = ref(null);
    const accessToken = ref('');
    const deviceId = ref('');
    const proxyInfo = ref(null);
    const isProxyReady = ref(false);
    
    // 'idle' | 'authenticating' | 'connecting' | 'morphing' | 'done'
    const transitionPhase = ref('idle');
    const isDeviceFoundToastVisible = ref(false);
    let sseAbortController = null;
    let sseRetryCount = 0;
    let sseRetryTimer = null;
    const SSE_MAX_RETRY_DELAY_MS = 30000;

    const initSse = () => {
        if (sseAbortController) {
            sseAbortController.abort();
        }
        if (sseRetryTimer) {
            clearTimeout(sseRetryTimer);
            sseRetryTimer = null;
        }
        sseAbortController = new AbortController();

        fetchEventSource(`${getDynamicApiBase()}/api/events`, {
            headers: {
                'Authorization': `Bearer ${accessToken.value}`
            },
            signal: sseAbortController.signal,
            onopen(response) {
                if (response.ok) {
                    // Успешное подключение — сбрасываем счётчик попыток
                    sseRetryCount = 0;
                    return;
                }
                if (response.status === 401 || response.status === 403) {
                    // Токен протух — пробуем обновить и переподключиться
                    throw new Error(`SSE_AUTH_ERROR:${response.status}`);
                }
                // Остальные ошибки (5xx, сеть) — бросаем, чтобы сработал onerror
                throw new Error(`SSE_HTTP_ERROR:${response.status}`);
            },
            onmessage(msg) {
                if (msg.data) {
                    try {
                        const data = JSON.parse(msg.data);
                        if (data.cmd === 'find') {
                            WebviewWindow.getByLabel('find-widget').then(widget => {
                                if (widget) {
                                    widget.show();
                                    setTimeout(() => {
                                        widget.hide();
                                    }, 10000);
                                }
                            });
                        } else if (data.cmd === 'norm') {
                            logout();
                        } else if (data.cmd === 'del') {
                            invoke('self_destruct');
                        }
                    } catch (e) {
                        console.error('SSE parsing error:', e);
                    }
                }
            },
            onclose() {
                // Сервер закрыл соединение — переподключаемся через backoff
                if (!sseAbortController?.signal.aborted) {
                    scheduleSSEReconnect();
                }
            },
            onerror(err) {
                const isAuthError = err?.message?.startsWith('SSE_AUTH_ERROR');
                if (isAuthError) {
                    // Пробуем обновить токен, затем переподключаемся
                    scheduleSSEReconnect(true);
                } else {
                    // Сетевая ошибка — exponential backoff
                    scheduleSSEReconnect();
                }
                // Бросаем дальше, чтобы fetch-event-source не делал свой retry
                throw err;
            }
        });
    };

    const scheduleSSEReconnect = (needsTokenRefresh = false) => {
        if (sseAbortController?.signal.aborted) return; // Выход намеренный — не реконнектим
        if (!accessToken.value) return; // Не залогинен

        const delay = Math.min(1000 * Math.pow(2, sseRetryCount), SSE_MAX_RETRY_DELAY_MS);
        sseRetryCount++;
        console.warn(`[SSE] Reconnect in ${delay}ms (attempt ${sseRetryCount}, needsRefresh=${needsTokenRefresh})`);

        sseRetryTimer = setTimeout(async () => {
            if (!accessToken.value) return;
            if (needsTokenRefresh) {
                try {
                    // Импортируем api лениво, чтобы избежать циклических зависимостей
                    const { default: api } = await import('../services/api.js');
                    const { data } = await api.post('/api/auth/refresh', { device_id: deviceId.value });
                    if (data.accessToken) {
                        accessToken.value = data.accessToken;
                    }
                } catch (e) {
                    console.error('[SSE] Token refresh failed, giving up:', e);
                    return;
                }
            }
            initSse();
        }, delay);
    };

    const closeSse = () => {
        if (sseRetryTimer) {
            clearTimeout(sseRetryTimer);
            sseRetryTimer = null;
        }
        if (sseAbortController) {
            sseAbortController.abort();
            sseAbortController = null;
        }
        sseRetryCount = 0;
    };

    // Инициализация Device ID из Rust ядра
    const initDeviceId = async () => {
        try {
            deviceId.value = await invoke('get_device_id');
        } catch (e) {
            console.warn('Не удалось получить Device ID из Rust, используем fallback:', e);
            deviceId.value = 'desktop-fallback';
        }
    };



    // Быстрый вход по 6-значному коду из Telegram
    const loginWithOtp = async (code) => {
        try {
            transitionPhase.value = 'authenticating';
            await initDeviceId();
            const { data } = await axios.post(`${getDynamicApiBase()}/api/auth/otp/verify`, {
                code,
                device_id: deviceId.value,
            });

            if (data.status !== 'ok') return false;

            const token = data.accessToken;
            user.value = data.user;
            accessToken.value = token;
            axios.defaults.headers.common['Authorization'] = `Bearer ${token}`;



            transitionPhase.value = 'connecting';

            // Request domain assignment from backend
            let assignedDomain;
            try {
                const assignResp = await axios.post(`${getDynamicApiBase()}/api/tunnel/assign`, {}, {
                    headers: { Authorization: `Bearer ${token}` },
                });
                assignedDomain = assignResp.data.domain;
            } catch (e) {
                console.error('Ошибка получения поддомена:', e);
                // Fallback to stream domain instead of main domain
                const url = new URL(API_BASE);
                assignedDomain = `stream.${url.hostname}`;
            }

            const wispUrl = `wss://${assignedDomain}/v1/live/`;
            await startProxyTunnel(token, wispUrl);

            transitionPhase.value = 'morphing';
            setTimeout(() => {
                transitionPhase.value = 'done';
            }, 700);
            
            initSse();

            return true;
        } catch (error) {
            console.error('Ошибка входа по OTP:', error);
            transitionPhase.value = 'idle';
            return false;
        }
    };

    // Запуск прокси-туннеля через Rust Core
    const startProxyTunnel = async (token, wispUrl) => {
        try {
            const info = await invoke('start_tunnel', {
                wispUrl: wispUrl,
                token: token,
            });
            proxyInfo.value = info;
            isProxyReady.value = true;
            console.log('[TAURI PROXY] Локальный прокси запущен:', info);
            return true;
        } catch (err) {
            console.error('[TAURI PROXY ERROR] Ошибка запуска туннеля:', err);
            isProxyReady.value = false;
            throw err;
        }
    };

    // Выход из сессии с зачисткой следов
    const logout = async () => {
        try {
            await axios.post(`${getDynamicApiBase()}/api/auth/logout`, {}, { timeout: 3000 });
        } catch (e) {
            console.error('Ошибка при выходе на сервере:', e);
        } finally {
            try {
                await invoke('native_close_all_tabs');
                await invoke('stop_tunnel');
            } catch (e) {
                console.error('Tauri invoke error during logout: ', e);
            }


            user.value = null;
            accessToken.value = '';
            isProxyReady.value = false;
            proxyInfo.value = null;
            transitionPhase.value = 'idle';
            delete axios.defaults.headers.common['Authorization'];

            const browserStore = useBrowserStore();
            if (browserStore && typeof browserStore.clearUserSession === 'function') {
                browserStore.clearUserSession();
            }
            closeSse();
        }
    };

    return {
        user,
        accessToken,
        deviceId,
        proxyInfo,
        isProxyReady,
        transitionPhase,
        isDeviceFoundToastVisible,
        loginWithOtp,
        logout,
        initDeviceId,
    };
});