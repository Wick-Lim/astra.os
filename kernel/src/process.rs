// Process management for ASTRA.OS
// Enables userspace execution with Ring 3 privilege level

use alloc::vec::Vec;
use alloc::boxed::Box;
use x86_64::structures::paging::{PageTable, PhysFrame};
use x86_64::VirtAddr;

/// Process ID type
pub type Pid = u64;

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// CPU register state for context switching
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RegisterState {
    // General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Instruction pointer
    pub rip: u64,

    // Flags
    pub rflags: u64,

    // Segment selectors
    pub cs: u64,
    pub ss: u64,
}

impl Default for RegisterState {
    fn default() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0,
            rflags: 0x202, // IF (interrupt enable) flag set
            cs: 0x08,  // Kernel code segment (will be changed for userspace)
            ss: 0x10,  // Kernel data segment
        }
    }
}

/// Process Control Block (PCB)
pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    pub registers: RegisterState,

    // Memory management
    pub page_table: Box<PageTable>,

    // Stack pointer (virtual address)
    pub stack_pointer: VirtAddr,

    // Entry point
    pub entry_point: VirtAddr,
}

impl Process {
    /// Create a new process
    pub fn new(pid: Pid, entry_point: VirtAddr) -> Self {
        // Allocate new page table for process
        // TODO: Actually allocate and set up page table
        let page_table = Box::new(PageTable::new());

        Self {
            pid,
            state: ProcessState::Ready,
            registers: RegisterState::default(),
            page_table,
            stack_pointer: VirtAddr::new(0x7FFF_FFFF_0000), // User stack top
            entry_point,
        }
    }
}

/// Process scheduler
pub struct Scheduler {
    processes: Vec<Process>,
    current_pid: Option<Pid>,
    next_pid: Pid,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            current_pid: None,
            next_pid: 1,
        }
    }

    /// Spawn a new process
    pub fn spawn(&mut self, entry_point: VirtAddr) -> Pid {
        let pid = self.next_pid;
        self.next_pid += 1;

        let mut process = Process::new(pid, entry_point);
        process.registers.rip = entry_point.as_u64();
        process.registers.rsp = process.stack_pointer.as_u64();

        // Set user mode segments (Ring 3)
        process.registers.cs = 0x1B; // User code segment with RPL=3
        process.registers.ss = 0x23; // User data segment with RPL=3

        self.processes.push(process);
        pid
    }

    /// Get current running process
    pub fn current_process(&self) -> Option<&Process> {
        self.current_pid.and_then(|pid| {
            self.processes.iter().find(|p| p.pid == pid)
        })
    }

    /// Get current running process (mutable)
    pub fn current_process_mut(&mut self) -> Option<&mut Process> {
        self.current_pid.and_then(|pid| {
            self.processes.iter_mut().find(|p| p.pid == pid)
        })
    }

    /// Simple round-robin scheduler
    pub fn schedule(&mut self) -> Option<&Process> {
        if self.processes.is_empty() {
            return None;
        }

        // Find next ready process
        let current_idx = self.current_pid
            .and_then(|pid| self.processes.iter().position(|p| p.pid == pid))
            .unwrap_or(0);

        for i in 0..self.processes.len() {
            let idx = (current_idx + i + 1) % self.processes.len();
            if self.processes[idx].state == ProcessState::Ready {
                self.processes[idx].state = ProcessState::Running;
                self.current_pid = Some(self.processes[idx].pid);
                return Some(&self.processes[idx]);
            }
        }

        None
    }
}

use spin::Mutex;

/// Global scheduler instance
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
