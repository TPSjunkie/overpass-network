use std::time::Duration;

pub struct PerformanceConfig {
    pub num_iterations: usize,
    pub proof_size_limit: usize,
    pub max_generation_time: Duration,
    pub max_verification_time: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            num_iterations: 100,
            proof_size_limit: 1024 * 1024, // 1MB
            max_generation_time: Duration::from_secs(5),
            max_verification_time: Duration::from_secs(1),
        }
    }
}

pub struct CircuitConfig {
    pub max_constraints: usize,
    pub max_variables: usize,
    pub max_degree: usize,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            max_constraints: 1 << 16, // 65536
            max_variables: 1 << 15,   // 32768
            max_degree: 8,
        }
    }
}

#[derive(Default)]
pub struct TestParameters {
    pub perf_config: PerformanceConfig,
    pub circuit_config: CircuitConfig,
}

impl TestParameters {
    pub fn verify_performance_metrics(
        &self,
        generation_time: Duration,
        verification_time: Duration,
        proof_size: usize,
    ) -> bool {
        generation_time <= self.perf_config.max_generation_time
            && verification_time <= self.perf_config.max_verification_time
            && proof_size <= self.perf_config.proof_size_limit
    }
}
