//! Thread-local last-error storage for the Node C ABI.
//!
//! Every exported function records a human-readable message here when it
//! returns a failure status. The string is owned by the library and valid
//! until the next ABI call on the same thread; callers must not free it.

use std::cell::RefCell;
use std::ffi::{CString, c_char};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

/// Records `message` as this thread's last error.
pub fn set<S: Into<String>>(message: S) {
    let message = message.into();
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = CString::new(message).ok();
    });
}

/// Borrows this thread's last error string, or `None` when the last call
/// succeeded (or never set one).
pub fn last() -> Option<String> {
    LAST_ERROR.with(|slot| slot.borrow().as_ref().map(|s| s.to_string_lossy().into_owned()))
}

/// C-ABI accessor: pointer to the thread's last error, or to an empty
/// string when none. The CString stays parked in the thread-local until the
/// next [`set`] on this thread, so the pointer remains valid across calls
/// per the ABI contract.
pub fn last_ptr() -> *const c_char {
    const EMPTY: &[u8] = b"\0";
    LAST_ERROR.with(|slot| {
        let guard = slot.borrow();
        match guard.as_ref() {
            Some(message) => message.as_ptr(),
            None => EMPTY.as_ptr().cast(),
        }
    })
}

