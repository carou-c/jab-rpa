use std::sync::mpsc;
use std::thread;

use crossbeam::channel;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
use windows::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, PM_NOREMOVE, PeekMessageW, TranslateMessage,
    },
};

use crate::callbacks::{ChangeEvent, shutdown_event_channel, subscribe_events};

fn run_message_pump() {
    unsafe {
        let mut msg = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, None, 0, 0);
            if result.0 <= 0 {
                if result.0 < 0 {
                    eprintln!("Message pump error: {}", result.0);
                }
                break; // WM_QUIT
            } else {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct JabRuntime {
    message_pump_handle: Option<thread::JoinHandle<()>>,
    message_pump_thread_id: u32,
    pub(crate) cb_rx: channel::Receiver<ChangeEvent>,
}

impl JabRuntime {
    pub(crate) fn new() -> Self {
        // Channel to synchronize initialization
        let (init_tx, init_rx) = mpsc::channel();
        let (thread_id_tx, thread_id_rx) = mpsc::channel();
        let (cb_channel_tx, cb_channel_rx) = mpsc::channel();

        // Start Windows message pump in dedicated thread (same thread will call initializeAccessBridge)
        let pump_handle = thread::spawn(move || {
            // Store thread ID for later shutdown
            let thread_id = unsafe { GetCurrentThreadId() };
            let _ = thread_id_tx.send(thread_id);

            unsafe {
                let _ = PeekMessageW(&mut std::mem::zeroed(), None, 0, 0, PM_NOREMOVE);
            }

            // Initialize JAB on this thread
            let init_result = unsafe { jab_sys::initializeAccessBridge() };
            let success = init_result != 0;
            let _ = init_tx.send(success);

            if success {
                let rx = unsafe { subscribe_events() };
                let _ = cb_channel_tx.send(rx);

                // Run message pump loop
                run_message_pump();
            }

            shutdown_event_channel();

            // Shutdown JAB
            if success {
                unsafe {
                    jab_sys::shutdownAccessBridge();
                }
            }
        });

        // Wait for initialization to complete
        match init_rx.recv() {
            Ok(true) => {}
            Ok(false) => panic!("Failed to initialize JAB"),
            Err(_) => panic!("Message pump thread crashed during initialization"),
        }

        let thread_id = match thread_id_rx.recv() {
            Ok(thread_id) => thread_id,
            Err(_) => panic!("Message pump thread crashed during initialization"),
        };
        let cb_rx = match cb_channel_rx.recv() {
            Ok(cb_rx) => cb_rx,
            Err(_) => panic!("Message pump thread crashed during initialization"),
        };

        Self {
            message_pump_handle: Some(pump_handle),
            message_pump_thread_id: thread_id,
            cb_rx,
        }
    }
}

impl Drop for JabRuntime {
    fn drop(&mut self) {
        // Post WM_QUIT to the message pump thread to exit the loop
        let tid = self.message_pump_thread_id;

        unsafe {
            let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
        }

        let curr_thread_id = unsafe { GetCurrentThreadId() };

        if tid != curr_thread_id {
            // Wait for the message pump thread to finish
            let handle = self.message_pump_handle.take();

            if let Some(h) = handle {
                let _ = h.join();
            }
        }
    }
}
