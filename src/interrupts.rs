use crate::pic;
use crate::scheduler;

#[no_mangle]
pub extern "C" fn irq_timer_handler(frame: *mut usize) -> *mut usize {
    // send EOI for IRQ0
    pic::send_eoi(0);

    // let scheduler decide next task; pass current frame pointer
    let s = scheduler::get();
    let new = s.preempt(frame as usize);
    if new == 0 {
        core::ptr::null_mut()
    } else {
        new as *mut usize
    }
}
