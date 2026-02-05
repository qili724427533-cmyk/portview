use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{System, Process, Pid};
use std::fs;
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConnection {
    pub protocol: String,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub icon: Option<String>, // Base64 encoded icon data
    pub start_time: Option<u64>, // Process start time in seconds since Unix epoch
    pub fill_column: String, // Fill column for filling remaining space
}

// 定义进程详情结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub command_line: String,
    pub executable_path: String,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
}

// 将 netstat2 的 TCP 状态转换为字符串
fn tcp_state_to_string(state: TcpState) -> String {
    match state {
        TcpState::Established => "ESTABLISHED".to_string(),
        TcpState::SynSent => "SYN_SENT".to_string(),
        TcpState::SynReceived => "SYN_RECV".to_string(),
        TcpState::FinWait1 => "FIN_WAIT1".to_string(),
        TcpState::FinWait2 => "FIN_WAIT2".to_string(),
        TcpState::TimeWait => "TIME_WAIT".to_string(),
        TcpState::Closed => "CLOSED".to_string(),
        TcpState::CloseWait => "CLOSE_WAIT".to_string(),
        TcpState::LastAck => "LAST_ACK".to_string(),
        TcpState::Listen => "LISTEN".to_string(),
        TcpState::Closing => "CLOSING".to_string(),
        TcpState::DeleteTcb => "DELETE_TCB".to_string(),
        TcpState::Unknown => "UNKNOWN".to_string(),
    }
}

// 获取进程的应用图标
#[cfg(target_os = "windows")]
fn get_process_icon_by_pid(pid: u32) -> Option<String> {
    use windows_icons::IconSource;

    // 尝试通过PID获取进程的可执行文件路径
    if let Ok(exe_path) = get_executable_path_from_pid(pid) {
        // 尝试从可执行文件中提取图标
        if let Ok(icon_data) = IconSource::from_path(&exe_path).and_then(|source| source.get_icon(None)) {
            // 将图标数据编码为Base64
            return Some(base64::encode(&icon_data));
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn get_process_icon_by_pid(_pid: u32) -> Option<String> {
    // 对于非Windows平台，暂时返回None
    // 实现跨平台图标获取较为复杂，需要专门的库
    None
}

// 通过PID获取可执行文件路径的辅助函数
#[cfg(target_os = "windows")]
fn get_executable_path_from_pid(pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
    use winapi::shared::minwindef::DWORD;
    use std::ptr;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if handle.is_null() {
            return Err("Failed to open process".into());
        }

        let mut buffer: [u16; 1024] = [0; 1024];
        let mut size: DWORD = buffer.len() as DWORD;

        let success = winapi::um::psapi::GetModuleFileNameExW(
            handle,
            ptr::null_mut(),
            buffer.as_mut_ptr(),
            size,
        );

        CloseHandle(handle);

        if success == 0 {
            return Err("Failed to get module filename".into());
        }

        let os_string = OsStr::from_wide(&buffer[..(size as usize)]);
        Ok(os_string.to_string_lossy().to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn get_executable_path_from_pid(_pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    // 非Windows平台暂时不实现
    Ok(String::new())
}

// 获取系统网络连接列表
#[tauri::command]
async fn get_connections() -> Result<Vec<TcpConnection>, String> {
    let mut connections = Vec::new();

    // 设置地址族标志 (IPv4 和 IPv6)
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    // 设置协议标志 (TCP 和 UDP)
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

    // 获取网络连接信息
    let sockets_info = get_sockets_info(af_flags, proto_flags).map_err(|e| e.to_string())?;

    // 创建系统信息实例以获取进程名称
    let mut system = System::new_all();
    system.refresh_processes();

    for si in sockets_info {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                let protocol = "TCP".to_string();

                // 处理本地地址和端口
                let local_addr = tcp_si.local_addr.to_string();
                let local_port = tcp_si.local_port;

                // 处理远程地址和端口
                let remote_addr = tcp_si.remote_addr.to_string();
                let remote_port = tcp_si.remote_port;

                // 获取连接状态
                let state = tcp_state_to_string(tcp_si.state);

                // 获取 PID (如果有)
                let pid = if !si.associated_pids.is_empty() {
                    Some(si.associated_pids[0]) // 取第一个关联的PID
                } else {
                    None
                };

                // 根据 PID 获取进程名称
                let process_name = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.name().to_string())
                    } else {
                        // 如果无法获取进程信息，可能是内核进程，显示特殊标识
                        Some("[KERNEL]".to_string())
                    }
                } else {
                    None
                };

                // 获取进程图标
                let icon = if let Some(pid_val) = pid {
                    if system.process((pid_val as usize).into()).is_some() {
                        get_process_icon_by_pid(pid_val)
                    } else {
                        // 内核进程的图标可以设置为特殊图标
                        None
                    }
                } else {
                    None
                };

                // 获取进程启动时间
                let start_time = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.start_time())
                    } else {
                        // 内核进程的时间信息可能不可用，返回0
                        Some(0)
                    }
                } else {
                    None
                };

                connections.push(TcpConnection {
                    protocol,
                    local_addr,
                    local_port,
                    remote_addr,
                    remote_port,
                    state,
                    pid,
                    process_name,
                    icon,
                    start_time,
                    fill_column: String::new(), // 填充列，留空
                });
            },
            ProtocolSocketInfo::Udp(udp_si) => {
                let protocol = "UDP".to_string();

                // 处理本地地址和端口
                let local_addr = udp_si.local_addr.to_string();
                let local_port = udp_si.local_port;

                // UDP 没有远程地址和端口的概念，通常设置为通配符
                let remote_addr = "*".to_string();
                let remote_port = 0;

                // UDP 没有连接状态，设置为 UNCONN
                let state = "UNCONN".to_string();

                // 获取 PID (如果有)
                let pid = if !si.associated_pids.is_empty() {
                    Some(si.associated_pids[0]) // 取第一个关联的PID
                } else {
                    None
                };

                // 根据 PID 获取进程名称
                let process_name = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.name().to_string())
                    } else {
                        // 如果无法获取进程信息，可能是内核进程，显示特殊标识
                        Some("[KERNEL]".to_string())
                    }
                } else {
                    None
                };

                // 获取进程图标
                let icon = if let Some(pid_val) = pid {
                    if system.process((pid_val as usize).into()).is_some() {
                        get_process_icon_by_pid(pid_val)
                    } else {
                        // 内核进程的图标可以设置为特殊图标
                        None
                    }
                } else {
                    None
                };

                // 获取进程启动时间
                let start_time = if let Some(pid_val) = pid {
                    if let Some(process) = system.process((pid_val as usize).into()) {
                        Some(process.start_time())
                    } else {
                        // 内核进程的时间信息可能不可用，返回0
                        Some(0)
                    }
                } else {
                    None
                };

                connections.push(TcpConnection {
                    protocol,
                    local_addr,
                    local_port,
                    remote_addr,
                    remote_port,
                    state,
                    pid,
                    process_name,
                    icon,
                    start_time,
                    fill_column: String::new(), // 填充列，留空
                });
            }
        }
    }

    Ok(connections)
}

