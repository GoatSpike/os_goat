#![no_std] // Rust の標準ライブラリにリンクしない
#![no_main] // 全ての Rust レベルのエントリポイントを無効にする
mod vga_buffer;

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)] // この関数の名前修飾をしない
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    loop {}
}

/// この関数はパニック時に呼ばれる
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}