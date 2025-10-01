use protocol::gateway::state::{DiskState, MemState, NetState, State, SystemState};
use std::fs;
use std::sync::{Arc, LazyLock, Mutex};
use sysinfo::{Disks, Networks, System};

#[derive(Debug, Default)]
pub struct GatewayState {
    pub state: Arc<Mutex<State>>,
}
impl GatewayState {
    /// 刷新状态，并清空计数器。返回旧状态。
    pub fn refresh(&self) -> State {
        let system_state = self.build_system_state();

        // 锁定
        let mut state_guard = self.state.lock().unwrap();
        // 旧状态
        let old = state_guard.clone();

        // 更新状态并重置计数器
        *state_guard = State {
            timestamp: chrono::Local::now().timestamp_millis(),
            system_state,
            counter: Default::default(),
            moment_counter: old.moment_counter.clone(),
        };

        old
    }

    fn build_system_state(&self) -> SystemState {
        let mut system_state = SystemState::default();

        let mut sys = System::new();

        sys.refresh_memory();
        sys.refresh_cpu_all();

        system_state.os = format!(
            "{} {}",
            System::name().unwrap_or_default(),
            System::os_version().unwrap_or_default()
        );
        system_state.host_name = System::host_name().unwrap_or_default();

        system_state.cpu_usage = sys.global_cpu_usage();
        system_state.mem_state = MemState {
            total: sys.total_memory(),
            free: sys.free_memory(),
            used: sys.used_memory(),
        };

        let disks = Disks::new_with_refreshed_list();
        let root_disk = disks
            .list()
            .iter()
            .find(|d| d.mount_point().to_str() == Some("/"));
        if let Some(root_disk) = root_disk {
            system_state.disk_state = DiskState {
                total: root_disk.total_space(),
                free: root_disk.available_space(),
            };
        }

        let networks = Networks::new_with_refreshed_list();
        let net = networks.list().get("eth0");
        if let Some(net) = net {
            system_state.net_state = NetState {
                rx: net.total_received(),
                tx: net.total_transmitted(),
                tcp_conn_count: Self::get_http_connections().unwrap_or(0),
            };
        }
        // let pid = std::process::id();
        // let pid = Pid::from_u32(pid);
        // sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        // let process = sys.process(pid);
        // if let Some(process) = process {
        //     println!("{:?}", process.open_files());
        // }

        system_state
    }

    fn get_http_connections() -> Result<usize, std::io::Error> {
        let content = fs::read_to_string("/proc/net/sockstat")?;
        for line in content.lines() {
            if line.starts_with("TCP:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    return Ok(parts[2].parse().unwrap_or(0));
                }
            }
        }
        Ok(0)
    }

    /// 更新请求计数
    pub fn inc_request_count(&self, n: usize) {
        self.state.lock().unwrap().counter.request_count += n;
    }

    pub fn inc_status_request_count(&self, status_code: u16, n: usize) {
        match status_code {
            200..300 => self.state.lock().unwrap().counter.response_2xx_count += n,
            300..400 => self.state.lock().unwrap().counter.response_3xx_count += n,
            400..500 => self.state.lock().unwrap().counter.response_4xx_count += n,
            500..600 => self.state.lock().unwrap().counter.response_5xx_count += n,
            _ => {}
        }
    }

    pub fn inc_response_time(&self, time: usize) {
        self.state.lock().unwrap().counter.response_time_since_last += time;
    }

    pub fn inc_http_connect_count(&self, n: isize) {
        self.state.lock().unwrap().moment_counter.http_connect_count += n;
    }

    pub fn inc_request_invalid_count(&self, n: usize) {
        self.state.lock().unwrap().counter.request_invalid_count += n;
    }

    pub fn get_http_connect_count(&self) -> isize {
        self.state.lock().unwrap().moment_counter.http_connect_count
    }
}

pub static STATE: LazyLock<GatewayState> = LazyLock::new(|| GatewayState::default());
