use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::IInputPaneInterop;
use windows::Win32::UI::Input::Touch::{RegisterTouchWindow, TWF_FINETOUCH};
use windows::UI::ViewManagement::{IInputPane, IInputPane2, InputPane};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};
// use winit::platform::windows::WindowExtWindows;

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        
        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);
        
        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("This is a sample window"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        );

        ShowWindow(hwnd, SW_NORMAL);
        println!("{:?}", RegisterTouchWindow(hwnd, TWF_FINETOUCH));
        
        let mut message = MSG::default();
        
        while GetMessageA(&mut message, hwnd, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
        
        Ok(())
    }
}

fn get_input_pane(hwnd: HWND) -> IInputPane2 {
    let input_pane_interop = factory::<InputPane, IInputPaneInterop>().unwrap();
    let input_pane = unsafe { input_pane_interop.GetForWindow::<_, IInputPane>(hwnd).unwrap() };
    let input_pane2 = (&input_pane).cast::<IInputPane2>().unwrap();
    println!("1111111");
    input_pane2
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_TOUCH => {
                println!("WM_TOUCH");
                Vtable::vtable(&get_input_pane(window)).TryShow;
                LRESULT(0)
            }
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
