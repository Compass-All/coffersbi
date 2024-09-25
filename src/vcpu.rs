use core::fmt::{self, Formatter, Debug};

use fast_trap::FlowContext;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct MCsr {
    mstatus: usize,
    mepc: usize,
    mip: usize,
    mie: usize,
    medeleg: usize,
    mideleg: usize,
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
        let gpr = FlowContext {
            ra: self.gpr.ra,
            t: self.gpr.t,
            a: self.gpr.a,
            s: self.gpr.s,
            gp: self.gpr.gp,
            tp: self.gpr.tp,
            sp: self.gpr.sp,
            pc: self.gpr.pc,
        };
        f.debug_struct("General Purpose Registers")
            .field("ra", &gpr.ra)
            .field("t", &gpr.t)
            .field("a", &gpr.a)
            .field("s", &gpr.s)
            .field("gp", &gpr.gp)
            .field("tp", &gpr.tp)
            .field("sp", &gpr.sp)
            .field("pc", &gpr.pc)
            .finish()?;
        f.debug_struct("Floating Point Registers")
            .field("fpr", &self.fpr)
            .finish()?;
        f.debug_struct("Supervisor CSR")
            .field("scsr", &self.scsr)
            .finish()?;
        f.debug_struct("Machine CSR")
            .field("mcsr", &self.mcsr)
            .finish()
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
}
