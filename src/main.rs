#![no_std] // Rust の標準ライブラリにリンクしない
#![no_main] // 全ての Rust レベルのエントリポイントを無効にする
#![feature(custom_test_frameworks)] // カスタムテストフレームワークを有効にする
#![test_runner(blog_os_goat::test_runner)] // テストランナーをカスタムのものに置き換える
#![reexport_test_harness_main = "test_main"] // テストハーネスのメイン関数をカスタムのものに置き換える

use core::panic::PanicInfo;
use blog_os_goat::println; // println!マクロをインポート

#[unsafe(no_mangle)] // この関数の名前修飾をしない
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os_goat::init(); // IDTを初期化

    use x86_64::registers::control::Cr3;

    let (level4_page_table, _) = Cr3::read();
    println!("Level 4 page table: {:?}", level4_page_table.start_address());

    let ptr = 0x204dd9 as *mut u8;
    unsafe {
        let x = *ptr;
    }
    println!("read worked");

    // unsafe {
    //     *ptr = 42; // 0xdeadbeafのアドレスに42を書き込む
    // }

    #[cfg(test)]
    test_main(); // テストを実行

    println!("It dit not crash!");
    blog_os_goat::hlt_loop();
}

/// この関数はパニック時に呼ばれる
#[cfg(not(test))] // 新しく追加した属性
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os_goat::hlt_loop();
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