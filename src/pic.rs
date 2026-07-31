// Minimal PIC remapping and EOI support
pub const PIC1: u16 = 0x20;
pub const PIC2: u16 = 0xA0;
pub const PIC1_CMD: u16 = PIC1;
pub const PIC1_DATA: u16 = PIC1 + 1;
pub const PIC2_CMD: u16 = PIC2;
pub const PIC2_DATA: u16 = PIC2 + 1;

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

unsafe fn inb(port: u16) -> u8 {
    let mut val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port);
    val
}

pub fn remap() {
    unsafe {
        let a1 = inb(PIC1_DATA);
        let a2 = inb(PIC2_DATA);

        // start init sequence in cascade mode
        outb(PIC1_CMD, 0x11);
        outb(PIC2_CMD, 0x11);

        // set vector offset: remap PIC1->0x20, PIC2->0x28
        outb(PIC1_DATA, 0x20);
        outb(PIC2_DATA, 0x28);

        // tell PIC about cascading
        outb(PIC1_DATA, 4);
        outb(PIC2_DATA, 2);

        // set environment info
        outb(PIC1_DATA, 1);
        outb(PIC2_DATA, 1);

        // restore saved masks
        outb(PIC1_DATA, a1);
        outb(PIC2_DATA, a2);
    }
}

pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_CMD, 0x20);
        }
        outb(PIC1_CMD, 0x20);
    }
}
