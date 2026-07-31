#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod memory;
mod scheduler;
mod timer;
mod vga;
mod pic;
mod pit;
mod idt;
mod interrupts;
// include assembly ISR stubs
core::arch::global_asm!(include_str!("interrupts.s"));

use vga::Writer;

const KERNEL_NAME: &str = "Jinn Kernel";
const KERNEL_VERSION: &str = "0.0.1";
const PRELOAD_ENABLED: bool = true;

/// Ticks between VGA status refreshes. Keeps the screen readable without
/// flooding the scroll buffer.
const DISPLAY_INTERVAL: u64 = 100;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut writer = Writer::new();
    writer.write_line("KERNEL PANIC — sistema parado.");
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut writer = Writer::new();
    writer.clear_screen();

    // ── Banner de boot ────────────────────────────────────────────────────────
    writer.write_string(KERNEL_NAME);
    writer.write_string(" v");
    writer.write_line(KERNEL_VERSION);
    writer.write_line("Inicializando...");

    // ── Configurações ─────────────────────────────────────────────────────────
    writer.write_line("Configuracoes:");
    if PRELOAD_ENABLED {
        writer.write_line("  [x] Pre-carregamento habilitado");
    } else {
        writer.write_line("  [ ] Pre-carregamento desabilitado");
    }
    writer.write_line("  [x] Configuracao dinamica desabilitada (zero overhead)");

    // ── Subsistemas ───────────────────────────────────────────────────────────
    memory::init();
    timer::init();

    // remap PIC and init PIT + IDT
    pic::remap();
    pit::init_hz(100);
    idt::init();

    // enable interrupts
    unsafe { core::arch::asm!("sti") };

    writer.write_string("Heap total : ");
    writer.write_decimal(memory::total_bytes());
    writer.write_line(" bytes");

    writer.write_string("Heap usado : ");
    writer.write_decimal(memory::used_bytes());
    writer.write_line(" bytes");

    // ── Scheduler ─────────────────────────────────────────────────────────────
    let scheduler = scheduler::get();

    fn task_loader() {
        let mut w = Writer::new();
        w.write_line("[Loader] iniciando...");
        w.write_line("[Loader] carregando recursos...");
        // yield back to scheduler so other tasks may run
        crate::scheduler::yield_now();
    }

    fn task_worker() {
        let mut w = Writer::new();
        w.write_line("[Worker] iniciando trabalho...");
        for i in 0..3 {
            w.write_string("[Worker] step ");
            w.write_decimal(i);
            w.write_line("");
            crate::scheduler::yield_now();
        }
    }

    fn task_monitor() {
        let mut w = Writer::new();
        w.write_line("[Monitor] verificando status...");
        crate::scheduler::yield_now();
    }

    scheduler.add_task("Loader", task_loader);
    scheduler.add_task("Worker", task_worker);
    scheduler.add_task("Monitor", task_monitor);

    writer.write_line("");
    scheduler.print_status(&mut writer);
    writer.write_line("");
    writer.write_line("Boot completo. Entrando no loop principal.");
    writer.write_line("----------------------------------------");

    // ── Loop principal ────────────────────────────────────────────────────────
    // Only print a status line every DISPLAY_INTERVAL ticks to avoid an
    // unreadable infinite scroll. Between prints we still run the scheduler
    // every iteration so the round-robin cadence is correct.
    let mut last_display: u64 = 0;

    loop {
        timer::tick();
        let ticks = timer::ticks();

        // Schedule the next task every tick.
        let _ = scheduler.run_next();

        // Only update the display at the configured interval.
        if ticks - last_display >= DISPLAY_INTERVAL {
            last_display = ticks;

            writer.write_string("Tick: ");
            writer.write_decimal(ticks as usize);
            writer.write_string("  |  Mem usada: ");
            writer.write_decimal(memory::used_bytes());
            writer.write_line(" bytes");
        }

        // Spin-delay acting as a crude time quantum.
        // TODO: replace with a hardware timer interrupt (PIT/APIC).
        for _ in 0..1_000_000 {
            core::hint::spin_loop();
        }
    }
}
