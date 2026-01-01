pub mod device;

use lazy_static::lazy_static;
use spin::Mutex;

pub use device::DummyDevice;

lazy_static! {
    pub static ref NETWORK: Mutex<Option<NetworkInfo>> = Mutex::new(None);
}

/// 간단한 네트워크 정보 구조 (smoltcp 문제로 인한 임시 구현)
pub struct NetworkInfo {
    pub ip_address: [u8; 4],
    pub mac_address: [u8; 6],
    pub status: &'static str,
}

impl NetworkInfo {
    pub fn new() -> Self {
        NetworkInfo {
            ip_address: [10, 0, 2, 15],
            mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01],
            status: "Ready (DummyDevice)",
        }
    }
}

/// 네트워크 스택 초기화
pub fn init() {
    use crate::serial_println;

    serial_println!("  Creating network info...");
    let info = NetworkInfo::new();
    *NETWORK.lock() = Some(info);
    serial_println!("  Network info initialized");
    serial_println!("  IP Address: 10.0.2.15/24");
    serial_println!("  MAC Address: 02:00:00:00:00:01");
    serial_println!("  Status: Ready");
}

/// 네트워크 폴링 (메인 루프에서 호출)
pub fn poll() {
    // 현재는 더미 구현
}
