use volatile::Volatile; // メモリの読み書きを保証するための型
use core::fmt; // フォーマット関連のトレイトをインポート
use lazy_static::lazy_static; // 遅延初期化を行うためのマクロ
use spin::Mutex; // スピンロックを提供するミューテックス

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    // VGAテキストモードで使用可能な色を定義
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8); // 前景色と背景色を格納する構造体

impl ColorCode {
    // 新しいColorCodeを生成する関数
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    // VGAバッファ内の1文字を表す構造体
    ascii_character: u8, // ASCII文字
    color_code: ColorCode, // 色コード
}

const BUFFER_HEIGHT: usize = 25; // VGAバッファの高さ
const BUFFER_WIDTH: usize = 80; // VGAバッファの幅

#[repr(transparent)]
struct Buffer {
    // VGAバッファ全体を表す構造体
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], // 文字の2次元配列
}

pub struct Writer {
    // 文字列を書き込むための構造体
    column_position: usize, // 現在の列位置
    color_code: ColorCode, // 使用する色コード
    buffer: &'static mut Buffer, // VGAバッファへの参照
}

lazy_static! {
    // グローバルなWriterインスタンスを定義
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black), // デフォルトの色設定
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, // VGAバッファのメモリアドレス
    });
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        // 1バイトを書き込む関数
        match byte {
            b'\n' => self.new_line(), // 改行の場合は新しい行を作成
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line(); // 行がいっぱいの場合は改行
                }
                let low = BUFFER_HEIGHT - 1; // 最下行
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[low][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1; // 列位置を更新
            }
        }
    }

    fn new_line(&mut self) {
        // 新しい行を作成する関数
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character); // 上の行にコピー
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1); // 最下行をクリア
        self.column_position = 0; // 列位置をリセット
    }

    fn clear_row(&mut self, row: usize) {
        // 指定した行を空白でクリアする関数
        let blank = ScreenChar {
            ascii_character: b' ', // 空白文字
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        // 文字列を書き込む関数
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte), // 出力可能なASCIIバイトまたは改行
                _ => self.write_byte(0xfe), // 出力不可能なバイトは「■」で代用
            }
        }
    }
}

impl fmt::Write for Writer {
    // フォーマットされた文字列を書き込むためのトレイト実装
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    // println!マクロの定義
    () => {
     ($crate::print!("\n"));   
    };

    ($($arg:tt)*) => {
        ($crate::print!("{}\n", format_args!($($arg)*)));
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    })
}