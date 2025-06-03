#[allow(unused_imports)]
use super::{RegisterFormat, RegisterInfo, RegisterType};

// Macro to calculate offset of a general-purpose register within `libc::user`.
macro_rules! gp_reg_offset {
    ($reg:ident) => {
        std::mem::offset_of!(libc::user, regs) + std::mem::offset_of!(libc::user_regs_struct, $reg)
    };
}

pub(crate) use gp_reg_offset;

// Macro to define register information for a 64-bit general-purpose register.
macro_rules! gp_reg_64_bit {
    ($name:ident, $dwarf_id:expr) => {
        RegisterInfo {
            name: stringify!($name),
            dwarf_id: $dwarf_id,
            size: 8,
            offset: gp_reg_offset!($name),
            reg_type: RegisterType::GeneralPurpose,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use gp_reg_64_bit;

// Macro to define register information for a 32-bit general-purpose sub-register.
macro_rules! gp_reg_32_bit {
    // `$name` is the sub-register, and `$super` is the parent 64-bit register.
    ($name:ident, $super:ident) => {
        RegisterInfo {
            name: stringify!($name),
            dwarf_id: -1,
            size: 4,
            offset: gp_reg_offset!($super),
            reg_type: RegisterType::SubRegister,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use gp_reg_32_bit;

// Macro to define register information for a 16-bit general-purpose sub-register.
macro_rules! gp_reg_16_bit {
    // `$name` is the sub-register, and `$super` is the parent 64-bit register.
    ($name:ident, $super:ident) => {
        RegisterInfo {
            name: stringify!($name),
            dwarf_id: -1,
            size: 2,
            offset: gp_reg_offset!($super),
            reg_type: RegisterType::SubRegister,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use gp_reg_16_bit;

// Macro to define register information for an 8-bit (high) general-purpose sub-register.
macro_rules! gp_reg_8_bit_h {
    // `$name` is the sub-register, and `$super` is the parent 64-bit register.
    ($name:ident, $super:ident) => {
        RegisterInfo {
            name: stringify!($name),
            dwarf_id: -1,
            size: 1,
            offset: gp_reg_offset!($super),
            reg_type: RegisterType::SubRegister,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use gp_reg_8_bit_h;

// Macro to define register information for an 8-bit (low) general-purpose sub-register.
macro_rules! gp_reg_8_bit_l {
    // `$name` is the sub-register, and `$super` is the parent 64-bit register.
    ($name:ident, $super:ident) => {
        RegisterInfo {
            name: stringify!($name),
            dwarf_id: -1,
            size: 1,
            offset: gp_reg_offset!($super),
            reg_type: RegisterType::SubRegister,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use gp_reg_8_bit_l;

//=============================================================================

// Macro to calculate offset of a floating-point register within the `i387`
// member of `libc::user`.
macro_rules! fp_reg_offset {
    ($reg:ident) => {
        std::mem::offset_of!(libc::user, i387)
            + std::mem::offset_of!(libc::user_fpregs_struct, $reg)
    };
}

pub(crate) use fp_reg_offset;

// Macro to calculate size of a floating-point register.
macro_rules! fp_reg_size {
    (cwd) => {
        std::mem::size_of::<libc::c_ushort>()
    };
    (swd) => {
        std::mem::size_of::<libc::c_ushort>()
    };
    (ftw) => {
        std::mem::size_of::<libc::c_ushort>()
    };
    (fop) => {
        std::mem::size_of::<libc::c_ushort>()
    };
    (rip) => {
        std::mem::size_of::<libc::c_ulonglong>()
    };
    (rdp) => {
        std::mem::size_of::<libc::c_ulonglong>()
    };
    (mxcsr) => {
        std::mem::size_of::<libc::c_uint>()
    };
    (mxcr_mask) => {
        std::mem::size_of::<libc::c_uint>()
    };
}

pub(crate) use fp_reg_size;

// Macro to define register information for a floating-point register.
macro_rules! fp_reg {
    ($name:ident, $dwarf_id:expr, $user_name:ident) => {
        RegisterInfo {
            name: stringify!($name),
            dwarf_id: $dwarf_id,
            size: fp_reg_size!($user_name),
            offset: fp_reg_offset!($user_name),
            reg_type: RegisterType::FloatingPoint,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use fp_reg;

// Macro to define register information for a `st` floating-point register.
macro_rules! fp_reg_st {
    ($number:expr) => {
        RegisterInfo {
            name: concat!("st", stringify!($number)),
            dwarf_id: 33 + $number,
            size: 16,
            offset: fp_reg_offset!(st_space) + ($number * 16),
            reg_type: RegisterType::FloatingPoint,
            format: RegisterFormat::LongDouble,
        }
    };
}

pub(crate) use fp_reg_st;

// Macro to define register information for a `mm` floating-point register.
macro_rules! fp_reg_mm {
    ($number:expr) => {
        RegisterInfo {
            name: concat!("mm", stringify!($number)),
            dwarf_id: 41 + $number,
            size: 8,
            offset: fp_reg_offset!(st_space) + ($number * 16),
            reg_type: RegisterType::FloatingPoint,
            format: RegisterFormat::Vector,
        }
    };
}

pub(crate) use fp_reg_mm;

// Macro to define register information for a `xmm` floating-point register.
macro_rules! fp_reg_xmm {
    ($number:expr) => {
        RegisterInfo {
            name: concat!("xmm", stringify!($number)),
            dwarf_id: 17 + $number,
            size: 16,
            offset: fp_reg_offset!(xmm_space) + ($number * 16),
            reg_type: RegisterType::FloatingPoint,
            format: RegisterFormat::Vector,
        }
    };
}

pub(crate) use fp_reg_xmm;

//=============================================================================

// Macro to calculate offset of a debug register within `libc::user`.
macro_rules! debug_reg_offset {
    ($number:expr) => {
        std::mem::offset_of!(libc::user, u_debugreg) + $number * 8
    };
}

pub(crate) use debug_reg_offset;

// Macro to define register information for a debug register.
macro_rules! debug_reg {
    ($number:expr) => {
        RegisterInfo {
            name: concat!("dr", stringify!($number)),
            dwarf_id: -1,
            size: 8,
            offset: debug_reg_offset!($number),
            reg_type: RegisterType::Debug,
            format: RegisterFormat::UInt,
        }
    };
}

pub(crate) use debug_reg;
