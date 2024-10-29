use alloc::vec::Vec;
use sbi_spec::binary::SbiRet;
use spin::{Once, RwLock};
use crate::vcpu::VCpu;
use crate::enclave_id::EnclaveId;
use fast_trap::FlowContext;
use riscv::register::*;

// Remember to change this value together with RustSBI's prototyper/src/trap_stack.rs
const NUM_HART_MAX: usize = 8;

struct Enclave {
    eid: EnclaveId,
    host_vcpu: VCpu,
    vcpus: Vec<VCpu>,
}

impl Enclave {
    pub fn new(start_pc: usize) -> Self {
        let mut vcpus = Vec::with_capacity(NUM_HART_MAX);
        vcpus.push(VCpu::init_context(start_pc));
        Enclave {
            eid: EnclaveId::Encl(1),  // TODO: alocated by the allocator
            host_vcpu: VCpu::init_context(0x0),
            vcpus: vcpus,
        }
    }
}

type LockedEnclaveVec = RwLock<Vec<RwLock<Enclave>>>;

static ENCLAVE_VEC: Once<LockedEnclaveVec> = Once::new();


pub(crate) fn coffer_sm_init() -> SbiRet {
    log::info!("Initializing CofferSBI Security Monitor");

    ENCLAVE_VEC.call_once(|| RwLock::new(Vec::with_capacity(1024)));

    SbiRet::success(0)
}

// ----- Utility functions -----
fn create_empty_enclave() {
    // randomly chosen. To be replaced with an allocated address
    let start_pc = 0x1_2000_0000_usize;

    let mut enclaves = ENCLAVE_VEC.get().unwrap().write();
    enclaves.push(RwLock::new(Enclave::new(start_pc - 4)));  // after cofferInst handler, the pc will be increased by 4
}



// -----------------------------
// test
pub(crate) fn coffer_sm_test(ctx: &mut FlowContext) -> SbiRet {
    log::debug!("CofferSBI Security Monitor test");

    // Test read
    {
        log::debug!("Test 1: Read enclave count");
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        log::debug!("Enclave count: {}", enclaves.len());
    }

    // Test write (adding a new enclave)
    {
        log::debug!("Test 2: Create enclave");
        create_empty_enclave();
    }

    // Test write to an enclave
    {
        log::debug!("Test 3: Write to enclave 1");
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        let encl1 = &mut enclaves[0].write();
        encl1.vcpus[0].gpr.a[0] = 0x12345678;
    }

    // Test read
    {
        log::debug!("Test 4: Read enclave 1");
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        log::debug!("Enclave count: {}", enclaves.len());
        
        // Print a0..a2 of the first vCPU
        let encl1 = &enclaves[0].read();
        let vcpus = &encl1.vcpus;
        log::debug!("vcpu 0:\n{:#?}", vcpus[0]);
    }

    // vcpu save and load context test
    {
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        let encl1 = &mut enclaves[0].write();  // write lock, or it will raise "cannot borrow as mutable"
        encl1.vcpus[0].save_context(ctx);
        encl1.vcpus[0].gpr.a[0] = 0x98765432;
        encl1.vcpus[0].load_context(ctx);  // without save, it will load the initial context which trigger a panic
        let vcpus = &encl1.vcpus;
        log::debug!("vcpu 0:\n{:#?}", vcpus[0]);
    }

    // Test vcpu context load (will cause a panic)
    // {
    //     let enclaves = ENCLAVE_VEC.get().unwrap().read();
    //     let encl1 = &mut enclaves[0].write();

    //     encl1.vcpus[0].vcpu_csr_test();    // write 0x12345678 into vcpu's all csr 
    //     encl1.vcpus[0].load_context(ctx);  // after load, it will cause a panic
    //     show_current_csr();
    // }

    log::debug!("CofferSBI Security Monitor test passed.");
    SbiRet::success(0)
}

pub(crate) fn coffer_enclave_test(ctx: &mut FlowContext) -> SbiRet {
    log::debug!("CofferSBI Enclave test");

    // entering enclave
    {
        log::debug!("entering enclave test");
        create_empty_enclave();

        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        let encl1 = &mut enclaves[0].write();

        encl1.host_vcpu.save_context(ctx);  // save host context
        encl1.vcpus[0].load_context(ctx);  // load enclave context
        show_current_csr();

        let vcpus = &encl1.vcpus;
        log::debug!("vcpu 0:\n{:#?}", vcpus[0]);

        show_current_csr();
    }

    log::debug!("CofferSBI Enclave created and going to enter...");
    SbiRet::success(0)
}

// ------- Debugging -------
fn show_current_csr() {
    let mepc = mepc::read();
    let medeleg = medeleg::read().bits();
    let mideleg = mideleg::read().bits();
    let mstatus = mstatus::read().bits();
    let mip = mip::read().bits();
    let mie = mie::read().bits();
    
    let sepc = sepc::read();
    let sstatus = sstatus::read().bits();
    let sscratch = sscratch::read();
    let stvec = stvec::read().bits();
    let satp = satp::read().bits();
    let scause = scause::read().bits();
    let sip = sip::read().bits();
    let sie = sie::read().bits();
    let stval = stval::read();
    
    log::debug!("mepc: 0x{:x}", mepc);
    log::debug!("medeleg: 0x{:x}", medeleg);
    log::debug!("mideleg: 0x{:x}", mideleg);
    log::debug!("mstatus: 0x{:x}", mstatus);
    log::debug!("mip: 0x{:x}", mip);
    log::debug!("mie: 0x{:x}", mie);

    log::debug!("sepc: 0x{:x}", sepc);
    log::debug!("sstatus: 0x{:x}", sstatus);
    log::debug!("sscratch: 0x{:x}", sscratch);
    log::debug!("stvec: 0x{:x}", stvec);
    log::debug!("satp: 0x{:x}", satp);
    log::debug!("scause: 0x{:x}", scause);
    log::debug!("sip: 0x{:x}", sip);
    log::debug!("sie: 0x{:x}", sie);
    log::debug!("stval: 0x{:x}", stval);
}
