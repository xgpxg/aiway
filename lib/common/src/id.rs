use idgenerator::{IdGeneratorOptions, IdInstance};

// 2025-01-01 00:00:00
const BASE_TIME: i64 = 1735660800000;

pub fn init() {
    let options = IdGeneratorOptions::new()
        .base_time(BASE_TIME)
        .worker_id(1)
        .worker_id_bit_len(2);
    let _ = IdInstance::init(options).unwrap();
}

pub fn next() -> i64 {
    IdInstance::next_id()
}
