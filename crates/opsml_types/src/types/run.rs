use crate::GraphStyle;
use chrono::NaiveDateTime;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub sys_ram_total: i32,
    pub sys_ram_used: i32,
    pub sys_ram_available: i32,
    pub sys_ram_percent_used: f64,
    pub sys_swap_total: Option<i32>,
    pub sys_swap_used: Option<i32>,
    pub sys_swap_free: Option<i32>,
    pub sys_swap_percent: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NetworkRates {
    pub bytes_recv: i32,
    pub bytes_sent: i32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HardwareMetrics {
    pub cpu: CPUMetrics,
    pub memory: MemoryMetrics,
    pub network: NetworkRates,
    pub gpu: Option<GPUMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub step: Option<i32>,
    pub timestamp: Option<i64>,
    pub created_at: Option<NaiveDateTime>,
}

impl Default for Metric {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            value: 0.0,
            step: None,
            timestamp: None,
            created_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: String,
    pub created_at: Option<NaiveDateTime>,
}

impl Parameter {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            created_at: None,
        }
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            value: "".to_string(),
            created_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct RunLineGraph {
    name: String,
    x_label: String,
    y_label: String,
    x: Vec<f64>,
    y: Vec<f64>,
    graph_style: GraphStyle,
}

#[pymethods]
impl RunLineGraph {
    #[new]
    fn new(
        name: String,
        x_label: String,
        y_label: String,
        x: Vec<f64>,
        y: Vec<f64>,
        graph_style: GraphStyle,
    ) -> Self {
        Self {
            name,
            x_label,
            y_label,
            x,
            y,
            graph_style,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass]
pub struct RunMultiLineGraph {
    name: String,
    x_label: String,
    y_label: String,
    x: Vec<f64>,
    y: HashMap<String, Vec<f64>>,
    graph_style: GraphStyle,
}

#[pymethods]
impl RunMultiLineGraph {
    #[new]
    fn new(
        name: String,
        x_label: String,
        y_label: String,
        x: Vec<f64>,
        y: HashMap<String, Vec<f64>>,
        graph_style: GraphStyle,
    ) -> Self {
        Self {
            name,
            x_label,
            y_label,
            x,
            y,
            graph_style,
        }
    }
}