// 获取进程详情
#[tauri::command]
async fn get_process_details(pid: u32) -> Result<ProcessDetails, String> {
    let mut system = System::new_all();
    system.refresh_processes();

    if let Some(process) = system.process(Pid::from_u32(pid)) {
        Ok(ProcessDetails {
            pid,
            name: process.name().to_string(),
            command_line: process.cmd().join(" "),
            executable_path: process.exe().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            memory_usage: process.memory(),
            cpu_usage: process.cpu_usage(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            start_time: process.start_time(),
        })
    } else {
        Err(format!("Process with PID {} not found", pid))
    }
}

// 杀死进程
#[tauri::command]
async fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::process::Command;

        // 尝试使用 SIGTERM 优雅地终止进程
        let output = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output()
            .map_err(|e| format!("Failed to execute kill command: {}", e))?;

        if !output.status.success() {
            // 如果 SIGTERM 失败，尝试使用 SIGKILL 强制终止
            let output = Command::new("kill")
                .arg("-KILL")
                .arg(pid.to_string())
                .output()
                .map_err(|e| format!("Failed to execute kill command: {}", e))?;

            if !output.status.success() {
                return Err(format!("Failed to kill process with PID {}", pid));
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    {
        use std::mem;
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::winnt::PROCESS_TERMINATE;
        use winapi::um::processthreadsapi::TerminateProcess;

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle.is_null() {
                return Err(format!("Failed to open process with PID {}", pid));
            }

            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);

            if result != 0 {
                Ok(())
            } else {
                Err(format!("Failed to kill process with PID {}", pid))
            }
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_connections, get_process_details, kill_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
