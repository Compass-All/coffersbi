use core::fmt::{self, Debug, Formatter};
use core::arch::asm;

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
    scsr: SCsr,
    mcsr: MCsr,
    pub(crate) gpr: FlowContext,
    #[cfg(target_feature = "f")]  // not test yet
    fpr: [f64; 32], // Currently only 64-bit floating point registers are supported
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
            #[cfg(target_feature = "f")]
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

        #[cfg(target_feature = "f")]
        {
            f.debug_struct("Floating Point Registers")
                .field("fpr", &format_compact_array(&self.fpr))
                .finish()?;
            writeln!(f)?;
        }

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

    fn load_csr(&self) {
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
    
    fn save_gpr(&mut self, gpr: &FlowContext) {
        // FlowContext is a struct without Clone trait
        self.gpr.ra = gpr.ra;
        self.gpr.t = gpr.t;
        self.gpr.a = gpr.a;
        self.gpr.s = gpr.s;
        self.gpr.gp = gpr.gp;
        self.gpr.tp = gpr.tp;
        self.gpr.sp = gpr.sp;
        self.gpr.pc = gpr.pc;
    }

    fn load_gpr(&self, gpr: &mut FlowContext) {
        // FlowContext is a struct without Clone trait
        gpr.ra = self.gpr.ra;
        gpr.t = self.gpr.t;
        gpr.a = self.gpr.a;
        gpr.s = self.gpr.s;
        gpr.gp = self.gpr.gp;
        gpr.tp = self.gpr.tp;
        gpr.sp = self.gpr.sp;
        gpr.pc = self.gpr.pc;        
    }

    #[cfg(target_feature = "f")]
    fn save_fprs(&mut self) {  // not test yet
        unsafe {
            asm!(
                "fsd f0, 0*8({0})",
                "fsd f1, 1*8({0})",
                "fsd f2, 2*8({0})",
                "fsd f3, 3*8({0})",
                "fsd f4, 4*8({0})",
                "fsd f5, 5*8({0})",
                "fsd f6, 6*8({0})",
                "fsd f7, 7*8({0})",
                "fsd f8, 8*8({0})",
                "fsd f9, 9*8({0})",
                "fsd f10, 10*8({0})",
                "fsd f11, 11*8({0})",
                "fsd f12, 12*8({0})",
                "fsd f13, 13*8({0})",
                "fsd f14, 14*8({0})",
                "fsd f15, 15*8({0})",
                "fsd f16, 16*8({0})",
                "fsd f17, 17*8({0})",
                "fsd f18, 18*8({0})",
                "fsd f19, 19*8({0})",
                "fsd f20, 20*8({0})",
                "fsd f21, 21*8({0})",
                "fsd f22, 22*8({0})",
                "fsd f23, 23*8({0})",
                "fsd f24, 24*8({0})",
                "fsd f25, 25*8({0})",
                "fsd f26, 26*8({0})",
                "fsd f27, 27*8({0})",
                "fsd f28, 28*8({0})",
                "fsd f29, 29*8({0})",
                "fsd f30, 30*8({0})",
                "fsd f31, 31*8({0})",
                in(reg) self.fpr.as_ptr(),
            );
        }
    }

    #[cfg(target_feature = "f")]
    fn load_fprs(&self) {  // not test yet
        unsafe {
            asm!(
                "fld f0, 0*8({0})",
                "fld f1, 1*8({0})",
                "fld f2, 2*8({0})",
                "fld f3, 3*8({0})",
                "fld f4, 4*8({0})",
                "fld f5, 5*8({0})",
                "fld f6, 6*8({0})",
                "fld f7, 7*8({0})",
                "fld f8, 8*8({0})",
                "fld f9, 9*8({0})",
                "fld f10, 10*8({0})",
                "fld f11, 11*8({0})",
                "fld f12, 12*8({0})",
                "fld f13, 13*8({0})",
                "fld f14, 14*8({0})",
                "fld f15, 15*8({0})",
                "fld f16, 16*8({0})",
                "fld f17, 17*8({0})",
                "fld f18, 18*8({0})",
                "fld f19, 19*8({0})",
                "fld f20, 20*8({0})",
                "fld f21, 21*8({0})",
                "fld f22, 22*8({0})",
                "fld f23, 23*8({0})",
                "fld f24, 24*8({0})",
                "fld f25, 25*8({0})",
                "fld f26, 26*8({0})",
                "fld f27, 27*8({0})",
                "fld f28, 28*8({0})",
                "fld f29, 29*8({0})",
                "fld f30, 30*8({0})",
                "fld f31, 31*8({0})",
                in(reg) self.fpr.as_ptr(),
            );
        }
    }
}

impl VCpu {
    pub fn init_context(start_pc: usize) -> Self {
        VCpu {
            gpr: FlowContext::ZERO,
            #[cfg(target_feature = "f")]
            fpr: [0.0; 32],
            scsr: SCsr {
                sstatus: 0b11 << 13,  // sstatus.fs: FS::Dirty
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
                mstatus: 0b11   << 13    // mstatus.FS: FS::Dirty
                       | 0b01   << 11    // mstatus.MPP: MPP::Supervisor
                       | 0b1    << 1     // mstatus.SIE: true
                       | 0b1    << 18,   // mstatus.SUM: true
                mepc: start_pc,
                mip: 0_usize,
                mie: 0b1    << 1    // mie.ssoft
                   | 0b1    << 3    // mie.msoft
                // TODO?
                //    | 0b1    << 5    // mie.stimer
                //    | 0b1    << 7    // mie.mtimer
                   | 0b1    << 9,   // mie.sext
                medeleg: 0b1    << 0    // medeleg.instruction_misaligned
                       | 0b1    << 3    // medeleg.breakpoint
                       | 0b1    << 4    // medeleg.load_misaligned
                       | 0b1    << 6,   // medeleg.store_misaligned
                mideleg: 0b1    << 1    // mideleg.ssoft
                       | 0b1    << 5    // mideleg.stimer
                       | 0b1    << 9,   // mideleg.sext
            },
        }
    }

    pub fn save_context(&mut self, ctx: &FlowContext) {
        self.save_gpr(ctx);
        #[cfg(target_feature = "f")]
        self.save_fprs();
        self.save_csr();
    }

    pub fn load_context(&self, ctx: &mut FlowContext) {
        self.load_gpr(ctx);
        #[cfg(target_feature = "f")]
        self.load_fprs();
        self.load_csr();
    }


    // ------- test -------
    pub fn vcpu_csr_test(&mut self) {   
        self.mcsr.mepc = 0x12345678;
        self.mcsr.medeleg = 0x12345678;
        self.mcsr.mideleg = 0x12345678;
        self.mcsr.mstatus = 0x12345678;
        self.mcsr.mip = 0x12345678;
        self.mcsr.mie = 0x12345678;

        self.scsr.sepc = 0x12345678;
        self.scsr.sstatus = 0x12345678;
        self.scsr.sscratch = 0x12345678;
        self.scsr.stvec = 0x12345678;
        self.scsr.satp = 0x12345678;
        self.scsr.scause = 0x12345678;
        self.scsr.sip = 0x12345678;
        self.scsr.sie = 0x12345678;
        self.scsr.stval = 0x12345678;
    }
}

