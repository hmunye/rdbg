#![allow(dead_code)]

use super::{
    debug_reg, debug_reg_offset, fp_reg, fp_reg_mm, fp_reg_offset, fp_reg_size, fp_reg_st,
    fp_reg_xmm, gp_reg_8_bit_h, gp_reg_8_bit_l, gp_reg_16_bit, gp_reg_32_bit, gp_reg_64_bit,
    gp_reg_offset,
};

/// Specifies the type of a given [`RegisterInfo`].
#[derive(Debug)]
pub(crate) enum RegisterType {
    GeneralPurpose,
    SubRegister,
    FloatingPoint,
    Debug,
}

/// Different ways a given [`RegisterInfo`] can be interpreted.
#[derive(Debug)]
pub(crate) enum RegisterFormat {
    UInt,
    DoubleFloat,
    LongDouble,
    Vector,
}

/// Collection of information needed for a single [`RegisterInfo`].
#[derive(Debug)]
pub(crate) struct RegisterInfo {
    /// Name of the register
    pub name: &'static str,
    /// DWARF register number assigned in SYSV ABI.
    pub dwarf_id: i32,
    /// Size of register in bytes.
    pub size: usize,
    /// Byte offset into [`libc::user`].
    pub offset: usize,
    /// Type of register (e.g., general-purpose, sub-register, floating-point, debug).
    pub reg_type: RegisterType,
    /// How to interpret the data of the register.
    pub format: RegisterFormat,
}

impl RegisterInfo {
    /// Find a register by its name and return a reference to the [`RegisterInfo`].
    pub(crate) fn register_info_by_name(name: &str) -> Option<&'static RegisterInfo> {
        REGISTER_INFO.iter().find(|&reg| reg.name == name)
    }

    /// Find a register by its DWARF register number and return a reference to
    /// the [`RegisterInfo`].
    pub(crate) fn register_info_by_dwarf(dwarf_id: i32) -> Option<&'static RegisterInfo> {
        REGISTER_INFO.iter().find(|&reg| reg.dwarf_id == dwarf_id)
    }
}

