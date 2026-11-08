# telegram_bot/bot.py
import os
import asyncio
import aiohttp
import ipaddress
from dotenv import load_dotenv
from aiogram import Bot, Dispatcher, types, BaseMiddleware, F
from aiogram.filters import Command
from aiogram.client.default import DefaultBotProperties
from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton, CallbackQuery
from aiogram.fsm.context import FSMContext
from aiogram.fsm.state import State, StatesGroup
from aiogram.fsm.storage.memory import MemoryStorage
from aiogram.exceptions import TelegramBadRequest

load_dotenv()

BOT_TOKEN = os.getenv("BOT_TOKEN")
API_BASE = os.getenv("API_URL", "http://backend:8080").replace("/api/auth/register", "").rstrip("/")
API_SECRET = os.getenv("API_SECRET") or os.getenv("BOT_TOKEN")
ADMIN_ID = str(os.getenv("ADMIN_ID"))

bot = Bot(token=BOT_TOKEN, default=DefaultBotProperties(parse_mode="Markdown"))
dp = Dispatcher(storage=MemoryStorage())

# Локальный кэш белых списков
cached_ips = []
cached_tg_ids = {}

async def fetch_whitelists():
    global cached_ips, cached_tg_ids
    headers = {"x-bot-token": API_SECRET}
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{API_BASE}/api/admin/whitelists", headers=headers, timeout=10) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    cached_ips = data.get("ips", [])
                    cached_tg_ids = {str(item["id"]): item.get("name") for item in data.get("tg_ids", [])}
                    print("[BOT] Whitelists loaded from backend:", len(cached_ips), "IPs,", len(cached_tg_ids), "TGs")
    except Exception as e:
        print("[BOT] Failed to load whitelists on startup:", e)

class WhitelistMiddleware(BaseMiddleware):
    async def __call__(self, handler, event: types.Message, data: dict):
        if not event or not getattr(event, 'from_user', None):
            return await handler(event, data)
            
        user_id = str(event.from_user.id)
        
        if "all" not in cached_tg_ids:
            if user_id not in cached_tg_ids and user_id != ADMIN_ID:
                print(f"[FIREWALL BOT] Blocked message from unapproved ID: {user_id}")
                try:
                    await event.answer("❌ У вас нет доступа к этому боту.")
                except Exception:
                    pass
                return # Игнорируем
                
        return await handler(event, data)

dp.message.middleware(WhitelistMiddleware())

def get_username(message: types.Message):
    return message.from_user.username or f"tg_{message.from_user.id}"

# ----------------- ADMIN FSM -----------------
class AdminStates(StatesGroup):
    waiting_for_ip = State()
    waiting_for_tg_id = State()
    waiting_for_name = State()
    waiting_for_user_new_name = State()
    waiting_for_devlogs_user = State()
    waiting_for_wisplogs_user = State()
    waiting_for_wispstats_user = State()
    waiting_for_wispstats_period_custom = State()
    waiting_for_hwid_name = State()
    waiting_for_global_hwid = State()
    waiting_for_global_hwid_name = State()

def get_admin_keyboard():
    markup = InlineKeyboardMarkup(inline_keyboard=[
        [InlineKeyboardButton(text="🧑‍💻 Зарегистрированные юзеры", callback_data="admin_dbusers")],
        [InlineKeyboardButton(text="📜 Логи устройств", callback_data="admin_devlogs")],
        [InlineKeyboardButton(text="🌐 Логи Wisp", callback_data="admin_wisplogs")],
        [InlineKeyboardButton(text="📊 Статистика Wisp", callback_data="admin_wispstats")],
        [InlineKeyboardButton(text="🌐 Управление IP", callback_data="admin_ip")],
        [InlineKeyboardButton(text="👥 Управление TG ID", callback_data="admin_tg")],
        [InlineKeyboardButton(text="💻 Назвать ПК", callback_data="admin_name_pc")],
        [InlineKeyboardButton(text="🔄 Обновить кэш с сервера", callback_data="admin_refresh")]
    ])
    return markup

def get_list_keyboard(type_name):
    markup = InlineKeyboardMarkup(inline_keyboard=[
        [InlineKeyboardButton(text="➕ Добавить", callback_data=f"add_{type_name}")],
        [InlineKeyboardButton(text="➖ Удалить", callback_data=f"del_{type_name}")],
        [InlineKeyboardButton(text="Включить/Выключить ALL", callback_data=f"toggle_{type_name}_all")],
        [InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]
    ])
    return markup

@dp.message(Command("cancel"))
async def cancel_handler(message: types.Message, state: FSMContext):
    current_state = await state.get_state()
    if current_state is None:
        return
    await state.clear()
    sent = await message.answer("✅ Действие отменено.")
    asyncio.create_task(delete_messages_later(message, sent, 30))

@dp.message(Command("admin"))
async def admin_panel(message: types.Message, state: FSMContext):
    if str(message.from_user.id) != ADMIN_ID:
        return
    await state.clear()
    sent = await message.answer("🛠 *Панель администратора*\n\nВыберите раздел для управления белыми списками:", reply_markup=get_admin_keyboard())
    asyncio.create_task(delete_messages_later(message, sent, 300))

async def render_admin_list(message: types.Message, type_name: str):
    if type_name == "ip":
        status = "🟢 Фильтр ОТКЛЮЧЕН (ALL)" if "all" in cached_ips else "🔴 Фильтр ВКЛЮЧЕН"
        text = f"🌐 *Управление IP*\n\n{status}\n\n*Текущий список:*\n`" + "`\n`".join(cached_ips) + "`"
        if not cached_ips: text += "Пусто"
        await message.edit_text(text, reply_markup=get_list_keyboard("ip"))
    elif type_name == "tg":
        status = "🟢 Фильтр ОТКЛЮЧЕН (ALL)" if "all" in cached_tg_ids else "🔴 Фильтр ВКЛЮЧЕН"
        text = f"👥 *Управление TG ID*\n\n{status}\n\n*Текущий список:*\n"
        if not cached_tg_ids: 
            text += "Пусто"
        else:
            for tid, tname in cached_tg_ids.items():
                text += f"`{tid}` - {tname or 'Без имени'}\n"
        await message.edit_text(text, reply_markup=get_list_keyboard("tg"))

