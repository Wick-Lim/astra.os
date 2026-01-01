pub mod device;

use smoltcp::iface::{Config, Interface, SocketSet, SocketHandle};
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv4Address};
use smoltcp::time::Instant;
use smoltcp::socket::tcp;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::vec;

pub use device::DummyDevice;

lazy_static! {
    pub static ref NETWORK: Mutex<Option<NetworkStack>> = Mutex::new(None);
}

pub struct NetworkStack {
    pub interface: Interface,
    pub sockets: SocketSet<'static>,
    pub device: DummyDevice,
    pub tcp_active: bool,
    pub tcp_handle: SocketHandle,
}

impl NetworkStack {
    pub fn new() -> Self {
        // 더미 디바이스 생성
        let device = DummyDevice::new();

        // MAC 주소 설정
        let mac_addr = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);

        // 인터페이스 설정
        let config = Config::new(mac_addr.into());
        let mut interface = Interface::new(config, &mut device.clone(), Instant::from_millis(0));

        // IP 주소 설정 (10.0.2.15/24)
        interface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(Ipv4Address::new(10, 0, 2, 15).into(), 24))
                .ok();
        });

        // 소켓 세트 생성
        let mut sockets = SocketSet::new(Vec::new());

        // TCP 에코 서버 소켓 추가 (포트 7)
        let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
        let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
        let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = sockets.add(tcp_socket);

        NetworkStack {
            interface,
            sockets,
            device,
            tcp_active: false,
            tcp_handle,
        }
    }

    pub fn poll(&mut self, timestamp: Instant) {
        self.interface.poll(timestamp, &mut self.device, &mut self.sockets);

        // TCP 에코 서버 처리
        let socket = self.sockets.get_mut::<tcp::Socket>(self.tcp_handle);

        if !socket.is_open() && !self.tcp_active {
            // 포트 7에서 리슨 시작
            socket.listen(7).ok();
            self.tcp_active = true;
        }

        // TCP 에코: 받은 데이터를 임시 버퍼에 저장
        let mut echo_buffer = [0u8; 1024];
        let mut echo_len = 0;

        if socket.can_recv() {
            let _ = socket.recv(|buffer: &mut [u8]| {
                let len = buffer.len().min(1024);
                echo_buffer[..len].copy_from_slice(&buffer[..len]);
                echo_len = len;
                (len, ())
            });
        }

        // 받은 데이터를 에코
        if echo_len > 0 && socket.can_send() {
            let _ = socket.send_slice(&echo_buffer[..echo_len]);
        }
    }
}

/// 네트워크 스택 초기화
pub fn init() {
    use crate::serial_println;

    serial_println!("  Creating network stack...");
    let stack = NetworkStack::new();
    *NETWORK.lock() = Some(stack);
    serial_println!("  Network stack initialized");
    serial_println!("  IP Address: 10.0.2.15/24");
    serial_println!("  MAC Address: 02:00:00:00:00:01");
    serial_println!("  TCP Echo Server: Listening on port 7");
}

/// 네트워크 폴링 (메인 루프에서 호출)
pub fn poll() {
    if let Some(ref mut stack) = *NETWORK.lock() {
        // 타임스탬프는 간단하게 0으로 고정 (실제로는 시간을 가져와야 함)
        stack.poll(Instant::from_millis(0));
    }
}
