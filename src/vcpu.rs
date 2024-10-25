use core::fmt::{self, Debug, Formatter};

use riscv::register::*;
use alloc::{string::String, vec::Vec};
use fast_trap::FlowContext;

#[derive(Clone)]
struct SCsr {
    sstatus: usize,
    sscratch: usize,
    sepc: usize,
    stvec: usize,
    satp: usize,
    scause: usize,
    stval: usize,
    sip: usize,
    sie: usize,
}

impl Debug for SCsr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Supervisor CSR")
            .field("sstatus", &format!("{:#x}", self.sstatus))
            .field("sscratch", &format!("{:#x}", self.sscratch))
            .field("sepc", &format!("{:#x}", self.sepc))
            .field("stvec", &format!("{:#x}", self.stvec))
            .field("satp", &format!("{:#x}", self.satp))
            .field("scause", &format!("{:#x}", self.scause))
            .field("stval", &format!("{:#x}", self.stval))
            .field("sip", &format!("{:#x}", self.sip))
            .field("sie", &format!("{:#x}", self.sie))
            .finish()
    }
}

#[derive(Clone)]
struct MCsr {
    mstatus: usize,
    mepc: usize,
    mip: usize,
    mie: usize,
    medeleg: usize,
    mideleg: usize,
}

// fmt MCsr using hex
impl Debug for MCsr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Machine CSR")
            .field("mstatus", &format!("{:#x}", self.mstatus))
            .field("mepc", &format!("{:#x}", self.mepc))
            .field("mip", &format!("{:#x}", self.mip))
            .field("mie", &format!("{:#x}", self.mie))
            .field("medeleg", &format!("{:#x}", self.medeleg))
            .field("mideleg", &format!("{:#x}", self.mideleg))
            .finish()
    }
}

pub(crate) struct VCpu {
    pub(crate) gpr: FlowContext,
    fpr: [f64; 32], // Currently only 64-bit floating point registers are supported
    scsr: SCsr,
    mcsr: MCsr,
}

impl Clone for VCpu {
    fn clone(&self) -> Self {
        VCpu {
            gpr: FlowContext {
                ra: self.gpr.ra,
                t: self.gpr.t,
                a: self.gpr.a,
                s: self.gpr.s,
                gp: self.gpr.gp,
                tp: self.gpr.tp,
                sp: self.gpr.sp,
                pc: self.gpr.pc,
            },
            fpr: self.fpr.clone(),
            scsr: self.scsr.clone(),
            mcsr: self.mcsr.clone(),
        }
    }
}

impl Debug for VCpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Helper function to format arrays in hex format
        fn format_compact_hex_array<T: fmt::LowerHex>(array: &[T]) -> String {
            let formatted_array: Vec<String> = array
                .iter()
                .map(|val| format!("{:#x}", val)) // Format each element in hex
                .collect();
            format!("[ {} ]", formatted_array.join(", ")) // Join elements with comma and enclose in brackets
        }
        fn format_compact_array<T: fmt::Debug>(array: &[T]) -> String {
            let formatted_array: Vec<String> = array
                .iter()
                .map(|val| format!("{:?}", val)) // Format each element in hex
                .collect();
            format!("[ {} ]", formatted_array.join(", ")) // Join elements with comma and enclose in brackets
        }

        writeln!(f, "General Purpose Registers")?;
        writeln!(f, "ra: {:#x}", self.gpr.ra)?;
        writeln!(f, "t: {}", format_compact_hex_array(&self.gpr.t))?;
        writeln!(f, "a: {}", format_compact_hex_array(&self.gpr.a))?;
        writeln!(f, "s: {}", format_compact_hex_array(&self.gpr.s))?;
        writeln!(f, "gp: {:#x}", self.gpr.gp)?;
        writeln!(f, "tp: {:#x}", self.gpr.tp)?;
        writeln!(f, "sp: {:#x}", self.gpr.sp)?;
        writeln!(f, "pc: {:#x}", self.gpr.pc)?;

        f.debug_struct("Floating Point Registers")
            .field("fpr", &format_compact_array(&self.fpr))
            .finish()?;
        writeln!(f)?;

        f.debug_struct("Supervisor CSR")
            .field("scsr", &self.scsr)
            .finish()?;
        writeln!(f)?;

        f.debug_struct("Machine CSR")
            .field("mcsr", &self.mcsr)
            .finish()
    }
}

