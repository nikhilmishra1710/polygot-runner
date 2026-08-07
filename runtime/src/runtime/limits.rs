use std::io;

use rustix::{
    io::Errno,
    process::{Resource, Rlimit, setrlimit},
};

use crate::model::ResourceLimits;

fn apply_limit(resource: Resource, soft: u64, hard: u64) -> Result<(), Errno> {
    let limit = Rlimit {
        current: Some(soft),
        maximum: Some(hard),
    };

    setrlimit(resource, limit)
}

fn apply_cpu_limit(limits: &ResourceLimits) -> Result<(), Errno> {
    let seconds = limits.cpu_time.as_secs();
    apply_limit(Resource::Cpu, seconds, seconds + 1)
}

fn apply_file_size_limit(limits: &ResourceLimits) -> Result<(), Errno> {
    apply_limit(Resource::Fsize, limits.max_file_size, limits.max_file_size)
}

fn apply_open_file_limit(limits: &ResourceLimits) -> Result<(), Errno> {
    apply_limit(
        Resource::Nofile,
        limits.max_open_files,
        limits.max_open_files,
    )
}

pub fn apply_resource_limits(limits: &ResourceLimits) -> io::Result<()> {
    apply_cpu_limit(limits)?;
    apply_open_file_limit(limits)?;
    apply_file_size_limit(limits)?;

    Ok(())
}
