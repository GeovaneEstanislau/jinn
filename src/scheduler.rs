use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use crate::vga::Writer;

const MAX_TASKS: usize = 8;
const STACK_SIZE: usize = 4096;

static CURRENT_TASK: AtomicI32 = AtomicI32::new(-1);
static YIELDED: AtomicBool = AtomicBool::new(false);

/// Tasks should call `scheduler::yield_now()` to return to the scheduler and
/// allow it to run the next ready task. This is cooperative yielding — the
/// task implementation is responsible for maintaining its own state to
/// continue on the next activation.
pub fn yield_now() {
    YIELDED.store(true, Ordering::SeqCst);
}

pub fn current_task_index() -> i32 {
    CURRENT_TASK.load(Ordering::SeqCst)
}

pub struct Task {
    pub id: usize,
    pub name: &'static str,
    pub state: TaskState,
    pub stack_top: usize,
    pub entry: Option<fn()>,
    pub saved_rsp: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Completed,
}

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    stacks: [[u8; STACK_SIZE]; MAX_TASKS],
    next_id: usize,
    current: Option<usize>,
}

// Global scheduler instance for IRQ handlers to access.
pub static mut SCHEDULER: Scheduler = Scheduler::new();

pub fn get() -> &'static mut Scheduler {
    unsafe { &mut SCHEDULER }
}

impl Scheduler {
    pub const fn new() -> Self {
        const NONE_TASK: Option<Task> = None;
        Scheduler {
            tasks: [NONE_TASK; MAX_TASKS],
            stacks: [[0u8; STACK_SIZE]; MAX_TASKS],
            next_id: 0,
            current: None,
        }
    }

    pub fn add_task(&mut self, name: &'static str, entry: fn()) -> usize {
        let mut id = self.next_id;
        while id < MAX_TASKS {
            if self.tasks[id].is_none() {
                // compute stack top (aligned)
                let stack_ptr = self.stacks[id].as_ptr() as usize;
                let mut top = stack_ptr + STACK_SIZE;
                top &= !15; // align to 16
                self.tasks[id] = Some(Task {
                    id,
                    name,
                    state: TaskState::Ready,
                    stack_top: top,
                    entry: Some(entry),
                    saved_rsp: 0,
                });
                self.next_id = id + 1;
                return id;
            }
            id += 1;
        }
        0
    }

    pub fn run_next(&mut self) -> Option<usize> {
        let mut start = self.current.unwrap_or(usize::MAX);
        for _ in 0..MAX_TASKS {
            start = if start == usize::MAX { 0 } else { (start + 1) % MAX_TASKS };
            if let Some(task) = &mut self.tasks[start] {
                if task.state == TaskState::Ready {
                    self.current = Some(start);
                    task.state = TaskState::Running;
                    // set current task index for yield() to use
                    CURRENT_TASK.store(start as i32, Ordering::SeqCst);
                    YIELDED.store(false, Ordering::SeqCst);
                    // run the task on its own stack
                    if let Some(entry) = task.entry {
                        unsafe { run_on_stack(task.stack_top, entry) }
                    }
                    // after returning, check whether it yielded
                    if YIELDED.load(Ordering::SeqCst) {
                        // task yielded and wants to continue later
                        task.state = TaskState::Ready;
                        YIELDED.store(false, Ordering::SeqCst);
                    } else {
                        task.state = TaskState::Completed;
                    }
                    CURRENT_TASK.store(-1, Ordering::SeqCst);
                    return Some(start);
                }
            }
        }
        None
    }

    pub fn preempt(&mut self, curr_frame_ptr: usize) -> usize {
        // Save current frame pointer into current task if any
        if let Some(cur) = self.current {
            if let Some(task) = &mut self.tasks[cur] {
                task.saved_rsp = curr_frame_ptr;
                task.state = TaskState::Ready;
            }
        }

        // pick next ready task
        let mut start = self.current.unwrap_or(usize::MAX);
        for _ in 0..MAX_TASKS {
            start = if start == usize::MAX { 0 } else { (start + 1) % MAX_TASKS };
            if let Some(task) = &mut self.tasks[start] {
                if task.state == TaskState::Ready {
                    // prepare initial frame if needed
                    if task.saved_rsp == 0 {
                        // build initial stack frame so iretq returns to entry
                        let stack_ptr = self.stacks[start].as_ptr() as usize;
                        let mut sp = stack_ptr + STACK_SIZE;
                        sp &= !15;
                        // push registers (as zeros) in the same order the ISR pushes them
                        for _ in 0..15 {
                            sp -= 8;
                            unsafe { (sp as *mut usize).write_volatile(0); }
                        }
                        let entry_addr = task.entry.unwrap() as usize;
                        // push RIP, CS, RFLAGS (so iretq will return to entry)
                        sp -= 8; unsafe { (sp as *mut usize).write_volatile(entry_addr); }
                        sp -= 8; unsafe { (sp as *mut usize).write_volatile(0x08); }
                        sp -= 8; unsafe { (sp as *mut usize).write_volatile(0x202); }
                        task.saved_rsp = sp + 8*0; // pointer to first pushed reg (rax) which is current sp + size of pushed regs? but we constructed regs first, so compute
                        // actually, after pushing regs, reg_ptr = stack_ptr + STACK_SIZE - 8*15
                        task.saved_rsp = stack_ptr + STACK_SIZE - 8*15;
                    }

                    self.current = Some(start);
                    task.state = TaskState::Running;
                    CURRENT_TASK.store(start as i32, Ordering::SeqCst);
                    return task.saved_rsp;
                }
            }
        }

        // no switch
        curr_frame_ptr
    }

    pub fn print_status(&self, writer: &mut Writer) {
        writer.write_line("Scheduler status:");
        for opt in self.tasks.iter().flatten() {
            writer.write_string("  task ");
            writer.write_decimal(opt.id);
            writer.write_string(": ");
            writer.write_string(opt.name);
            writer.write_string(" ");
            writer.write_string(match opt.state {
                TaskState::Ready => "Ready",
                TaskState::Running => "Running",
                TaskState::Completed => "Completed",
            });
            writer.write_line("");
        }
    }
}

unsafe fn run_on_stack(stack_top: usize, func: fn()) {
    let mut old_rsp: usize = 0;
    core::arch::asm!(
        "mov [rdi], rsp",
        "mov rsp, rsi",
        "call rdx",
        "mov rsp, [rdi]",
        in("rdi") &mut old_rsp,
        in("rsi") stack_top,
        in("rdx") func as usize,
        options(nostack)
    );
}
