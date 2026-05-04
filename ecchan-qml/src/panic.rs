use std::{panic, sync::Once};

use crate::q_fatal;

pub fn set_hook() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        panic::set_hook(Box::new(|info| {
            q_fatal!("[PANIC] {info}");
        }));
    });
}
