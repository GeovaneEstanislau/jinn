use core::fmt;
use core::ptr::write_volatile;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new() -> Writer {
        let buffer = unsafe { &mut *(0xb8000 as *mut Buffer) };
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::LightGreen, Color::Black),
            buffer,
        }
    }

    /// Change the active foreground/background color for subsequent writes.
    #[allow(dead_code)] // used by future color-coded kernel messages
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }

    pub fn write_line(&mut self, s: &str) {
        self.write_string(s);
        self.write_byte(b'\n');
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_decimal(&mut self, mut value: usize) {
        if value == 0 {
            self.write_byte(b'0');
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 0;
        while value > 0 {
            buf[i] = b'0' + ((value % 10) as u8);
            value /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            self.write_byte(buf[i]);
        }
    }

    /// Write a string at a **fixed screen position** (row, col) without moving
    /// the scroll cursor. Pads the remaining cells up to `pad_width` total
    /// columns (from `col`) with spaces, erasing stale characters from
    /// previous renders.
    ///
    /// This is the building block for the kernel status bar — any write here
    /// survives until overwritten again, because it bypasses `new_line()`.
    #[allow(dead_code)] // foundation for the kernel status bar (no-scroll display)
    pub fn write_at(&mut self, row: usize, col: usize, s: &str, pad_width: usize) {
        let row = row.min(BUFFER_HEIGHT - 1);
        let mut c = col;
        for byte in s.bytes() {
            if c >= BUFFER_WIDTH {
                break;
            }
            let ch = match byte {
                0x20..=0x7e => byte,
                _ => 0xfe,
            };
            unsafe {
                write_volatile(
                    &mut self.buffer.chars[row][c].ascii_character,
                    ch,
                );
                write_volatile(
                    &mut self.buffer.chars[row][c].color_code,
                    self.color_code,
                );
            }
            c += 1;
        }
        // Pad remaining cells with spaces so previous (longer) values are erased.
        let end = (col + pad_width).min(BUFFER_WIDTH);
        while c < end {
            unsafe {
                write_volatile(
                    &mut self.buffer.chars[row][c].ascii_character,
                    b' ',
                );
                write_volatile(
                    &mut self.buffer.chars[row][c].color_code,
                    self.color_code,
                );
            }
            c += 1;
        }
    }

    /// Write a decimal number at a **fixed screen position** without scrolling.
    /// `pad_width` works the same as in `write_at`.
    #[allow(dead_code)] // foundation for the kernel status bar (no-scroll display)
    pub fn write_decimal_at(&mut self, row: usize, col: usize, mut value: usize, pad_width: usize) {
        let row = row.min(BUFFER_HEIGHT - 1);
        let mut buf = [0u8; 20];
        let mut i = 0;
        if value == 0 {
            buf[0] = b'0';
            i = 1;
        } else {
            while value > 0 {
                buf[i] = b'0' + (value % 10) as u8;
                value /= 10;
                i += 1;
            }
            // Reverse digits to get correct order.
            buf[..i].reverse();
        }
        let mut c = col;
        for &byte in &buf[..i] {
            if c >= BUFFER_WIDTH {
                break;
            }
            unsafe {
                write_volatile(
                    &mut self.buffer.chars[row][c].ascii_character,
                    byte,
                );
                write_volatile(
                    &mut self.buffer.chars[row][c].color_code,
                    self.color_code,
                );
            }
            c += 1;
        }
        // Pad remaining cells.
        let end = (col + pad_width).min(BUFFER_WIDTH);
        while c < end {
            unsafe {
                write_volatile(
                    &mut self.buffer.chars[row][c].ascii_character,
                    b' ',
                );
                write_volatile(
                    &mut self.buffer.chars[row][c].color_code,
                    self.color_code,
                );
            }
            c += 1;
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                unsafe {
                    write_volatile(
                        &mut self.buffer.chars[row][col].ascii_character,
                        byte,
                    );
                    write_volatile(
                        &mut self.buffer.chars[row][col].color_code,
                        self.color_code,
                    );
                }
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col];
                unsafe {
                    write_volatile(&mut self.buffer.chars[row - 1][col], character);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            unsafe {
                write_volatile(&mut self.buffer.chars[row][col], blank);
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