@dp.callback_query(F.data.startswith("admin_"))
async def admin_callbacks(call: CallbackQuery, state: FSMContext):
    if str(call.from_user.id) != ADMIN_ID:
        return await call.answer("Доступ запрещен", show_alert=True)
        
    action = call.data.replace("admin_", "")
    
    if action == "back":
        await call.message.edit_text("🛠 *Панель администратора*\n\nВыберите раздел для управления белыми списками:", reply_markup=get_admin_keyboard())
    elif action == "refresh":
        await fetch_whitelists()
        await call.answer("Кэш обновлен!", show_alert=True)
    elif action == "name_pc":
        await state.set_state(AdminStates.waiting_for_global_hwid)
        sent = await call.message.answer("💻 Введите HWID компьютера, которому хотите назначить имя:\n\n(Для отмены введите /cancel)")
        asyncio.create_task(delete_messages_later(sent, delay=300))
        await call.answer()
    elif action in ["ip", "tg"]:
        await render_admin_list(call.message, action)
    elif action == "dbusers":
        await render_dbusers_page(call.message, 0)
        await call.answer()
    elif action == "devlogs":
        await state.update_data(devlogs_filter_user=None)
        await render_devlogs_page(call.message, 0, state)
        await call.answer()
    elif action.startswith("devlogsPage:"):
        page = int(action.split(":")[1])
        await render_devlogs_page(call.message, page, state)
        await call.answer()
    elif action == "devlogs_clear":
        headers = {"x-bot-token": API_SECRET}
        async with aiohttp.ClientSession() as session:
            async with session.delete(f"{API_BASE}/api/admin/device_events", headers=headers) as del_resp:
                del_status = del_resp.status
                del_text = await del_resp.text()
                print(f"DEBUG: DELETE /api/admin/device_events returned {del_status} - {del_text}")
        await call.answer("Логи очищены!", show_alert=True)
        await render_devlogs_page(call.message, 0, state)
    elif action == "devlogs_filter":
        await state.set_state(AdminStates.waiting_for_devlogs_user)
        sent = await call.message.answer("🔍 Введите логин пользователя для фильтрации логов:\n\n(Для отмены введите /cancel)")
        asyncio.create_task(delete_messages_later(sent, delay=300))
        await call.answer()
    elif action == "devlogs_clearfilter":
        await state.update_data(devlogs_filter_user=None)
        await call.answer("Фильтр сброшен!", show_alert=True)
        await render_devlogs_page(call.message, 0, state)
    elif action == "wisplogs":
        await state.update_data(wisplogs_filter_user=None)
        await render_wisplogs_page(call.message, 0, state)
        await call.answer()
    elif action.startswith("wisplogsPage:"):
        page = int(action.split(":")[1])
        await render_wisplogs_page(call.message, page, state)
        await call.answer()
    elif action == "wisplogs_clear":
        headers = {"x-bot-token": API_SECRET}
        async with aiohttp.ClientSession() as session:
            async with session.delete(f"{API_BASE}/api/admin/wisp_logs", headers=headers) as del_resp:
                del_status = del_resp.status
                del_text = await del_resp.text()
                print(f"DEBUG: DELETE /api/admin/wisp_logs returned {del_status} - {del_text}")
        await call.answer("Логи очищены!", show_alert=True)
        await render_wisplogs_page(call.message, 0, state)
    elif action == "wisplogs_filter":
        await state.set_state(AdminStates.waiting_for_wisplogs_user)
        sent = await call.message.answer("🔍 Введите логин, TG ID или домен для фильтрации логов:\n\n(Для отмены введите /cancel)")
        asyncio.create_task(delete_messages_later(sent, delay=300))
        await call.answer()
    elif action == "wisplogs_clearfilter":
        await state.update_data(wisplogs_filter_user=None)
        await call.answer("Фильтр сброшен!", show_alert=True)
        await render_wisplogs_page(call.message, 0, state)
    elif action == "wispstats":
        await state.set_state(AdminStates.waiting_for_wispstats_user)
        sent = await call.message.answer("📊 Введите логин или TG ID пользователя для сбора статистики (или отправьте `все` для общей статистики):\n\n(Для отмены введите /cancel)")
        asyncio.create_task(delete_messages_later(sent, delay=300))
        await call.answer()
    elif action.startswith("wispstats_period:"):
        period = action.split(":")[1]
        data = await state.get_data()
        
        if period == "custom":
            await state.set_state(AdminStates.waiting_for_wispstats_period_custom)
            sent = await call.message.answer("📅 Введите период в формате:\n`ДД.ММ.ГГГГ ЧЧ:ММ - ДД.ММ.ГГГГ ЧЧ:ММ`\n\nПример:\n`25.08.2026 18:00 - 25.08.2026 19:00`\n\n(Для отмены введите /cancel)", parse_mode="Markdown")
            asyncio.create_task(delete_messages_later(sent, delay=300))
            await call.answer()
            return

        from datetime import datetime, timedelta, timezone
        now_utc = datetime.now(timezone.utc)
        now_msk = now_utc + timedelta(hours=3)
        
        start_date = None
        end_date = None
        
        if period == "today":
            start_msk = now_msk.replace(hour=0, minute=0, second=0, microsecond=0)
            start_utc = start_msk - timedelta(hours=3)
            end_utc = start_utc + timedelta(days=1)
            start_date = start_utc.isoformat().replace("+00:00", "Z")
            end_date = end_utc.isoformat().replace("+00:00", "Z")
        elif period == "yesterday":
            start_msk = now_msk.replace(hour=0, minute=0, second=0, microsecond=0) - timedelta(days=1)
            start_utc = start_msk - timedelta(hours=3)
            end_utc = start_utc + timedelta(days=1)
            start_date = start_utc.isoformat().replace("+00:00", "Z")
            end_date = end_utc.isoformat().replace("+00:00", "Z")
        
        await state.update_data(wispstats_start=start_date, wispstats_end=end_date, wispstats_period_name=period)
        await render_wispstats_page(call.message, 0, state)
        await call.answer()
    elif action.startswith("wispstatsPage:"):
        page = int(action.split(":")[1])
        await render_wispstats_page(call.message, page, state)
        await call.answer()

