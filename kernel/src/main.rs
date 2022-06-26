#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(exosystem::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use exosystem::{allocator, memory, print};
use wasmi::{
    Externals, FuncInstance, MemoryRef, ModuleImportResolver, RuntimeArgs, RuntimeValue, Signature,
    Trap, TrapKind, ValueType,
};

entry_point!(kernel_main);

const HELLO_WORLD: &[u8] = include_bytes!("hello_world.wasm");

type WasmResult = Result<Option<RuntimeValue>, Trap>;
const FD_WRITE_FUNC_INDEX: usize = 0;

struct Runtime {
    memory: Option<MemoryRef>,
}
impl Runtime {
    pub fn memory(&self) -> &MemoryRef {
        self.memory.as_ref().unwrap()
    }
    pub fn memory_mut(&mut self) -> &mut MemoryRef {
        self.memory.as_mut().unwrap()
    }

    fn fd_write(&mut self, args: RuntimeArgs) -> WasmResult {
        let fd: u32 = args.nth(0);
        let iovs: u32 = args.nth(1);
        let iov_lens: u32 = args.nth(2);
        let written: u32 = args.nth(3);

        let mut bytes = 0;

        if fd != 1 {
            return Err(Trap::new(TrapKind::Unreachable));
        }

        for i in 0..iov_lens {
            let iov_ptr = iovs + i * 8;

            let iov_base: u32 = self
                .memory()
                .get_value(iov_ptr + 0)
                .map_err(|_| Trap::new(TrapKind::MemoryAccessOutOfBounds))?;
            let iov_len: u32 = self
                .memory()
                .get_value(iov_ptr + 4)
                .map_err(|_| Trap::new(TrapKind::MemoryAccessOutOfBounds))?;

            let iov_start = iov_base as usize;
            let iov_end = (iov_base + iov_len) as usize;

            self.memory().with_direct_access(|mem| {
                let payload = core::str::from_utf8(&mem[iov_start..iov_end])
                    .map_err(|_| Trap::new(TrapKind::Unreachable))?;
                print!("{}", payload);
                Ok::<(), Trap>(())
            })?;

            bytes += iov_len;
        }

        self.memory_mut()
            .set_value(written, bytes)
            .map_err(|_| Trap::new(TrapKind::MemoryAccessOutOfBounds))?;

        Ok(Some(RuntimeValue::I32(bytes as i32)))
    }
}
impl Externals for Runtime {
    fn invoke_index(&mut self, index: usize, args: RuntimeArgs) -> WasmResult {
        match index {
            FD_WRITE_FUNC_INDEX => self.fd_write(args),
            _ => panic!("unknown function index"),
        }
    }
}

struct WasiSnapshotPreview1Resolver;
impl ModuleImportResolver for WasiSnapshotPreview1Resolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        match field_name {
            "fd_write" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    Some(ValueType::I32),
                ),
                FD_WRITE_FUNC_INDEX,
            )),
            _ => Err(wasmi::Error::Instantiation(alloc::format!(
                "Export {} not found",
                field_name
            ))),
        }
    }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    exosystem::init();

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let module = wasmi::Module::from_buffer(HELLO_WORLD).expect("failed to load wasm");
    let mut runtime = Runtime { memory: None };
    let instance = wasmi::ModuleInstance::new(
        &module,
        &wasmi::ImportsBuilder::default()
            .with_resolver("wasi_snapshot_preview1", &WasiSnapshotPreview1Resolver),
    )
    .expect("failed to instantiate wasm module")
    .run_start(&mut runtime)
    .expect("failed to execute start");

    runtime.memory = Some(
        instance
            .export_by_name("memory")
            .expect("Expected export with a name 'memory'")
            .as_memory()
            .expect("'memory' should be a memory instance")
            .clone(),
    );
    instance.invoke_export("main", &[], &mut runtime).unwrap();

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    exosystem::println!("{}", info);
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
