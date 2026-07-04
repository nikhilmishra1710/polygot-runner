use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLimits {
    pub wall_time: Duration,
}