// `RegisterInfo` definitions for 124 registers, including general-purpose registers
// (in 64-bit, 32-bit, 16-bit, and 8-bit sizes), FPU, MMX, SSE (xmm0 to xmm15),
// debug registers, and the `orig_rax` register.
const REGISTER_INFO: &[RegisterInfo] = &[
    gp_reg_64_bit!(rax, 0),
    gp_reg_64_bit!(rdx, 1),
    gp_reg_64_bit!(rcx, 2),
    gp_reg_64_bit!(rbx, 3),
    gp_reg_64_bit!(rsi, 4),
    gp_reg_64_bit!(rdi, 5),
    gp_reg_64_bit!(rbp, 6),
    gp_reg_64_bit!(rsp, 7),
    gp_reg_64_bit!(r8, 8),
    gp_reg_64_bit!(r9, 9),
    gp_reg_64_bit!(r10, 10),
    gp_reg_64_bit!(r11, 11),
    gp_reg_64_bit!(r12, 12),
    gp_reg_64_bit!(r13, 13),
    gp_reg_64_bit!(r14, 14),
    gp_reg_64_bit!(r15, 15),
    gp_reg_64_bit!(rip, 16),
    gp_reg_64_bit!(eflags, 49),
    gp_reg_64_bit!(cs, 51),
    gp_reg_64_bit!(fs, 54),
    gp_reg_64_bit!(gs, 55),
    gp_reg_64_bit!(ss, 52),
    gp_reg_64_bit!(ds, 53),
    gp_reg_64_bit!(es, 50),
    // Provided by [`libc::ptrace`] to get the ID of a syscall.
    gp_reg_64_bit!(orig_rax, -1),
    //=========================================================================
    gp_reg_32_bit!(eax, rax),
    gp_reg_32_bit!(edx, rdx),
    gp_reg_32_bit!(ecx, rcx),
    gp_reg_32_bit!(ebx, rbx),
    gp_reg_32_bit!(esi, rsi),
    gp_reg_32_bit!(edi, rdi),
    gp_reg_32_bit!(ebp, rbp),
    gp_reg_32_bit!(esp, rsp),
    gp_reg_32_bit!(r8d, r8),
    gp_reg_32_bit!(r9d, r9),
    gp_reg_32_bit!(r10d, r10),
    gp_reg_32_bit!(r11d, r11),
    gp_reg_32_bit!(r12d, r12),
    gp_reg_32_bit!(r13d, r13),
    gp_reg_32_bit!(r14d, r14),
    gp_reg_32_bit!(r15d, r15),
    //=========================================================================
    gp_reg_16_bit!(ax, rax),
    gp_reg_16_bit!(dx, rdx),
    gp_reg_16_bit!(cx, rcx),
    gp_reg_16_bit!(bx, rbx),
    gp_reg_16_bit!(si, rsi),
    gp_reg_16_bit!(di, rdi),
    gp_reg_16_bit!(bp, rbp),
    gp_reg_16_bit!(sp, rsp),
    gp_reg_16_bit!(r8w, r8),
    gp_reg_16_bit!(r9w, r9),
    gp_reg_16_bit!(r10w, r10),
    gp_reg_16_bit!(r11w, r11),
    gp_reg_16_bit!(r12w, r12),
    gp_reg_16_bit!(r13w, r13),
    gp_reg_16_bit!(r14w, r14),
    gp_reg_16_bit!(r15w, r15),
    //=========================================================================
    gp_reg_8_bit_h!(ah, rax),
    gp_reg_8_bit_h!(dh, rdx),
    gp_reg_8_bit_h!(ch, rcx),
    gp_reg_8_bit_h!(bh, rbx),
    //=========================================================================
    gp_reg_8_bit_l!(al, rax),
    gp_reg_8_bit_l!(dl, rdx),
    gp_reg_8_bit_l!(cl, rcx),
    gp_reg_8_bit_l!(bl, rbx),
    gp_reg_8_bit_l!(sil, rsi),
    gp_reg_8_bit_l!(dil, rdi),
    gp_reg_8_bit_l!(bpl, rbp),
    gp_reg_8_bit_l!(spl, rsp),
    gp_reg_8_bit_l!(r8b, r8),
    gp_reg_8_bit_l!(r9b, r9),
    gp_reg_8_bit_l!(r10b, r10),
    gp_reg_8_bit_l!(r11b, r11),
    gp_reg_8_bit_l!(r12b, r12),
    gp_reg_8_bit_l!(r13b, r13),
    gp_reg_8_bit_l!(r14b, r14),
    gp_reg_8_bit_l!(r15b, r15),
    //=========================================================================
    fp_reg!(fcw, 65, cwd),
    fp_reg!(fsw, 66, swd),
    fp_reg!(ftw, -1, ftw),
    fp_reg!(fop, -1, fop),
    fp_reg!(frip, -1, rip),
    fp_reg!(frdp, -1, rdp),
    fp_reg!(mxcsr, 64, mxcsr),
    fp_reg!(mxcsrmask, -1, mxcr_mask),
    //=========================================================================
    fp_reg_st!(0),
    fp_reg_st!(1),
    fp_reg_st!(2),
    fp_reg_st!(3),
    fp_reg_st!(4),
    fp_reg_st!(5),
    fp_reg_st!(6),
    fp_reg_st!(7),
    //=========================================================================
    fp_reg_mm!(0),
    fp_reg_mm!(1),
    fp_reg_mm!(2),
    fp_reg_mm!(3),
    fp_reg_mm!(4),
    fp_reg_mm!(5),
    fp_reg_mm!(6),
    fp_reg_mm!(7),
    //=========================================================================
    fp_reg_xmm!(0),
    fp_reg_xmm!(1),
    fp_reg_xmm!(2),
    fp_reg_xmm!(3),
    fp_reg_xmm!(4),
    fp_reg_xmm!(5),
    fp_reg_xmm!(6),
    fp_reg_xmm!(7),
    fp_reg_xmm!(8),
    fp_reg_xmm!(9),
    fp_reg_xmm!(10),
    fp_reg_xmm!(11),
    fp_reg_xmm!(12),
    fp_reg_xmm!(13),
    fp_reg_xmm!(14),
    fp_reg_xmm!(15),
    //=========================================================================
    debug_reg!(0),
    debug_reg!(1),
    debug_reg!(2),
    debug_reg!(3),
    debug_reg!(4),
    debug_reg!(5),
    debug_reg!(6),
    debug_reg!(7),
];
