use std::{
    cmp::max,
    collections::HashMap,
    io::{BufReader, Read},
    process::{Command, Stdio},
};

use psutil::memory::{swap_memory, virtual_memory};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Metrics {
    /// mem
    pub mem: MemoryMetrics,

    /// cpu metrics
    pub cpu_metrics: CPUMetrics,

    /// gpu metrics
    pub gpu_metrics: GPUMetrics,

    /// net disk metrics
    pub net_disk_metrics: NetDiskMetrics,

    /// process metrics
    pub process_metrics: ProcessMetrics,

    /// os info
    pub soc_info: HashMap<String, String>,

    /// regex
    residency_re: Regex,
    frequency_re: Regex,
    re: Regex,
    out_re: Regex,
    in_re: Regex,
    read_re: Regex,
    write_re: Regex,
    data_re: Regex,
}

#[derive(Debug, Default, Clone)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl MemoryMetrics {
    pub fn update(&mut self) {
        let v = virtual_memory().unwrap();
        let s = swap_memory().unwrap();

        self.total = v.total();
        self.used = v.used();
        self.available = v.available();
        self.swap_total = s.total();
        self.swap_used = s.used();
    }
}

#[derive(Debug, Default, Clone)]
pub struct CPUMetrics {
    pub e_cluster_active: i64,
    pub e_cluster_freq_mhz: i64,
    pub p_cluster_active: i64,
    pub p_cluster_freq_mhz: i64,

    // e_cores: Vec<i8>,
    // p_cores: Vec<i8>,
    pub cores: Vec<i64>,

    pub ane_w: f64,
    pub cpu_w: f64,
    pub gpu_w: f64,
    pub package_w: f64,

    pub e0_cluster_active: i64,
    pub e0_cluster_freq_mhz: i64,
    pub e1_cluster_active: i64,
    pub e1_cluster_freq_mhz: i64,
    pub p0_cluster_active: i64,
    pub p0_cluster_freq_mhz: i64,
    pub p1_cluster_active: i64,
    pub p1_cluster_freq_mhz: i64,
    pub p2_cluster_active: i64,
    pub p2_cluster_freq_mhz: i64,
    // p3_cluster_active: i64,
    // p3_cluster_freq_mhz: i64,
}

