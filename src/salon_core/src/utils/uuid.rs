use std::sync::atomic::{AtomicU32, Ordering};

pub type Uuid = u32;

static NEXT_UUID: AtomicU32 = AtomicU32::new(0);

pub fn get_next_uuid() -> Uuid {
    let result = NEXT_UUID.fetch_add(1, Ordering::Relaxed);
    result
}
