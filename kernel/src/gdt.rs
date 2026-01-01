// Global Descriptor Table (GDT) for ASTRA.OS
// Uses bootloader's GDT with Ring 3 selectors

use x86_64::structures::gdt::SegmentSelector;

/// Initialize GDT (currently uses bootloader's GDT)
pub fn init() {
    crate::serial_println!("  Using bootloader's GDT with Ring 3 segments");
    crate::serial_println!("  GDT initialization complete");
}

/// Get user code segment selector (for Ring 3)
/// Bootloader GDT layout: 0=null, 1=kernel_code, 2=kernel_data, 3=TSS, 4=user_code, 5=user_data
/// User code selector: index 4 with RPL=3 => 0x23
pub fn user_code_selector() -> SegmentSelector {
    SegmentSelector::new(4, x86_64::PrivilegeLevel::Ring3)
}

/// Get user data segment selector (for Ring 3)
/// User data selector: index 5 with RPL=3 => 0x2b
pub fn user_data_selector() -> SegmentSelector {
    SegmentSelector::new(5, x86_64::PrivilegeLevel::Ring3)
}
