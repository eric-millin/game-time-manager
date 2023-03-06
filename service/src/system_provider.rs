use sysinfo::{Pid, PidExt};
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::{GetLastError, HWND, RECT},
        Graphics::Gdi::{
            GetMonitorInfoA, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
        },
        Storage::EnhancedStorage::PKEY_Software_ProductName,
        System::Com::CoInitialize,
        UI::{
            Shell::{
                IShellItem2, SHCreateItemFromParsingName, SHQueryUserNotificationState, QUNS_BUSY,
                QUNS_NOT_PRESENT,
            },
            WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId},
        },
    },
};

const ERR_NO_GAME_FOUND: &str = "ErrNoGameFound";
static INIT_COM: std::sync::Once = std::sync::Once::new();

pub trait SystemProvider {
    fn try_get_game_pid(&self) -> Result<Pid, String>;
    fn try_get_product_name(&self, exe_name: String) -> Result<String, String>;
}

#[derive(Copy, Clone)]
pub struct Win32Provider {}

impl Win32Provider {
    pub fn new() -> Self {
        INIT_COM.call_once(|| {
            unsafe {
                // TODO: handle error
                CoInitialize(None);
            }
        });

        return Win32Provider {};
    }
}

impl SystemProvider for Win32Provider {
    fn try_get_game_pid(&self) -> Result<Pid, String> {
        // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ne-shellapi-query_user_notification_state
        let state = unsafe {
            match SHQueryUserNotificationState() {
                Ok(state) => state,
                Err(_) => QUNS_NOT_PRESENT,
            }
        };

        if state != QUNS_BUSY {
            return Err(ERR_NO_GAME_FOUND.to_string());
        }

        unsafe {
            let lpdwprocessid: u32 = 0;
            let hwnd = GetForegroundWindow();

            if !is_fullscreen(hwnd) {
                return Err(ERR_NO_GAME_FOUND.to_string());
            }

            let tid =
                GetWindowThreadProcessId(hwnd, Some((&lpdwprocessid as *const u32) as *mut u32));
            if tid == 0 {
                println!("win32 error: {:?}", GetLastError());
                return Err(ERR_NO_GAME_FOUND.to_string());
            }

            return Ok(Pid::from_u32(lpdwprocessid));
        };
    }

    fn try_get_product_name(&self, exe_name: String) -> Result<String, String> {
        unsafe {
            // TODO: error handling!
            let shi: IShellItem2 =
                SHCreateItemFromParsingName(PCWSTR(HSTRING::from(exe_name).as_ptr()), None)
                    .unwrap();

            match shi.GetString(&PKEY_Software_ProductName as _) {
                Ok(desc) => return Ok(desc.to_string().unwrap()),
                Err(_) => return Ok("".to_string()),
            }
        };
    }
}

fn is_fullscreen(hwnd: HWND) -> bool {
    let mut minfo = MONITORINFO::default();
    minfo.cbSize = std::mem::size_of::<MONITORINFO>() as _;

    let mut rect = RECT::default();

    unsafe {
        let hmnt = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);

        if !GetMonitorInfoA(hmnt, &mut minfo).as_bool() {
            return false;
        }

        if !GetWindowRect(hwnd, &mut rect).as_bool() {
            return false;
        }
    }

    return rect == minfo.rcMonitor;
}
