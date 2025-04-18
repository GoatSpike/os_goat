#![no_std] // Rust の標準ライブラリにリンクしない
#![no_main] // 全ての Rust レベルのエントリポイントを無効にする
#![feature(custom_test_frameworks)] // カスタムテストフレームワークを有効にする
#![test_runner(blog_os_goat::test_runner)] // テストランナーをカスタムのものに置き換える
#![reexport_test_harness_main = "test_main"] // テストハーネスのメイン関数をカスタムのものに置き換える

use core::panic::PanicInfo;
use blog_os_goat::println; // println!マクロをインポート
use bootloader::{BootInfo, entry_point}; 

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os_goat::memory::{self, BootInfoFrameAllocator};
    use x86_64::{VirtAddr, structures::paging::Page};

    println!("Hello World{}", "!");
    blog_os_goat::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // map an unused page
        let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));

    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
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