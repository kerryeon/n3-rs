mod code;
mod device;
mod exec;
mod host;
mod process;
mod python;

pub use n3_machine::Result;
pub use n3_torch_ffi::{self as ffi, pyo3, SignalHandler};

pub use self::host::HostMachine;

use self::device::CandidatesMachine;

/// Define built-in machine generators here.
pub(crate) const BUILTIN_MACHINES: &[(&str, n3_machine::Generator)] = &[
    ("cpu", self::device::CpuMachine::try_new),
    ("cuda", self::device::CudaMachine::try_new),
];