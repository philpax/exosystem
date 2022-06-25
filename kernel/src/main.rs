#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(exosystem::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use exosystem::{allocator, memory};

entry_point!(kernel_main);

const HELLO_WORLD: &[u8] = include_bytes!("hello_world.wasm");

#[allow(clippy::print_literal)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    exosystem::init();

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let module = wasmi::Module::from_buffer(HELLO_WORLD).expect("failed to load wasm");
    let _instance = wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default())
        .expect("failed to instantiate wasm module")
        .run_start(&mut wasmi::NopExternals)
        .expect("failed to execute start");

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use exosystem::println;
    println!("{}", info);
    exosystem::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    exosystem::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
