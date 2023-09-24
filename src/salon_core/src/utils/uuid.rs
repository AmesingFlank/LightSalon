use std::sync::atomic::{AtomicU32, Ordering};

static next_uuid: AtomicU32 = AtomicU32::new(0);

pub fn get_next_uuid() -> u32 {
    let result = next_uuid.fetch_add(1, Ordering::Relaxed);
    result
}