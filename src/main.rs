#![no_std] // Rust の標準ライブラリにリンクしない
#![no_main] // 全ての Rust レベルのエントリポイントを無効にする
#![feature(custom_test_frameworks)] // カスタムテストフレームワークを有効にする
#![test_runner(blog_os_goat::test_runner)] // テストランナーをカスタムのものに置き換える
#![reexport_test_harness_main = "test_main"] // テストハーネスのメイン関数をカスタムのものに置き換える

mod vga_buffer;
mod serial;

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)] // この関数の名前修飾をしない
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    x86_64::instructions::interrupts::int3(); // ブレークポイントをトリガー

    #[cfg(test)]
    test_main(); // テストを実行

    loop {}
}

/// この関数はパニック時に呼ばれる
#[cfg(not(test))] // 新しく追加した属性
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// テストモードで使うパニックハンドラ
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os_goat::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}