#[derive(Debug, Default, Clone)]
pub struct NetDiskMetrics {
    pub out_packets_per_sec: f64,
    pub out_bytes_per_sec: f64,
    pub in_packets_per_sec: f64,
    pub in_bytes_per_sec: f64,
    pub read_ops_per_sec: f64,
    pub write_ops_per_sec: f64,
    pub read_k_bytes_per_sec: f64,
    pub write_k_bytes_per_sec: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GPUMetrics {
    pub freq_mhz: i64,
    pub active: f64,
}

#[derive(Debug, Default, Clone)]
pub struct ProcessMetrics {
    id: i64,
    name: String,
    cpu_usage: f64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            soc_info: get_soc_info(),
            residency_re: Regex::new(r"(\w+-Cluster)\s+HW active residency:\s+(\d+\.\d+)%")
                .unwrap(),
            frequency_re: Regex::new(r"(\w+-Cluster)\s+HW active frequency:\s+(\d+)\s+MHz")
                .unwrap(),
            re: Regex::new(r"GPU\s*(HW)?\s*active\s*(residency|frequency):\s+(\d+(\.)?(\d+)?)%?")
                .unwrap(),
            out_re: Regex::new(r"out:\s*([\d.]+)\s*packets/s,\s*([\d.]+)\s*bytes/s").unwrap(),
            in_re: Regex::new(r"in:\s*([\d.]+)\s*packets/s,\s*([\d.]+)\s*bytes/s").unwrap(),
            read_re: Regex::new(r"read:\s*([\d.]+)\s*ops/s\s*([\d.]+)\s*KBytes/s").unwrap(),
            write_re: Regex::new(r"write:\s*([\d.]+)\s*ops/s\s*([\d.]+)\s*KBytes/s").unwrap(),
            data_re: Regex::new(r"(?m)^\s*(\S.*?)\s+(\d+)\s+(\d+\.\d+)\s+\d+\.\d+\s+").unwrap(),
            mem: MemoryMetrics::default(),
            cpu_metrics: CPUMetrics::default(),
            gpu_metrics: GPUMetrics::default(),
            net_disk_metrics: NetDiskMetrics::default(),
            process_metrics: ProcessMetrics::default(),
        }
    }

    pub fn collect_metrics(&mut self) {
        let mut child = Command::new("powermetrics")
            .args([
                "--samplers",
                "cpu_power,gpu_power,thermal,network,disk",
                "--show-process-gpu",
                "--show-process-energy",
                "--show-initial-usage",
                "--show-process-netstats",
                "-n 1",
                "-i 1000",
            ])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut info = String::new();
            reader.read_to_string(&mut info).unwrap();

            self.parse_cpu_metrics(&info);
            self.parse_gpu_metrics(&info);
            self.parse_activity_metrics(&info);
            self.parse_process_metrics(&info);
        }
        self.mem.update();
    }

    fn parse_gpu_metrics(&mut self, info: &str) {
        info.split('\n')
            .filter(|line| line.contains("GPU active") || line.contains("GPU HW active"))
            .map(|line| self.re.captures(line))
            .for_each(|captures| {
                if let Some(matches) = captures {
                    // println!("{},{}", &matches[2], &matches[3]);
                    if &matches[2] == "frequency" {
                        self.gpu_metrics.freq_mhz = matches[3].parse::<i64>().unwrap();
                    } else if &matches[2] == "residency" {
                        self.gpu_metrics.active = matches[3].parse::<f64>().unwrap();
                    }
                }
            });
    }

    fn parse_cpu_metrics(&mut self, info: &str) {
        // let mut e_cores: Vec<i64> = vec![];
        // let mut p_cores: Vec<i64> = vec![];
        let mut e_cluster_active_total = 0i64;
        let mut e_cluster_count = 0i64;
        let mut p_cluster_active_total = 0i64;
        let mut p_cluster_count = 0i64;

        let mut e_cluster_freq_total = 0i64;
        info.split('\n')
            .map(|line| {
                (
                    self.residency_re.captures(line),
                    self.frequency_re.captures(line),
                    line,
                )
            })
            .for_each(|(residency_opt, frequency_opt, line)| {
                if let Some(residency) = residency_opt {
                    let cluster = &residency[1];
                    let value = residency[2].parse::<f64>().unwrap() as i64;
                    match cluster {
                        "E0-Cluster" => {
                            // println!("{}", &residency[2]);
                            self.cpu_metrics.e0_cluster_active = value;
                        }
                        "E1-Cluster" => {
                            self.cpu_metrics.e1_cluster_active = value;
                        }
                        "P0-Cluster" => {
                            self.cpu_metrics.p0_cluster_active = value;
                        }
                        "P1-Cluster" => {
                            self.cpu_metrics.p1_cluster_active = value;
                        }
                        "P2-Cluster" => {
                            self.cpu_metrics.p2_cluster_active = value;
                        }
                        _ => {
                            // println!("{}", &residency[1]);
                        }
                    }
                    if cluster.starts_with('E') {
                        e_cluster_active_total += value;
                        e_cluster_count += 1;
                    } else if cluster.starts_with('P') {
                        p_cluster_active_total += value;
                        p_cluster_count += 1;
                        self.cpu_metrics.p_cluster_active = p_cluster_active_total / p_cluster_count
                    }
                }

                if let Some(frequency) = frequency_opt {
                    let cluster = &frequency[1];
                    let value = frequency[2].parse::<i64>().unwrap();
                    match cluster {
                        "E0-Cluster" => {
                            self.cpu_metrics.e0_cluster_freq_mhz = value;
                        }
                        "E1-Cluster" => {
                            self.cpu_metrics.e1_cluster_freq_mhz = value;
                        }
                        "P0-Cluster" => {
                            self.cpu_metrics.p0_cluster_freq_mhz = value;
                        }
                        "P1-Cluster" => {
                            self.cpu_metrics.p1_cluster_freq_mhz = value;
                        }
                        "P2-Cluster" => {
                            self.cpu_metrics.p2_cluster_freq_mhz = value;
                        }
                        _ => {
                            // println!("{}", &frequency[1]);
                        }
                    }
                    if cluster.starts_with('E') {
                        e_cluster_freq_total += value;
                        self.cpu_metrics.e_cluster_freq_mhz = e_cluster_freq_total;
                    }
                }

                if line.contains("CPU") && line.contains("frequency") {
                    let fields: Vec<&str> = line.split(' ').collect();
                    if fields.len() >= 5 {
                        // TODO split e_core,p_core
                        self.cpu_metrics.cores.clear();
                        self.cpu_metrics.cores.push(fields[1].parse().unwrap());
                    }
                } else if line.contains("ANE Power") {
                    let fields: Vec<&str> = line.split(' ').collect();
                    if fields.len() >= 4 {
                        // Convert mW to W
                        self.cpu_metrics.ane_w = fields[2].parse::<f64>().unwrap() / 1000.0;
                    }
                } else if line.contains("CPU Power") {
                    let fields: Vec<&str> = line.split(' ').collect();
                    if fields.len() >= 4 {
                        // Convert mW to W
                        self.cpu_metrics.cpu_w = fields[2].parse::<f64>().unwrap() / 1000.0;
                    }
                } else if line.contains("GPU Power") {
                    let fields: Vec<&str> = line.split(' ').collect();
                    if fields.len() >= 4 {
                        // Convert mW to W
                        self.cpu_metrics.gpu_w = fields[2].parse::<f64>().unwrap() / 1000.0;
                    }
                } else if line.contains("Combined Power (CPU + GPU + ANE)") {
                    let fields: Vec<&str> = line.split(' ').collect();
                    if fields.len() >= 8 {
                        // Convert mW to W
                        self.cpu_metrics.package_w = fields[7].parse::<f64>().unwrap() / 1000.0;
                    }
                }
                // M1 Pro
                self.cpu_metrics.p_cluster_active =
                    (self.cpu_metrics.p0_cluster_active + self.cpu_metrics.p1_cluster_active) / 2;
                self.cpu_metrics.p_cluster_freq_mhz = max(
                    self.cpu_metrics.p0_cluster_freq_mhz,
                    self.cpu_metrics.p1_cluster_freq_mhz,
                );

                if e_cluster_count > 0 {
                    self.cpu_metrics.e_cluster_active = e_cluster_active_total / e_cluster_count;
                }
            });
    }

    fn parse_activity_metrics(&mut self, info: &str) {
        info.split('\n')
            .map(|line| {
                (
                    self.in_re.captures(line),
                    self.out_re.captures(line),
                    self.read_re.captures(line),
                    self.write_re.captures(line),
                )
            })
            .for_each(|(in_opts, out_opts, read_opts, write_opts)| {
                if let Some(in_caps) = in_opts {
                    // println!("in_caps:{:?}", in_caps);
                    self.net_disk_metrics.in_packets_per_sec = in_caps[1].parse::<f64>().unwrap();
                    self.net_disk_metrics.in_bytes_per_sec = in_caps[2].parse::<f64>().unwrap();
                }

                if let Some(out_caps) = out_opts {
                    // println!("out_caps:{:?}", out_caps);
                    self.net_disk_metrics.out_packets_per_sec = out_caps[1].parse::<f64>().unwrap();
                    self.net_disk_metrics.out_bytes_per_sec = out_caps[2].parse::<f64>().unwrap();
                }

                if let Some(read_caps) = read_opts {
                    self.net_disk_metrics.read_ops_per_sec = read_caps[1].parse::<f64>().unwrap();
                    self.net_disk_metrics.read_k_bytes_per_sec =
                        read_caps[2].parse::<f64>().unwrap();
                }

                if let Some(write_caps) = write_opts {
                    self.net_disk_metrics.write_ops_per_sec = write_caps[1].parse::<f64>().unwrap();
                    self.net_disk_metrics.write_k_bytes_per_sec =
                        write_caps[2].parse::<f64>().unwrap();
                }
            })
    }

    fn parse_process_metrics(&mut self, info: &str) {
        info.split('\n')
            .map(|line| self.data_re.captures(line))
            .for_each(|data_ops| {
                if let Some(data_caps) = data_ops {
                    // println!("data: {:?}", data_caps);
                }
            })
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

fn get_soc_info() -> HashMap<String, String> {
    let cpu_info = get_cpu_info();
    let core_count = get_core_count();

    let mut res = HashMap::new();
    if let Some(val) = core_count.get("hw.perflevel1.logicalcpu") {
        res.insert("e_core_count".to_string(), val.to_owned());
    }
    if let Some(val) = core_count.get("hw.perflevel0.logicalcpu") {
        res.insert("p_core_count".to_string(), val.to_owned());
    }
    res.insert(
        "name".to_string(),
        cpu_info["machdep.cpu.brand_string"].to_owned(),
    );
    res.insert(
        "core_count".to_string(),
        cpu_info["machdep.cpu.core_count"].to_owned(),
    );
    res.insert("gpu_core_count".to_string(), get_gpu_cores());

    res
}

fn get_cpu_info() -> HashMap<String, String> {
    let mut res = HashMap::new();
    let mut child = Command::new("sysctl")
        .args(["machdep.cpu"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut info = String::new();
        reader.read_to_string(&mut info).unwrap();

        info.split('\n').for_each(|line| {
            if line.contains("machdep.cpu.brand_string") {
                if let Some((_, brand_string)) = line.split_once(':') {
                    res.insert(
                        "machdep.cpu.brand_string".to_string(),
                        brand_string.trim().to_string(),
                    );
                }
            } else if line.contains("machdep.cpu.core_count") {
                if let Some((_, brand_string)) = line.split_once(':') {
                    res.insert(
                        "machdep.cpu.core_count".to_string(),
                        brand_string.trim().to_string(),
                    );
                }
            }
        })
    }

    res
}

fn get_core_count() -> HashMap<String, String> {
    let mut res = HashMap::new();
    let mut child = Command::new("sysctl")
        .args(["hw.perflevel0.logicalcpu", "hw.perflevel1.logicalcpu"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut info = String::new();
        reader.read_to_string(&mut info).unwrap();

        info.split('\n').for_each(|line| {
            if line.contains("hw.perflevel0.logicalcpu") {
                if let Some((_, brand_string)) = line.split_once(':') {
                    res.insert(
                        "hw.perflevel0.logicalcpu".to_string(),
                        brand_string.trim().to_string(),
                    );
                }
            } else if line.contains("hw.perflevel1.logicalcpu") {
                if let Some((_, brand_string)) = line.split_once(':') {
                    res.insert(
                        "hw.perflevel1.logicalcpu".to_string(),
                        brand_string.trim().to_string(),
                    );
                }
            }
        })
    }

    res
}

fn get_gpu_cores() -> String {
    let mut child = Command::new("system_profiler")
        .args(["-detailLevel", "basic", "SPDisplaysDataType"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut info = String::new();
        reader.read_to_string(&mut info).unwrap();

        for line in info.lines() {
            if line.contains("Total Number of Cores") {
                let parts: Vec<&str> = line.split(':').collect();
                if let Some(cores) = parts.get(1) {
                    return cores.to_string();
                }
            }
        }
    }

    "?".to_string()
}
