#[cfg(windows)]
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
#[cfg(windows)]
use std::sync::OnceLock;
#[cfg(windows)]
use tauri::Manager;

#[cfg(windows)]
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
#[cfg(windows)]
static HOOK: AtomicIsize = AtomicIsize::new(0);
#[cfg(windows)]
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
#[cfg(windows)]
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

#[cfg(windows)]
#[derive(Clone, Copy)]
#[repr(C)]
struct KBDLLHOOKSTRUCT {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[cfg(windows)]
#[derive(Clone, Copy)]
#[repr(C)]
struct MSG {
    hwnd: isize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

#[cfg(windows)]
extern "system" {
    fn SetWindowsHookExW(
        id_hook: i32,
        lpfn: Option<unsafe extern "system" fn(i32, usize, isize) -> isize>,
        hmod: isize,
        dw_thread_id: u32,
    ) -> isize;
    fn UnhookWindowsHookEx(hhk: isize) -> i32;
    fn CallNextHookEx(hhk: isize, n_code: i32, w_param: usize, l_param: isize) -> isize;
    fn GetForegroundWindow() -> isize;
    fn GetWindowThreadProcessId(h_wnd: isize, lpdw_process_id: *mut u32) -> u32;
    fn GetCurrentThreadId() -> u32;
    fn PostThreadMessageW(id_thread: u32, msg: u32, w_param: usize, l_param: isize) -> i32;
    fn GetAncestor(h_wnd: isize, ga_flags: u32) -> isize;
    fn IsChild(h_wnd_parent: isize, h_wnd: isize) -> i32;
    fn GetModuleHandleW(lp_module_name: *const u16) -> isize;
    fn GetMessageW(lp_msg: *mut MSG, h_wnd: isize, w_msg_filter_min: u32, w_msg_filter_max: u32) -> i32;
    fn TranslateMessage(lp_msg: *const MSG) -> i32;
    fn DispatchMessageW(lp_msg: *const MSG) -> isize;
}

#[cfg(windows)]
unsafe fn is_our_app(fg_hwnd: isize) -> bool {
    if fg_hwnd == 0 {
        return false;
    }
    let our_pid = std::process::id();

    let mut fg_pid = 0u32;
    GetWindowThreadProcessId(fg_hwnd, &mut fg_pid);
    if fg_pid == our_pid {
        return true;
    }

    let mut main_hwnd = MAIN_HWND.load(Ordering::SeqCst);
    if main_hwnd == 0 {
        if let Some(app) = APP_HANDLE.get() {
            if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
                if let Ok(hwnd) = win.hwnd() {
                    main_hwnd = hwnd.0 as isize;
                    MAIN_HWND.store(main_hwnd, Ordering::SeqCst);
                }
            }
        }
    }

    if main_hwnd != 0 {
        if fg_hwnd == main_hwnd {
            return true;
        }
        let root = GetAncestor(fg_hwnd, 2);
        if root == main_hwnd {
            return true;
        }
        let root_owner = GetAncestor(fg_hwnd, 3);
        if root_owner == main_hwnd {
            return true;
        }
        if IsChild(main_hwnd, fg_hwnd) != 0 {
            return true;
        }
    }

    let root = GetAncestor(fg_hwnd, 2);
    if root != 0 {
        let mut root_pid = 0u32;
        GetWindowThreadProcessId(root, &mut root_pid);
        if root_pid == our_pid {
            return true;
        }
    }

    let root_owner = GetAncestor(fg_hwnd, 3);
    if root_owner != 0 {
        let mut owner_pid = 0u32;
        GetWindowThreadProcessId(root_owner, &mut owner_pid);
        if owner_pid == our_pid {
            return true;
        }
    }

    false
}

#[cfg(windows)]
unsafe extern "system" fn hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code >= 0 && (w_param == 0x0100 || w_param == 0x0104) {
        let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
        if kbd.vk_code == 0x7A || kbd.vk_code == 0x1B {
            let fg_hwnd = GetForegroundWindow();
            if is_our_app(fg_hwnd) {
                if let Some(app) = APP_HANDLE.get() {
                    if kbd.vk_code == 0x7A {
                        let app_clone = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            let _ = crate::commands::native_toggle_fullscreen(app_clone);
                        });
                        return 1;
                    } else if kbd.vk_code == 0x1B {
                        if crate::commands::IS_FULLSCREEN.load(Ordering::SeqCst) {
                            let app_clone = app.clone();
                            let _ = app.run_on_main_thread(move || {
                                let _ = crate::commands::native_exit_fullscreen_if_active(app_clone);
                            });
                            return 1;
                        }
                    }
                }
            }
        }
    }
    CallNextHookEx(HOOK.load(Ordering::SeqCst), n_code, w_param, l_param)
}

pub fn start(app: tauri::AppHandle) {
    #[cfg(windows)]
    {
        let _ = APP_HANDLE.set(app.clone());

        if let Some(win) = app.get_window("main").or_else(|| app.windows().into_values().next()) {
            if let Ok(hwnd) = win.hwnd() {
                MAIN_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
            }
        }

        std::thread::Builder::new()
            .name("keyboard_hook".to_string())
            .spawn(move || {
                unsafe {
                    let thread_id = GetCurrentThreadId();
                    HOOK_THREAD_ID.store(thread_id, Ordering::SeqCst);

                    let hmod = GetModuleHandleW(std::ptr::null());
                    let hook = SetWindowsHookExW(13, Some(hook_proc), hmod, 0);
                    if hook != 0 {
                        HOOK.store(hook, Ordering::SeqCst);
                    }

                    let mut msg = std::mem::zeroed();
                    while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            })
            .ok();
    }
    #[cfg(not(windows))]
    {
        let _ = app;
    }
}

pub fn stop() {
    #[cfg(windows)]
    {
        let hook = HOOK.swap(0, Ordering::SeqCst);
        if hook != 0 {
            unsafe {
                UnhookWindowsHookEx(hook);
            }
        }
        let thread_id = HOOK_THREAD_ID.swap(0, Ordering::SeqCst);
        if thread_id != 0 {
            unsafe {
                PostThreadMessageW(thread_id, 0x0012, 0, 0);
            }
        }
    }
}
