#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub cpu_model: Option<String>,
    pub logical_cpu_count: Option<usize>,
    pub physical_cpu_count: Option<usize>,
    pub memory_total_mb: Option<u64>,
    pub memory_peak_mb: Option<u64>,
}

pub fn collect_system_info() -> SystemInfo {
    SystemInfo {
        cpu_model: cpu_model(),
        logical_cpu_count: logical_cpu_count(),
        physical_cpu_count: physical_cpu_count(),
        memory_total_mb: memory_total_mb(),
        memory_peak_mb: memory_peak_mb(),
    }
}

fn logical_cpu_count() -> Option<usize> {
    std::thread::available_parallelism().ok().map(usize::from)
}

fn cpu_model() -> Option<String> {
    cpu_model_from_proc_cpuinfo()
        .or_else(|| env_non_empty("PROCESSOR_IDENTIFIER"))
        .or_else(|| env_non_empty("PROCESSOR_ARCHITECTURE"))
}

fn physical_cpu_count() -> Option<usize> {
    None
}

fn memory_total_mb() -> Option<u64> {
    meminfo_kb("MemTotal").map(kb_to_mb)
}

fn memory_peak_mb() -> Option<u64> {
    status_kb("VmHWM").map(kb_to_mb)
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn cpu_model_from_proc_cpuinfo() -> Option<String> {
    let contents = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    contents.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if matches!(key.trim(), "model name" | "Hardware" | "Processor") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
        None
    })
}

fn meminfo_kb(key: &str) -> Option<u64> {
    let contents = std::fs::read_to_string("/proc/meminfo").ok()?;
    parse_kb_value(&contents, key)
}

fn status_kb(key: &str) -> Option<u64> {
    let contents = std::fs::read_to_string("/proc/self/status").ok()?;
    parse_kb_value(&contents, key)
}

fn parse_kb_value(contents: &str, key: &str) -> Option<u64> {
    contents.lines().find_map(|line| {
        let (line_key, value) = line.split_once(':')?;
        if line_key.trim() != key {
            return None;
        }
        value.split_whitespace().next()?.parse().ok()
    })
}

fn kb_to_mb(kb: u64) -> u64 {
    kb.div_ceil(1024)
}

#[cfg(test)]
mod tests {
    use super::{collect_system_info, kb_to_mb, parse_kb_value};

    #[test]
    fn collect_system_info_does_not_fail() {
        let info = collect_system_info();
        assert!(info.logical_cpu_count.unwrap_or(1) > 0);
    }

    #[test]
    fn parses_kb_values() {
        let contents = "MemTotal:       16384256 kB\nVmHWM:\t1234 kB\n";

        assert_eq!(parse_kb_value(contents, "MemTotal"), Some(16_384_256));
        assert_eq!(parse_kb_value(contents, "VmHWM"), Some(1_234));
        assert_eq!(parse_kb_value(contents, "Missing"), None);
    }

    #[test]
    fn rounds_kb_up_to_mb() {
        assert_eq!(kb_to_mb(1), 1);
        assert_eq!(kb_to_mb(1024), 1);
        assert_eq!(kb_to_mb(1025), 2);
    }
}