async def api_manage_whitelist(action, type_name, value, name=None):
    headers = {"x-bot-token": API_SECRET, "Content-Type": "application/json"}
    payload = {"type": type_name, "value": value}
    if name:
        payload["name"] = name
        
    try:
        async with aiohttp.ClientSession() as session:
            if action == "add":
                async with session.post(f"{API_BASE}/api/admin/whitelists", json=payload, headers=headers) as resp:
                    return resp.status == 200
            elif action == "del":
                async with session.delete(f"{API_BASE}/api/admin/whitelists", json=payload, headers=headers) as resp:
                    return resp.status == 200
    except Exception as e:
        print("API error:", e)
        return False
    return False

async def api_manage_hwid_name(hwid, name):
    headers = {"x-bot-token": API_SECRET, "Content-Type": "application/json"}
    payload = {"hwid": hwid, "name": name}
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(f"{API_BASE}/api/admin/hwid_name", json=payload, headers=headers) as resp:
                return resp.status == 200
    except Exception as e:
        print("API error hwid_name:", e)
        return False

async def api_manage_users(action, user_id=None, name=None):
    headers = {"x-bot-token": API_SECRET}
    try:
        async with aiohttp.ClientSession() as session:
            if action == "list":
                async with session.get(f"{API_BASE}/api/admin/users", headers=headers) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        return data.get("users", [])
            elif action == "edit":
                payload = {"username": name}
                headers["Content-Type"] = "application/json"
                async with session.put(f"{API_BASE}/api/admin/users/{user_id}", json=payload, headers=headers) as resp:
                    return resp.status == 200
            elif action == "del":
                async with session.delete(f"{API_BASE}/api/admin/users/{user_id}", headers=headers) as resp:
                    status = resp.status
                    text = await resp.text()
                    print(f"DEBUG DELETE user {user_id}: status={status} text={text}")
                    return status == 200
    except Exception as e:
        print("API error users:", e)
        return [] if action == "list" else False
    return [] if action == "list" else False

async def api_fetch_traffic():
    headers = {"x-bot-token": API_SECRET}
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{API_BASE}/api/admin/traffic", headers=headers, timeout=10) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    return {str(u["id"]): u for u in data.get("users", [])}
    except Exception as e:
        print("API error traffic:", e)
    return {}

async def api_send_command(user_id, command):
    headers = {"x-bot-token": API_SECRET, "Content-Type": "application/json"}
    payload = {"user_id": user_id, "command": command}
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(f"{API_BASE}/api/admin/command", json=payload, headers=headers) as resp:
                return resp.status == 200
    except Exception as e:
        print("API error command:", e)
        return False

def format_bytes(b):
    if b < 1024:
        return f"{b} B"
    elif b < 1024 * 1024:
        return f"{b / 1024:.1f} KB"
    elif b < 1024 * 1024 * 1024:
        return f"{b / (1024*1024):.1f} MB"
    else:
        return f"{b / (1024*1024*1024):.2f} GB"

async def render_dbusers_page(message: types.Message, page: int):
    users = await api_manage_users("list")
    per_page = 8
    total_pages = max(1, (len(users) + per_page - 1) // per_page)
    if page >= total_pages: page = total_pages - 1
    if page < 0: page = 0
    
    start_idx = page * per_page
    end_idx = start_idx + per_page
    page_users = users[start_idx:end_idx]
    
    kb = []
    for u in page_users:
        kb.append([InlineKeyboardButton(text=f"👤 {u['username']} (ID: {u['id']})", callback_data=f"u_view:{u['id']}")])
        
    nav_row = []
    if page > 0:
        nav_row.append(InlineKeyboardButton(text="⬅️ Назад", callback_data=f"u_page:{page-1}"))
    if page < total_pages - 1:
        nav_row.append(InlineKeyboardButton(text="Вперед ➡️", callback_data=f"u_page:{page+1}"))
        
    if nav_row: kb.append(nav_row)
    kb.append([InlineKeyboardButton(text="🔙 Назад в меню", callback_data="admin_back")])
    
    text = f"🧑‍💻 *Зарегистрированные юзеры*\n\nВсего в базе: {len(users)}\nСтраница: {page+1}/{total_pages}"
    await message.edit_text(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))

