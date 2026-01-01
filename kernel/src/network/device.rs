use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::VecDeque;
use spin::Mutex;

/// 더미 네트워크 디바이스 (테스트용)
#[derive(Clone)]
pub struct DummyDevice {
    mtu: usize,
}

impl DummyDevice {
    pub fn new() -> Self {
        DummyDevice { mtu: 1500 }
    }
}

impl Device for DummyDevice {
    type RxToken<'a> = DummyRxToken where Self: 'a;
    type TxToken<'a> = DummyTxToken where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // 더미 구현 - 패킷을 받지 않음
        None
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        // 더미 구현 - 패킷 전송 가능
        Some(DummyTxToken)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = self.mtu;
        caps.medium = Medium::Ethernet;
        caps
    }
}

pub struct DummyRxToken;

impl RxToken for DummyRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = [0u8; 1500];
        f(&mut buffer)
    }
}

pub struct DummyTxToken;

impl TxToken for DummyTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);
        // 더미 구현 - 실제로 전송하지 않음
        result
    }
}

/// 루프백 네트워크 디바이스 (향후 구현)
pub struct LoopbackDevice {
    queue: Mutex<VecDeque<Vec<u8>>>,
    mtu: usize,
}

impl LoopbackDevice {
    pub fn new() -> Self {
        LoopbackDevice {
            queue: Mutex::new(VecDeque::new()),
            mtu: 1500,
        }
    }

    pub fn enqueue(&self, packet: Vec<u8>) {
        self.queue.lock().push_back(packet);
    }
}
