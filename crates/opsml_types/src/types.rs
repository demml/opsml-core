use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CPUMetrics {
    pub cpu_percent_utilization: f64,
    pub cpu_percent_per_core: Option<Vec<f64>>,
    pub compute_overall: Option<f64>,
    pub compute_utilized: Option<f64>,
    pub load_avg: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GPUMetrics {
    pub gpu_percent_utilization: f64,
    pub gpu_percent_per_core: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MemoryMetrics {
    pub sys_ram_total: i64,
    pub sys_ram_used: i64,
    pub sys_ram_available: i64,
    pub sys_ram_percent_used: f64,
    pub sys_swap_total: Option<i64>,
    pub sys_swap_used: Option<i64>,
    pub sys_swap_free: Option<i64>,
    pub sys_swap_percent: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NetworkRates {
    pub bytes_recv: i64,
    pub bytes_sent: i64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HardwareMetrics {
    pub cpu: CPUMetrics,
    pub memory: MemoryMetrics,
    pub network: NetworkRates,
    pub gpu: Option<GPUMetrics>,
}
