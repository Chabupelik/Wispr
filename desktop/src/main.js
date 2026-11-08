// frontend/src/main.js
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './assets/index.css';

const app = createApp(App);
const pinia = createPinia();

app.config.errorHandler = (err, vm, info) => { alert('Vue Error: ' + err.toString() + ' | ' + info); console.error(err); }; window.addEventListener('error', e => alert('Window Error: ' + e.message)); app.use(pinia);
app.mount('#app');