impl VCpu {
    fn save_csr(&mut self) {
        self.mcsr.mepc = mepc::read();  // return usize, not struct
        self.mcsr.medeleg = medeleg::read().bits();
        self.mcsr.mideleg = mideleg::read().bits();
        self.mcsr.mstatus = mstatus::read().bits();
        self.mcsr.mip = mip::read().bits();
        self.mcsr.mie = mie::read().bits();
        
        self.scsr.sepc = sepc::read();
        self.scsr.sstatus = sstatus::read().bits();
        self.scsr.sscratch = sscratch::read();
        self.scsr.stvec = stvec::read().bits();
        self.scsr.satp = satp::read().bits();
        self.scsr.scause = scause::read().bits();
        self.scsr.sip = sip::read().bits();
        self.scsr.sie = sie::read().bits();
        self.scsr.stval = stval::read();

    }

    fn load_csr(&mut self) {
        unsafe {
            sstatus::write(self.scsr.sstatus);
            sscratch::write(self.scsr.sscratch);
            sepc::write(self.scsr.sepc);
            stvec::write(self.scsr.stvec);
            satp::write(self.scsr.satp);
            scause::write(self.scsr.scause);
            stval::write(self.scsr.stval);
            sip::write(self.scsr.sip);
            sie::write(self.scsr.sie);

            mstatus::write(mstatus::Mstatus::from(self.mcsr.mstatus));
            mepc::write(self.mcsr.mepc);
            mip::write(self.mcsr.mip);
            mie::write(self.mcsr.mie);
            medeleg::write(self.mcsr.medeleg);
            mideleg::write(self.mcsr.mideleg);
        }
    }
    
    fn save_gpr(&mut self) {
       // TODO
    }

    fn load_gpr(&mut self) {
       // TODO
    }

    fn save_fprs(&mut self) {
        // TODO
    }

    fn load_fprs(&mut self) {
        // TODO
    }
}

impl VCpu {
    pub fn init_context(start_pc: usize) -> Self {
        VCpu {
            gpr: FlowContext::ZERO,
            fpr: [0.0; 32],
            scsr: SCsr {
                sstatus: 0b11 << 13, // sstatus.fs: FS::Dirty
                sscratch: 0,
                sepc: 0,
                stvec: 0,
                satp: 0,
                scause: 0,
                stval: 0,
                sip: 0,
                sie: 0,
            },
            mcsr: MCsr {
                mstatus: 0b11   << 13   // mstatus.fs: FS::Dirty
                       | 0b1    << 11   // mstatus.mpp: MPP::Supervisor
                       | 0b1    << 1    // mstatus.sie: true
                       | 0b1    << 18, // mstatus.sum: true
                mepc: start_pc,
                mip: 0_usize,
                mie: 0b1    << 1    // mie.ssoft
                   | 0b1    << 3    // mie.msoft
                   | 0b1    << 5    // mie.stimer
                   | 0b1    << 7    // mie.mtimer
                   | 0b1    << 9, // mie.sext
                medeleg: 0b1    << 0    // medeleg.instruction_misaligned
                       | 0b1    << 3    // medeleg.breakpoint
                       | 0b1    << 4    // medeleg.load_misaligned
                       | 0b1    << 6, // medeleg.store_misaligned
                mideleg: 0b1    << 1    // mideleg.ssoft
                       | 0b1    << 5    // mideleg.stimer
                       | 0b1    << 9, // mideleg.sext
            },
        }
    }

    pub fn save_context(&mut self) {
        self.save_gpr();
        self.save_fprs();
        self.save_csr();
    }

    pub fn load_context(&mut self) {
        self.load_gpr();
        self.load_fprs();
        self.load_csr();
    }
}