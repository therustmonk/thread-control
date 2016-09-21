//! Library to control thread execution.
//!
//! Usage example:
//!
//! ```rust
//! use std::thread;
//! use thread_control::*;
//!
//! fn main() {
//!     let (flag, control) = make_pair();
//!     let handle = thread::spawn(move || {
//!         while flag.alive() {
//!         }
//!     });
//!     assert_eq!(control.is_done(), false);
//!     control.stop();
//!     handle.join();
//!     assert_eq!(control.is_interrupted(), false);
//!     assert_eq!(control.is_done(), true);
//! }
//! ```
//!
//! Interrupt example:
//!
//! ```rust
//! use std::thread;
//! use thread_control::*;
//!
//! fn main() {
//!     let (flag, control) = make_pair();
//!     let handle = thread::spawn(move || {
//!         while flag.alive() {
//!         }
//!     });
//!     control.interrupt();
//!     handle.join();
//!     assert_eq!(control.is_interrupted(), true);
//!     assert_eq!(control.is_done(), true);
//! }
//! ```
//!
//! Panics example:
//!
//! ```rust
//! use std::thread;
//! use thread_control::*;
//!
//! fn main() {
//!     let (flag, control) = make_pair();
//!     let handle = thread::spawn(move || {
//!         while flag.alive() {
//!             panic!("PANIC!");
//!         }
//!     });
//!     handle.join();
//!     assert_eq!(control.is_interrupted(), true);
//!     assert_eq!(control.is_done(), true);
//! }
//! ```
//!

use std::thread;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

/// Struct to check execution status of spawned thread.
pub struct Flag {
    alive: Arc<AtomicBool>,
    interrupt: Arc<AtomicBool>,
}

impl Drop for Flag {
    fn drop(&mut self) {
        if thread::panicking() {
            (*self.interrupt).store(true, Ordering::Relaxed)
        }
    }
}

impl Flag {

    /// Creates new flag.
    pub fn new() -> Self {
        Flag {
            alive: Arc::new(AtomicBool::new(true)),
            interrupt: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates new `Control` to control this flag.
    pub fn take_control(&self) -> Control {
        Control {
            alive: Arc::downgrade(&self.alive),
            interrupt: self.interrupt.clone(),
        }
    }

    /// Check the flag isn't stopped or interrupted.
    pub fn alive(&self) -> bool {
        if (*self.interrupt).load(Ordering::Relaxed) {
            panic!("thread interrupted by thread-contol");
        }
        (*self.alive).load(Ordering::Relaxed)
    }
}

/// Struct to control thread execution.
pub struct Control {
    alive: Weak<AtomicBool>,
    interrupt: Arc<AtomicBool>,
}

impl Control {
    /// Interrupt execution of thread.
    /// Actually it panics when thread checking flag.
    pub fn interrupt(&self) {
        (*self.interrupt).store(true, Ordering::Relaxed)
    }

    /// Set stop flag.
    pub fn stop(&self) {
        self.alive.upgrade().map(|flag| {
            (*flag).store(false, Ordering::Relaxed)
        });
    }

    /// Return `true` if thread ended.
    pub fn is_done(&self) -> bool {
        self.alive.upgrade().is_none()
    }

    /// Return `true` if thread was interrupted or panicked.
    pub fn is_interrupted(&self) -> bool {
        (*self.interrupt).load(Ordering::Relaxed)
    }
}

/// Makes pair with connected flag and control.
pub fn make_pair() -> (Flag, Control) {
    let flag = Flag::new();
    let control = flag.take_control();
    (flag, control)
}

