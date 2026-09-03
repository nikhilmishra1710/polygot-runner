pub mod cgroup;
pub mod engine;
pub mod error;
pub mod job;
pub mod language;
pub mod model;
pub mod runtime;
pub mod sandbox;
pub mod seccomp;
pub mod toolchain;
pub mod worker;
pub mod workspace;
pub mod protocol;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