async def render_devlogs_page(message: types.Message, page: int, state: FSMContext = None, edit_msg: bool = True):
    headers = {"x-bot-token": API_SECRET}
    filter_user = None
    if state:
        data = await state.get_data()
        filter_user = data.get("devlogs_filter_user")
        
    url = f"{API_BASE}/api/admin/device_events"
    params = {}
    if filter_user:
        params["username"] = filter_user

    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(url, headers=headers, params=params, timeout=10) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    events = data.get("events", [])
                    
                    filter_str = f" (Поиск: `{filter_user}`)" if filter_user else ""
                    
                    if not events:
                        text = f"📜 *Логи устройств*{filter_str}\n\nПусто."
                        kb_btns = [
                            [InlineKeyboardButton(text="🔍 Фильтр по юзеру", callback_data="admin_devlogs_filter")],
                        ]
                        if filter_user:
                            kb_btns.append([InlineKeyboardButton(text="❌ Сбросить фильтр", callback_data="admin_devlogs_clearfilter")])
                        kb_btns.append([InlineKeyboardButton(text="🔙 Назад в меню", callback_data="admin_back")])
                        if edit_msg:
                            return await message.edit_text(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb_btns))
                        else:
                            return await message.answer(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb_btns))
                        
                    per_page = 10
                    total_pages = max(1, (len(events) + per_page - 1) // per_page)
                    if page >= total_pages: page = total_pages - 1
                    if page < 0: page = 0
                    
                    start_idx = page * per_page
                    end_idx = start_idx + per_page
                    page_events = events[start_idx:end_idx]
                    
                    text = f"📜 *Логи устройств*{filter_str}\nСтраница: {page+1}/{total_pages}\n\n"
                    for ev in page_events:
                        device_id = ev.get("device_id", "Unknown")
                        device_name = ev.get("device_name")
                        if device_name:
                            device_id = f"💻 {device_name}"
                        else:
                            device_id = f"`{device_id}`"
                        event_type = ev.get("event_type", "Unknown").upper()
                        browser_type = ev.get("browser_type", "Unknown")
                        username = ev.get("username")
                        
                        from datetime import datetime, timedelta
                        raw_date = ev.get("created_at", "")
                        try:
                            dt = datetime.fromisoformat(raw_date.replace("Z", "+00:00"))
                            dt_msk = dt + timedelta(hours=3)
                            created_at = dt_msk.strftime("%d.%m.%Y %H:%M:%S")
                        except Exception:
                            if "T" in raw_date:
                                parts = raw_date.split("T")
                                created_at = parts[0] + " " + parts[1][:5]
                            else:
                                created_at = str(raw_date)[:16]
                        
                        type_emoji = "🟢" if event_type == "OPEN" else "🔴" if event_type == "CLOSE" else "🔑"
                        browser_str = "🕵️ Скрытый" if browser_type == "hidden" else "🌐 Обычный"
                        user_str = f"👤 {username}" if username else "👤 Без логина"
                        
                        text += f"{type_emoji} {device_id}\n"
                        text += f"   {browser_str} | {user_str} | {event_type} | 🕒 {created_at}\n\n"
                        
                    kb = []
                    nav_row = []
                    if page > 0:
                        nav_row.append(InlineKeyboardButton(text="⬅️ Назад", callback_data=f"admin_devlogsPage:{page-1}"))
                    if page < total_pages - 1:
                        nav_row.append(InlineKeyboardButton(text="Вперед ➡️", callback_data=f"admin_devlogsPage:{page+1}"))
                    
                    if nav_row: kb.append(nav_row)
                    
                    # Control buttons
                    ctrl_row = [InlineKeyboardButton(text="🔍 Фильтр", callback_data="admin_devlogs_filter")]
                    if filter_user:
                        ctrl_row.append(InlineKeyboardButton(text="❌ Сбросить", callback_data="admin_devlogs_clearfilter"))
                    kb.append(ctrl_row)
                    
                    kb.append([InlineKeyboardButton(text="🗑 Очистить логи", callback_data="admin_devlogs_clear")])
                    kb.append([InlineKeyboardButton(text="🔙 Назад в меню", callback_data="admin_back")])
                    
                    from datetime import datetime
                    text += f"\n\n🕒 Последнее обновление: {datetime.now().strftime('%H:%M:%S')}"
                    
                    if edit_msg:
                        await message.edit_text(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))
                    else:
                        await message.answer(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))
                else:
                    err_text = await resp.text()
                    print(f"DEBUG: /api/admin/device_events returned {resp.status} - {err_text}")
                    err_msg = f"❌ Ошибка сервера при получении логов.\nСтатус: {resp.status}\nТекст: {err_text[:100]}"
                    if edit_msg:
                        await message.edit_text(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
                    else:
                        await message.answer(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
    except Exception as e:
        print("API error device events:", e)
        err_msg = "❌ Ошибка сети при получении логов."
        if edit_msg:
            await message.edit_text(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
        else:
            await message.answer(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))

async def render_wisplogs_page(message: types.Message, page: int, state: FSMContext = None, edit_msg: bool = True):
    headers = {"x-bot-token": API_SECRET}
    filter_user = None
    if state:
        data = await state.get_data()
        filter_user = data.get("wisplogs_filter_user")
        
    url = f"{API_BASE}/api/admin/wisp_logs"
    per_page = 10
    params = {"page": page, "limit": per_page}
    if filter_user:
        params["username"] = filter_user

    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(url, headers=headers, params=params, timeout=10) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    events = data.get("events", [])
                    total = data.get("total", 0)
                    
                    filter_str = f" (Поиск: `{filter_user}`)" if filter_user else ""
                    
                    if not events:
                        text = f"🌐 *Логи Wisp*{filter_str}\n\nПусто."
                        kb_btns = [
                            [InlineKeyboardButton(text="🔍 Фильтр", callback_data="admin_wisplogs_filter")],
                        ]
                        if filter_user:
                            kb_btns.append([InlineKeyboardButton(text="❌ Сбросить фильтр", callback_data="admin_wisplogs_clearfilter")])
                        kb_btns.append([InlineKeyboardButton(text="🔙 Назад в меню", callback_data="admin_back")])
                        if edit_msg:
                            return await message.edit_text(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb_btns))
                        else:
                            return await message.answer(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb_btns))
                        
                    total_pages = max(1, (total + per_page - 1) // per_page)
                    
                    text = f"🌐 *Логи Wisp*{filter_str}\nСтраница: {page+1}/{total_pages} (Всего: {total})\n\n"
                    for ev in events:
                        target_host = ev.get("target_host", "Unknown")
                        target_port = ev.get("target_port", "Unknown")
                        username = ev.get("username")
                        telegram_id = ev.get("telegram_id", "Unknown")
                        
                        from datetime import datetime, timedelta
                        raw_date = ev.get("created_at", "")
                        try:
                            dt = datetime.fromisoformat(raw_date.replace("Z", "+00:00"))
                            dt_msk = dt + timedelta(hours=3)
                            created_at = dt_msk.strftime("%d.%m.%Y %H:%M:%S")
                        except Exception:
                            if "T" in raw_date:
                                parts = raw_date.split("T")
                                created_at = parts[0] + " " + parts[1][:5]
                            else:
                                created_at = str(raw_date)[:16]
                        
                        user_str = f"{username} / {telegram_id}" if username else f"{telegram_id}"
                        
                        text += f"🌐 `{target_host}:{target_port}`\n"
                        text += f"   👤 {user_str}\n"
                        text += f"   🕒 {created_at}\n\n"
                        
                    kb = []
                    nav_row = []
                    if page > 0:
                        nav_row.append(InlineKeyboardButton(text="⬅️ Назад", callback_data=f"admin_wisplogsPage:{page-1}"))
                    if page < total_pages - 1:
                        nav_row.append(InlineKeyboardButton(text="Вперед ➡️", callback_data=f"admin_wisplogsPage:{page+1}"))
                    
                    if nav_row: kb.append(nav_row)
                    
                    ctrl_row = [InlineKeyboardButton(text="🔍 Фильтр", callback_data="admin_wisplogs_filter")]
                    if filter_user:
                        ctrl_row.append(InlineKeyboardButton(text="❌ Сбросить", callback_data="admin_wisplogs_clearfilter"))
                    kb.append(ctrl_row)
                    
                    kb.append([InlineKeyboardButton(text="🗑 Очистить логи", callback_data="admin_wisplogs_clear")])
                    kb.append([InlineKeyboardButton(text="🔙 Назад в меню", callback_data="admin_back")])
                    
                    from datetime import datetime
                    text += f"\n\n🕒 Последнее обновление: {datetime.now().strftime('%H:%M:%S')}"
                    
                    if edit_msg:
                        await message.edit_text(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))
                    else:
                        await message.answer(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))
                else:
                    err_text = await resp.text()
                    print(f"DEBUG: /api/admin/wisp_logs returned {resp.status} - {err_text}")
                    err_msg = f"❌ Ошибка сервера при получении логов.\nСтатус: {resp.status}\nТекст: {err_text[:100]}"
                    if edit_msg:
                        await message.edit_text(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
                    else:
                        await message.answer(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
    except Exception as e:
        print("API error wisp logs:", e)
        err_msg = "❌ Ошибка сети при получении логов."
        if edit_msg:
            await message.edit_text(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
        else:
            await message.answer(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))

async def render_wispstats_page(message: types.Message, page: int, state: FSMContext = None, edit_msg: bool = True):
    headers = {"x-bot-token": API_SECRET}
    filter_user = None
    start_date = None
    end_date = None
    period_name = "all"
    
    if state:
        data = await state.get_data()
        filter_user = data.get("wispstats_user")
        start_date = data.get("wispstats_start")
        end_date = data.get("wispstats_end")
        period_name = data.get("wispstats_period_name", "all")
        
    url = f"{API_BASE}/api/admin/wisp_stats"
    per_page = 10
    params = {"page": page, "limit": per_page}
    
    if filter_user and filter_user.lower() != "все":
        params["username"] = filter_user
    if start_date:
        params["start_date"] = start_date
    if end_date:
        params["end_date"] = end_date

    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(url, headers=headers, params=params, timeout=10) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    stats = data.get("stats", [])
                    total_hosts = data.get("total_hosts", 0)
                    total_requests = data.get("total_requests", 0)
                    
                    period_str = "За всё время"
                    if period_name == "today":
                        period_str = "Сегодня"
                    elif period_name == "yesterday":
                        period_str = "Вчера"
                    elif period_name == "custom":
                        period_str = "Свой промежуток"
                        
                    user_str = filter_user if (filter_user and filter_user.lower() != "все") else "Все пользователи"
                    
                    text = f"📊 *Статистика Wisp*\n👤 Пользователь: `{user_str}`\n📅 Период: `{period_str}`\n\n"
                    text += f"Всего запросов: `{total_requests}`\nУникальных сайтов: `{total_hosts}`\n\n"
                    
                    if not stats:
                        text += "Нет данных за выбранный период."
                    else:
                        total_pages = max(1, (total_hosts + per_page - 1) // per_page)
                        for s in stats:
                            host = s.get("target_host", "Unknown")
                            count = s.get("req_count", 0)
                            text += f"🌐 `{host}` — {count} запроса(ов)\n"
                        
                        text += f"\nСтраница {page+1}/{total_pages}"
                        
                    kb = []
                    nav_row = []
                    
                    # For pagination we use total_hosts
                    total_pages = max(1, (total_hosts + per_page - 1) // per_page)
                    if page > 0:
                        nav_row.append(InlineKeyboardButton(text="⬅️ Назад", callback_data=f"admin_wispstatsPage:{page-1}"))
                    if page < total_pages - 1:
                        nav_row.append(InlineKeyboardButton(text="Вперед ➡️", callback_data=f"admin_wispstatsPage:{page+1}"))
                    
                    if nav_row: kb.append(nav_row)
                    
                    kb.append([InlineKeyboardButton(text="🔙 К выбору пользователя", callback_data="admin_wispstats")])
                    kb.append([InlineKeyboardButton(text="🔙 Назад в меню", callback_data="admin_back")])
                    
                    if edit_msg:
                        await message.edit_text(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))
                    else:
                        await message.answer(text, reply_markup=InlineKeyboardMarkup(inline_keyboard=kb))
                else:
                    err_text = await resp.text()
                    err_msg = f"❌ Ошибка сервера.\nСтатус: {resp.status}"
                    if edit_msg:
                        await message.edit_text(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
                    else:
                        await message.answer(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
    except Exception as e:
        print("API error wisp stats:", e)
        err_msg = "❌ Ошибка сети при получении статистики."
        if edit_msg:
            await message.edit_text(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))
        else:
            await message.answer(err_msg, reply_markup=InlineKeyboardMarkup(inline_keyboard=[[InlineKeyboardButton(text="🔙 Назад", callback_data="admin_back")]]))


@dp.callback_query(F.data.startswith("u_"))
async def handle_dbusers_callbacks(call: CallbackQuery, state: FSMContext):
    if str(call.from_user.id) != ADMIN_ID: return
    parts = call.data.split(":")
    action = parts[0]
    val = parts[1]
    
    if action == "u_page":
        await render_dbusers_page(call.message, int(val))
    elif action == "u_view":
        users = await api_manage_users("list")
        u = next((x for x in users if str(x['id']) == val), None)
        if not u:
            return await call.answer("Пользователь не найден", show_alert=True)
        
        # Fetch traffic data
        traffic_data = await api_fetch_traffic()
        t = traffic_data.get(val, {})
        
        connected = t.get("connected", False)
        domain = t.get("domain", None)
        session_bytes = t.get("sessionBytes", 0)
        total_bytes = t.get("totalBytes", 0)
        
        status_emoji = "🟢" if connected else "🔴"
        status_text = "Онлайн" if connected else "Офлайн"
        
        hwid = t.get("hwid")
        hwid_name = t.get("hwid_name")
        hw_str = "Нет данных"
        if hwid:
            if hwid_name:
                hw_str = f"{hwid_name}"
            else:
                hw_str = f"`{hwid}`"
        
        text = (
            f"👤 *Профиль пользователя*\n\n"
            f"*ID в базе:* `{u['id']}`\n"
            f"*TG ID:* `{u.get('telegram_id', 'Не привязан')}`\n"
            f"*Имя/Логин:* {u['username']}\n"
            f"*ПК:* {hw_str}\n\n"
            f"{status_emoji} *Статус:* {status_text}\n"
        )
        
        if connected and domain:
            text += f"🌐 *Поддомен:* `{domain}`\n"
        
        text += (
            f"📥 *Трафик (сессия):* {format_bytes(session_bytes)}\n"
            f"📊 *Трафик (всего):* {format_bytes(total_bytes)}"
        )
        
        kb = InlineKeyboardMarkup(inline_keyboard=[
            [InlineKeyboardButton(text="🔍 Найти устройство", callback_data=f"u_cmd:find:{u['id']}")],
            [InlineKeyboardButton(text="🚪 Обычный режим", callback_data=f"u_cmd:norm:{u['id']}")],
            [InlineKeyboardButton(text="✒️ Изменить Имя", callback_data=f"u_edit:{u['id']}")],
            [InlineKeyboardButton(text="💻 Назвать ПК", callback_data=f"u_hwname:{u['id']}")],
            [InlineKeyboardButton(text="🗑 Удалить браузер", callback_data=f"u_delapp:{u['id']}")],
            [InlineKeyboardButton(text="❌ Удалить", callback_data=f"u_del:{u['id']}")],
            [InlineKeyboardButton(text="🔙 К списку", callback_data="admin_dbusers")]
        ])
        await call.message.edit_text(text, reply_markup=kb)
    elif action == "u_delapp":
        kb = InlineKeyboardMarkup(inline_keyboard=[
            [InlineKeyboardButton(text="✅ Да, удалить", callback_data=f"u_delapp_go:{val}")],
            [InlineKeyboardButton(text="❌ Отмена", callback_data=f"u_view:{val}")]
        ])
        await call.message.edit_text(
            "⚠️ *Удалить браузер на устройстве безвозвратно?*\n\n"
            "После подтверждения браузер закроется, его .exe будет затёрт нулями и удалён.",
            reply_markup=kb
        )
        await call.answer()
    elif action == "u_delapp_go":
        success = await api_send_command(val, "del")
        if success:
            await call.answer("Команда отправлена! Браузер будет удалён.", show_alert=True)
            await call.message.delete()
        else:
            await call.answer("Ошибка отправки команды", show_alert=True)
    elif action == "u_del":
        success = await api_manage_users("del", user_id=val)
        if success:
            await call.answer("Пользователь удален!", show_alert=True)
            await render_dbusers_page(call.message, 0)
        else:
            await call.answer("Ошибка сервера", show_alert=True)
    elif action == "u_cmd":
        cmd = parts[1]
        uid = parts[2]
        success = await api_send_command(uid, cmd)
        if success:
            await call.answer("Команда отправлена!", show_alert=True)
        else:
            await call.answer("Ошибка отправки команды", show_alert=True)
    elif action == "u_edit":
        await state.update_data(edit_user_id=val)
        await state.set_state(AdminStates.waiting_for_user_new_name)
        
        sent = await call.message.answer("✏️ Введите новое имя для этого пользователя:")
        asyncio.create_task(delete_messages_later(sent, delay=300))
        await call.answer()
    elif action == "u_hwname":
        traffic_data = await api_fetch_traffic()
        t = traffic_data.get(val, {})
        hwid = t.get("hwid")
        if not hwid:
            return await call.answer("Нет известного HWID для этого пользователя", show_alert=True)
            
        await state.update_data(hwid_to_name=hwid)
        await state.set_state(AdminStates.waiting_for_hwid_name)
        
        sent = await call.message.answer(f"💻 Введите новое название для ПК с HWID:\n`{hwid}`")
        asyncio.create_task(delete_messages_later(sent, delay=300))
        await call.answer()
    
    try: await call.answer()
    except: pass

@dp.message(AdminStates.waiting_for_hwid_name)
async def process_hwid_name(message: types.Message, state: FSMContext):
    data = await state.get_data()
    hwid = data.get("hwid_to_name")
    new_name = message.text.strip()
    
    success = await api_manage_hwid_name(hwid, new_name)
    if success:
        sent = await message.answer(f"✅ Название ПК успешно сохранено: '{new_name}' для HWID: `{hwid}`")
    else:
        sent = await message.answer("❌ Ошибка сервера при сохранении названия ПК.")
    asyncio.create_task(delete_messages_later(message, sent, 300))
    await state.clear()

@dp.message(AdminStates.waiting_for_global_hwid)
async def process_global_hwid(message: types.Message, state: FSMContext):
    hwid = message.text.strip()
    await state.update_data(global_hwid_to_name=hwid)
    await state.set_state(AdminStates.waiting_for_global_hwid_name)
    
    sent = await message.answer(f"💻 Введите название для ПК с HWID:\n`{hwid}`\n\n(Для отмены введите /cancel)")
    asyncio.create_task(delete_messages_later(message, sent, 300))

@dp.message(AdminStates.waiting_for_global_hwid_name)
async def process_global_hwid_name(message: types.Message, state: FSMContext):
    data = await state.get_data()
    hwid = data.get("global_hwid_to_name")
    new_name = message.text.strip()
    
    success = await api_manage_hwid_name(hwid, new_name)
    if success:
        sent = await message.answer(f"✅ Название ПК успешно сохранено: '{new_name}' для HWID: `{hwid}`")
    else:
        sent = await message.answer("❌ Ошибка сервера при сохранении названия ПК.")
    asyncio.create_task(delete_messages_later(message, sent, 300))
    await state.clear()

@dp.message(AdminStates.waiting_for_user_new_name)
async def process_edit_user_name(message: types.Message, state: FSMContext):
    data = await state.get_data()
    uid = data.get("edit_user_id")
    new_name = message.text.strip()
    
    success = await api_manage_users("edit", user_id=uid, name=new_name)
    if success:
        sent = await message.answer(f"✅ Имя пользователя обновлено на '{new_name}'.")
    else:
        sent = await message.answer("❌ Ошибка сервера при обновлении имени.")
    asyncio.create_task(delete_messages_later(message, sent, 300))
    await state.clear()

@dp.message(AdminStates.waiting_for_devlogs_user)
async def process_devlogs_user(message: types.Message, state: FSMContext):
    username = message.text.strip()
    await state.update_data(devlogs_filter_user=username)
    
    # Render immediately
    await render_devlogs_page(message, 0, state, edit_msg=False)
    asyncio.create_task(delete_messages_later(message, delay=300))
    await state.set_state(None)

@dp.message(AdminStates.waiting_for_wisplogs_user)
async def process_wisplogs_user(message: types.Message, state: FSMContext):
    username = message.text.strip()
    await state.update_data(wisplogs_filter_user=username)
    
    # Render immediately
    await render_wisplogs_page(message, 0, state, edit_msg=False)
    asyncio.create_task(delete_messages_later(message, delay=300))
    await state.set_state(None)

@dp.message(AdminStates.waiting_for_wispstats_user)
async def process_wispstats_user(message: types.Message, state: FSMContext):
    username = message.text.strip()
    await state.update_data(wispstats_user=username)
    
    kb = InlineKeyboardMarkup(inline_keyboard=[
        [InlineKeyboardButton(text="Сегодня", callback_data="admin_wispstats_period:today")],
        [InlineKeyboardButton(text="Вчера", callback_data="admin_wispstats_period:yesterday")],
        [InlineKeyboardButton(text="За 3 дня (Всё время)", callback_data="admin_wispstats_period:all")],
        [InlineKeyboardButton(text="Свой промежуток", callback_data="admin_wispstats_period:custom")],
        [InlineKeyboardButton(text="🔙 Отмена", callback_data="admin_back")]
    ])
    await message.answer(f"Пользователь: `{username}`\nВыберите период для статистики:", reply_markup=kb)
    asyncio.create_task(delete_messages_later(message, delay=300))
    await state.set_state(None)

@dp.message(AdminStates.waiting_for_wispstats_period_custom)
async def process_wispstats_custom(message: types.Message, state: FSMContext):
    text = message.text.strip()
    try:
        parts = text.split("-")
        if len(parts) != 2: raise ValueError("No dash")
        start_str = parts[0].strip()
        end_str = parts[1].strip()
        
        from datetime import datetime, timedelta, timezone
        start_msk = datetime.strptime(start_str, "%d.%m.%Y %H:%M")
        end_msk = datetime.strptime(end_str, "%d.%m.%Y %H:%M")
        
        start_utc = start_msk - timedelta(hours=3)
        end_utc = end_msk - timedelta(hours=3)
        
        start_iso = start_utc.replace(tzinfo=timezone.utc).isoformat().replace("+00:00", "Z")
        end_iso = end_utc.replace(tzinfo=timezone.utc).isoformat().replace("+00:00", "Z")
        
        await state.update_data(wispstats_start=start_iso, wispstats_end=end_iso, wispstats_period_name="custom")
        await render_wispstats_page(message, 0, state, edit_msg=False)
        asyncio.create_task(delete_messages_later(message, delay=300))
        await state.set_state(None)
    except Exception as e:
        await message.answer("❌ Неверный формат даты.\nОжидаемый формат: `ДД.ММ.ГГГГ ЧЧ:ММ - ДД.ММ.ГГГГ ЧЧ:ММ`\n\nПопробуйте ввести еще раз:", parse_mode="Markdown")

@dp.callback_query(F.data.startswith("toggle_"))
async def toggle_all_callback(call: CallbackQuery):
    if str(call.from_user.id) != ADMIN_ID:
        return
    type_name = call.data.split("_")[1]
    
    if type_name == "ip":
        if "all" in cached_ips:
            success = await api_manage_whitelist("del", "ip", "all")
            if success:
                cached_ips.remove("all")
                await call.answer("Значение ALL удалено (фильтр ВКЛЮЧЕН)", show_alert=True)
        else:
            success = await api_manage_whitelist("add", "ip", "all")
            if success:
                cached_ips.append("all")
                await call.answer("Значение ALL добавлено (фильтр ОТКЛЮЧЕН)", show_alert=True)
    else:
        if "all" in cached_tg_ids:
            success = await api_manage_whitelist("del", "tg_id", "all")
            if success:
                cached_tg_ids.pop("all", None)
                await call.answer("Значение ALL удалено (фильтр ВКЛЮЧЕН)", show_alert=True)
        else:
            success = await api_manage_whitelist("add", "tg_id", "all", "ALL")
            if success:
                cached_tg_ids["all"] = "ALL"
                await call.answer("Значение ALL добавлено (фильтр ОТКЛЮЧЕН)", show_alert=True)
            
    # Refresh menu
    await render_admin_list(call.message, type_name)

@dp.callback_query(F.data.startswith("add_"))
async def add_callback(call: CallbackQuery, state: FSMContext):
    if str(call.from_user.id) != ADMIN_ID:
        return
    type_name = call.data.split("_")[1]
    
    if type_name == "ip":
        await state.update_data(del_type="")
        await state.set_state(AdminStates.waiting_for_ip)
        sent = await call.message.answer("Введите IP или CIDR для ДОБАВЛЕНИЯ (например, 192.168.1.1 или 10.0.0.0/24):")
        asyncio.create_task(delete_messages_later(sent, delay=300))
    else:
        await state.update_data(del_type="")
        await state.set_state(AdminStates.waiting_for_tg_id)
        sent = await call.message.answer("Введите Telegram ID для ДОБАВЛЕНИЯ:")
        asyncio.create_task(delete_messages_later(sent, delay=300))
    await call.answer()

@dp.callback_query(F.data.startswith("del_"))
async def del_callback(call: CallbackQuery, state: FSMContext):
    if str(call.from_user.id) != ADMIN_ID:
        return
    type_name = call.data.split("_")[1]
    
    if type_name == "ip":
        await state.update_data(del_type="ip")
        await state.set_state(AdminStates.waiting_for_ip)
        sent = await call.message.answer("Введите IP для УДАЛЕНИЯ:")
        asyncio.create_task(delete_messages_later(sent, delay=300))
    else:
        await state.update_data(del_type="tg")
        await state.set_state(AdminStates.waiting_for_tg_id)
        sent = await call.message.answer("Введите TG ID для УДАЛЕНИЯ:")
        asyncio.create_task(delete_messages_later(sent, delay=300))
    await call.answer()

@dp.message(AdminStates.waiting_for_ip)
async def process_ip_input(message: types.Message, state: FSMContext):
    data = await state.get_data()
    action = "del" if data.get("del_type") else "add"
    ip_str = message.text.strip()
    
    try:
        if action == "add" and ip_str != "all":
            if "/" in ip_str: ipaddress.ip_network(ip_str, strict=False)
            else: ipaddress.ip_address(ip_str)
            
        success = await api_manage_whitelist(action, "ip", ip_str)
        if success:
            if action == "add" and ip_str not in cached_ips: cached_ips.append(ip_str)
            elif action == "del" and ip_str in cached_ips: cached_ips.remove(ip_str)
            sent = await message.answer(f"✅ IP {ip_str} {'добавлен' if action == 'add' else 'удален'}.")
        else:
            sent = await message.answer("❌ Ошибка сервера.")
    except ValueError:
        sent = await message.answer("❌ Некорректный формат IP.")
        
    asyncio.create_task(delete_messages_later(message, sent, 300))
    await state.clear()

@dp.message(AdminStates.waiting_for_tg_id)
async def process_tg_input(message: types.Message, state: FSMContext):
    data = await state.get_data()
    action = "del" if data.get("del_type") else "add"
    tg_str = message.text.strip()
    
    if tg_str != "all" and not tg_str.isdigit():
        sent = await message.answer("❌ TG ID должен быть числом.")
        asyncio.create_task(delete_messages_later(message, sent, 300))
        await state.clear()
        return
        
    if action == "del":
        success = await api_manage_whitelist("del", "tg_id", tg_str)
        if success:
            cached_tg_ids.pop(tg_str, None)
            sent = await message.answer(f"✅ TG ID {tg_str} удален.")
        else:
            sent = await message.answer("❌ Ошибка сервера.")
        asyncio.create_task(delete_messages_later(message, sent, 300))
        await state.clear()
    else:
        # add action
        if tg_str == "all":
            success = await api_manage_whitelist("add", "tg_id", "all", "ALL")
            if success:
                cached_tg_ids["all"] = "ALL"
                sent = await message.answer("✅ ALL добавлены.")
                asyncio.create_task(delete_messages_later(message, sent, 300))
            await state.clear()
        else:
            await state.update_data(tg_id=tg_str)
            await state.set_state(AdminStates.waiting_for_name)
            sent = await message.answer("Укажите имя для этого TG ID (например, Иван Иванов):")
            asyncio.create_task(delete_messages_later(message, sent, 300))

@dp.message(AdminStates.waiting_for_name)
async def process_tg_name(message: types.Message, state: FSMContext):
    data = await state.get_data()
    tg_str = data.get("tg_id")
    tg_name = message.text.strip()
    
    success = await api_manage_whitelist("add", "tg_id", tg_str, tg_name)
    if success:
        cached_tg_ids[tg_str] = tg_name
        sent = await message.answer(f"✅ TG ID {tg_str} сохранен под именем '{tg_name}'.")
    else:
        sent = await message.answer("❌ Ошибка сервера.")
        
    asyncio.create_task(delete_messages_later(message, sent, 300))
    await state.clear()

# ----------------- REGULAR USERS -----------------

async def delete_messages_later(msg1: types.Message, msg2: types.Message = None, delay: int = 120):
    await asyncio.sleep(delay)
    for m in (msg1, msg2):
        if m:
            try:
                await m.delete()
            except Exception as e:
                pass

@dp.message(Command("start", "help"))
async def send_welcome(message: types.Message):
    username = get_username(message)
    
    headers = {
        "x-bot-token": API_SECRET,
        "Content-Type": "application/json"
    }
    payload = {
        "username": username,
        "user_id": str(message.from_user.id)
    }
    
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(f"{API_BASE}/api/auth/register", json=payload, headers=headers, timeout=10) as response:
                status = response.status
        
        if status == 200:
            welcome_text = (
                f"🎉 *Регистрация успешна!*\n\n"
                f"⚡ *Быстрый вход:* напиши команду /login чтобы получить одноразовый 6-значный код для входа в браузер."
            )
            sent_message = await message.answer(welcome_text)
            asyncio.create_task(delete_messages_later(message, sent_message))
        elif status == 409 or status == 500:
            welcome_text = (
                f"ℹ️ *Вы уже зарегистрированы в системе!*\n\n"
                f"🔑 *Получить код для входа:* отправьте команду /login для получения одноразового кода авторизации в браузере."
            )
            sent_message = await message.answer(welcome_text)
        else:
            sent_message = await message.answer("❌ Произошла ошибка при регистрации. Попробуйте позже.")
            asyncio.create_task(delete_messages_later(message, sent_message))
    except Exception as e:
        print("Error during register:", e)
        sent_message = await message.answer("❌ Не удалось связаться с сервером авторизации.")
        asyncio.create_task(delete_messages_later(message, sent_message))

@dp.message(Command("login", "code", "otp"))
async def send_otp(message: types.Message):
    username = get_username(message)
    
    headers = {
        "x-bot-token": API_SECRET,
        "Content-Type": "application/json"
    }
    payload = {
        "username": username,
        "user_id": str(message.from_user.id)
    }
    
    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(f"{API_BASE}/api/auth/otp/generate", json=payload, headers=headers, timeout=10) as response:
                status = response.status
                if status == 200:
                    data = await response.json()
        
        if status == 200:
            code = data.get("code")
            expires = data.get("expiresIn", 120)
            
            otp_text = (
                f"🔑 *Твой одноразовый код для входа:*\n\n"
                f"#️⃣ `{code}`\n\n"
                f"⏳ Код действителен *{expires} секунд*.\n"
                f"Введи этот 6-значный код в окне браузера для входа."
            )
            sent_message = await message.answer(otp_text)
            asyncio.create_task(delete_messages_later(message, sent_message, expires))
        elif status == 404:
            sent_message = await message.answer("❌ Ваш аккаунт не найден. Отправьте /start для автоматической регистрации.")
            asyncio.create_task(delete_messages_later(message, sent_message))
        else:
            sent_message = await message.answer("❌ Ошибка сервера при генерации кода. Попробуйте позже.")
            asyncio.create_task(delete_messages_later(message, sent_message))
    except Exception as e:
        print("Error during OTP generate:", e)
        sent_message = await message.answer("❌ Не удалось связаться с сервером авторизации.")
        asyncio.create_task(delete_messages_later(message, sent_message))

@dp.message()
async def delete_unknown(message: types.Message):
    # Удаляем любое сообщение, кроме команд выше, через 120 секунд
    asyncio.create_task(delete_messages_later(message))

async def main():
    await fetch_whitelists()
    print("Telegram bot (aiogram) is running with OTP support...")
    await dp.start_polling(bot)

if __name__ == "__main__":
    asyncio.run(main())
