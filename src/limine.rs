/// Minimal Limine boot protocol support.
///
/// Limine requires at least the **base revision tag** to be present and
/// loaded in the kernel binary. With only this tag, Limine will:
///
///   - Load the kernel ELF into memory
///   - Switch the CPU to 64-bit long mode
///   - Set up a flat GDT (null, 64-bit code, 64-bit data)
///   - Configure a valid stack (RSP)
///   - Disable interrupts (IF = 0)
///   - Jump to `_start`
///
/// No other requests are required for a minimal VGA text kernel.
/// Future requests (memory map, HHDM, framebuffer) should be added
/// as dedicated modules in this file when needed.
///
/// # Protocol revision
/// Revision 0 is accepted by every Limine version ≥ v2.
/// Bump to 1 or 2 when you start relying on newer protocol features.

/// Limine base revision tag.
///
/// The three-word layout [id0, id1, revision] is defined by the Limine
/// protocol spec. The first two words are magic identifiers; the third
/// is the minimum revision the kernel requires from the bootloader.
///
/// `#[used]` prevents the compiler from dead-code-eliminating this
/// static. The `.requests` section is KEEP'd in the linker script so
/// the linker does not strip it either.
#[used]
#[link_section = ".requests"]
static LIMINE_BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8, // magic[0]
    0x6a7b384944536bdc, // magic[1]
    0,                  // revision = 0 (compatible with all Limine ≥ v2)
];
