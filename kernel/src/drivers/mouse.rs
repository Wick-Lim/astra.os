use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

/// PS/2 마우스 데이터 포트
const MOUSE_DATA_PORT: u16 = 0x60;
/// PS/2 마우스 커맨드 포트
const MOUSE_COMMAND_PORT: u16 = 0x64;

#[derive(Debug, Clone, Copy)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

impl MouseState {
    pub const fn new() -> Self {
        MouseState {
            x: 40,  // 화면 중앙
            y: 12,
            left_button: false,
            right_button: false,
            middle_button: false,
        }
    }
}

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

pub struct Mouse {
    data_port: Port<u8>,
    command_port: Port<u8>,
    state: MouseState,
    packet: [u8; 3],
    packet_index: usize,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            data_port: Port::new(MOUSE_DATA_PORT),
            command_port: Port::new(MOUSE_COMMAND_PORT),
            state: MouseState::new(),
            packet: [0; 3],
            packet_index: 0,
        }
    }

    /// 마우스 초기화
    pub fn init(&mut self) {
        // QEMU에서 PS/2 마우스가 없을 수 있으므로 간단하게 처리
        // 실제 하드웨어에서는 더 robust한 초기화가 필요할 수 있음
    }

    /// 커맨드 포트에 쓰기 가능할 때까지 대기
    fn wait_write(&mut self) {
        for _ in 0..1000 {
            unsafe {
                if (self.command_port.read() & 0x02) == 0 {
                    return;
                }
            }
        }
    }

    /// 데이터 포트에서 읽기 가능할 때까지 대기
    fn wait_read(&mut self) {
        for _ in 0..1000 {
            unsafe {
                if (self.command_port.read() & 0x01) != 0 {
                    return;
                }
            }
        }
    }

    /// 마우스에 커맨드 전송
    fn write_mouse(&mut self, value: u8) {
        unsafe {
            self.wait_write();
            self.command_port.write(0xD4);
            self.wait_write();
            self.data_port.write(value);
        }
    }

    /// 마우스에서 데이터 읽기
    fn read_mouse(&mut self) -> u8 {
        unsafe {
            self.wait_read();
            self.data_port.read()
        }
    }

    /// 마우스 인터럽트 처리
    pub fn handle_interrupt(&mut self) {
        unsafe {
            let packet_byte = self.data_port.read();

            // 3바이트 패킷 수집
            self.packet[self.packet_index] = packet_byte;
            self.packet_index += 1;

            if self.packet_index == 3 {
                self.process_packet();
                self.packet_index = 0;
            }
        }
    }

    /// 3바이트 패킷 처리
    fn process_packet(&mut self) {
        let flags = self.packet[0];
        let x_movement = self.packet[1] as i32;
        let y_movement = self.packet[2] as i32;

        // 버튼 상태 업데이트
        self.state.left_button = (flags & 0x01) != 0;
        self.state.right_button = (flags & 0x02) != 0;
        self.state.middle_button = (flags & 0x04) != 0;

        // X 이동 처리 (부호 확장)
        let x_sign = (flags & 0x10) != 0;
        let x_overflow = (flags & 0x40) != 0;
        if !x_overflow {
            let x_delta = if x_sign {
                x_movement - 256
            } else {
                x_movement
            };
            self.state.x = (self.state.x + x_delta).clamp(0, 79);
        }

        // Y 이동 처리 (부호 확장, Y축은 반대)
        let y_sign = (flags & 0x20) != 0;
        let y_overflow = (flags & 0x80) != 0;
        if !y_overflow {
            let y_delta = if y_sign {
                y_movement - 256
            } else {
                y_movement
            };
            self.state.y = (self.state.y - y_delta).clamp(0, 24);
        }
    }

    pub fn get_state(&self) -> MouseState {
        self.state
    }
}

/// 마우스 초기화
pub fn init() {
    MOUSE.lock().init();
}

/// 마우스 인터럽트 핸들러
pub fn handle_interrupt() {
    MOUSE.lock().handle_interrupt();
}

/// 마우스 상태 가져오기
pub fn get_state() -> MouseState {
    MOUSE.lock().get_state()
}
