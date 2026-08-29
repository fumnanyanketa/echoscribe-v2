//! The low-level keyboard hook.
//!
//! This is the most security-sensitive code in the project (record 0002, Risk).
//! A `WH_KEYBOARD_LL` hook is called by Windows for every keystroke on the
//! machine while the app runs. To keep that safe this file holds itself to one
//! job and one rule:
//!
//!   * It classifies each key as "the chosen modifier" or "anything else", then
//!     hands that one bit to `machine::Machine`. It never records, buffers,
//!     stores or compares the actual key. There is no key log, not even in
//!     memory, not even for diagnostics.
//!   * When nobody is signed in the hook is disarmed (`ARMED` is false) and the
//!     callback returns immediately without looking at the event at all
//!     (record 0002 AC-16).
//!
//! The hook is installed once, on the first sign-in of a run, and then stays
//! installed for the life of the process; signing out only disarms it. On a
//! clean double tap the callback sends one unit on a channel; the consumer
//! thread in `mod.rs` turns that into "toggle dictation".

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use std::time::Instant;

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LMENU, VK_RCONTROL, VK_RMENU};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION,
    MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::machine::{Class, Machine};

/// The chosen modifier, as the two settings values map to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Alt,
}

/// False whenever the hotkey must do nothing: nobody signed in, or the hook not
/// yet wanted this run. The callback checks this before anything else.
static ARMED: AtomicBool = AtomicBool::new(false);

/// The modifier the machine is currently watching: 0 = Ctrl, 1 = Alt.
static CHOSEN: AtomicU8 = AtomicU8::new(0);

/// Set once the hook thread has been spawned. It is never torn down.
static INSTALLED: OnceLock<()> = OnceLock::new();

thread_local! {
    /// The double-tap recogniser. Only ever touched on the hook thread.
    static MACHINE: RefCell<Machine> = RefCell::new(Machine::default());
    /// Where a completed double tap is announced. Set once when the thread starts.
    static TOGGLE_TX: RefCell<Option<Sender<()>>> = const { RefCell::new(None) };
}

/// Start the hook thread if it is not already running, and hand it the channel
/// a double tap is announced on. Safe to call more than once; only the first
/// call does anything.
pub fn install(toggle_tx: Sender<()>) {
    if INSTALLED.set(()).is_err() {
        return;
    }
    std::thread::Builder::new()
        .name("echoscribe-keyboard-hook".into())
        .spawn(move || run(toggle_tx))
        .expect("spawn the keyboard hook thread");
}

/// Watch this modifier, and act on a double tap. Called on sign-in.
pub fn arm(modifier: Modifier) {
    CHOSEN.store(matches!(modifier, Modifier::Alt) as u8, Ordering::SeqCst);
    ARMED.store(true, Ordering::SeqCst);
}

/// Stop acting on the hotkey. The callback now returns without inspecting
/// events. Called on sign-out.
pub fn disarm() {
    ARMED.store(false, Ordering::SeqCst);
}

fn run(toggle_tx: Sender<()>) {
    TOGGLE_TX.with(|slot| *slot.borrow_mut() = Some(toggle_tx));

    let hmod = unsafe { GetModuleHandleW(None) }.expect("module handle for the hook");
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            Some(HINSTANCE(hmod.0)),
            0,
        )
    }
    .expect("install the low-level keyboard hook");
    // `hook` is kept only so it is not dropped; the hook lives for the process.
    let _ = hook;

    // A low-level hook needs a message loop on its own thread. Nothing posts to
    // this queue, so this simply parks the thread while letting Windows deliver
    // the callback.
    let mut msg = MSG::default();
    loop {
        let got = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if got.0 <= 0 {
            break;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// The callback Windows invokes for every key event. Keep it short.
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // `code < 0` means "must pass through untouched". Disarmed means the same.
    if code != HC_ACTION as i32 || !ARMED.load(Ordering::SeqCst) {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    // The only fields read: which virtual key, and whether it is a press or a
    // release. `vk` is turned into one bit and dropped; it is never stored.
    let kb = &*(lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT);
    let vk = kb.vkCode;
    let message = wparam.0 as u32;

    let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
    let up = message == WM_KEYUP || message == WM_SYSKEYUP;
    if !down && !up {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let class = if is_chosen_modifier(vk) {
        Class::Modifier
    } else {
        Class::Other
    };

    let fired = MACHINE.with(|m| m.borrow_mut().on_key(class, down, Instant::now()));
    if fired {
        TOGGLE_TX.with(|slot| {
            if let Some(tx) = slot.borrow().as_ref() {
                let _ = tx.send(());
            }
        });
    }

    CallNextHookEx(None, code, wparam, lparam)
}

/// Whether `vk` is the left or right key of the currently chosen modifier.
/// Record 0002: either side of the keyboard counts.
fn is_chosen_modifier(vk: u32) -> bool {
    let (left, right) = if CHOSEN.load(Ordering::SeqCst) == 1 {
        (VK_LMENU.0 as u32, VK_RMENU.0 as u32)
    } else {
        (VK_LCONTROL.0 as u32, VK_RCONTROL.0 as u32)
    };
    vk == left || vk == right
}
