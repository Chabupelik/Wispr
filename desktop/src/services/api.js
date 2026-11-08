import axios from 'axios';
import { useAuthStore } from '../stores/auth.store';
import { encryptPayload, decryptPayload } from './crypto';

const API_BASE = import.meta.env.VITE_API_BASE;
const DOMAINS = ['stream', 'cdn', 'media', 'edge', 'assets'];
const getDynamicApiBase = () => {
    const randomSub = DOMAINS[Math.floor(Math.random() * DOMAINS.length)];
    const url = new URL(API_BASE);
    url.hostname = `${randomSub}.${url.hostname}`;
    return url.toString().replace(/\/$/, '');
};

const api = axios.create({
  baseURL: getDynamicApiBase(), // Запросы пойдут на случайный поддомен
  withCredentials: true, // ВАЖНО: разрешает отправку HttpOnly куки с Refresh токеном
  responseType: 'blob' // Для поддержки расшифровки бинарных данных, если ответ зашифрован. Мы перехватим и распарсим.
});

let isRefreshing = false;
let failedQueue = [];

const processQueue = (error, token = null) => {
  failedQueue.forEach(prom => {
    if (error) prom.reject(error);
    else prom.resolve(token);
  });
  failedQueue = [];
};

// Перехватчик Запросов (Шифрование)
api.interceptors.request.use(async (config) => {
  if (['post', 'put', 'patch'].includes(config.method) && config.data && typeof config.data === 'object') {
    try {
      const encryptedData = await encryptPayload(config.data);
      config.data = encryptedData;
      config.headers['X-Encrypted-Payload'] = '1';
      config.headers['Content-Type'] = 'application/octet-stream';
    } catch (e) {
      console.error('Ошибка шифрования payload:', e);
    }
  }
  return config;
});

// Перехватчик Ответов
api.interceptors.response.use(
  async (response) => {
    if (response.data instanceof Blob) {
      if (response.headers['x-encrypted-payload'] === '1') {
        try {
          const buffer = await response.data.arrayBuffer();
          response.data = await decryptPayload(buffer);
        } catch (e) {
          console.error('Ошибка расшифровки payload:', e);
        }
      } else if (response.data.type.includes('json') || response.data.size > 0) {
        try {
          const text = await response.data.text();
          response.data = JSON.parse(text);
        } catch (e) {
          // If it's not JSON, maybe leave it as text
          try { response.data = await response.data.text(); } catch(err){}
        }
      }
    }
    return response;
  },
  async (error) => {
    const originalRequest = error.config;

    // Если ошибка 401 и мы еще не пробовали обновить токен
    if (error.response?.status === 401 && !originalRequest._retry) {
      if (isRefreshing) {
        // Если уже обновляем, ставим запрос в очередь
        return new Promise(function(resolve, reject) {
          failedQueue.push({ resolve, reject });
        }).then(token => {
          originalRequest.headers['Authorization'] = 'Bearer ' + token;
          return api(originalRequest);
        }).catch(err => Promise.reject(err));
      }

      originalRequest._retry = true;
      isRefreshing = true;

      try {
        const authStore = useAuthStore(); // Вызываем тут, чтобы избежать ошибки инициализации Pinia
        
        // Дергаем эндпоинт рефреша (кука улетит автоматически)
        const { data } = await axios.post(`${getDynamicApiBase()}/api/auth/refresh`, { device_id: authStore.deviceId }, { withCredentials: true });
        
        authStore.accessToken = data.accessToken;
        api.defaults.headers.common['Authorization'] = 'Bearer ' + data.accessToken;
        originalRequest.headers['Authorization'] = 'Bearer ' + data.accessToken;
        
        processQueue(null, data.accessToken);
        return api(originalRequest); // Повторяем изначальный запрос
      } catch (refreshError) {
        processQueue(refreshError, null);
        const authStore = useAuthStore();
        authStore.logout(); // Если рефреш сдох — выкидываем на страницу логина
        return Promise.reject(refreshError);
      } finally {
        isRefreshing = false;
      }
    }

    return Promise.reject(error);
  }
);

export default api;