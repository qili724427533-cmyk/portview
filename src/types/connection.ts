// 网络连接类型定义
export interface TcpConnection {
  id: string; // 稳定唯一标识（协议+五元组），用于行 key、选中比较与差分
  protocol: string;
  local_addr: string;
  local_port: number;
  remote_addr: string;
  remote_port: number;
  state: string;
  pid: number | null;
  process_name: string | null;
  exe_path: string | null; // 可执行文件路径，用于从快照的 icons 映射中取图标
  icon: string | null; // Base64 encoded icon data（由前端从快照合并而来）
  start_time: number | null; // Process start time in seconds since Unix epoch
  isNew?: boolean; // 标记是否为新连接
  hasChanged?: boolean; // 标记连接状态是否有变化
  isDeleted?: boolean; // 标记是否为即将删除的连接
}

// get_connections 返回快照：图标按 exe_path 去重，避免每行重复传输
export interface NetRate {
  down_bps: number; // 系统下载速率（字节/秒）
  up_bps: number; // 系统上传速率（字节/秒）
  total_down: number; // 会话累计下载（字节，应用启动以来）
  total_up: number; // 会话累计上传（字节，应用启动以来）
}

export interface ConnectionsSnapshot {
  connections: TcpConnection[];
  icons: Record<string, string>;
  net_rate: NetRate;
}

// 进程详情类型定义
export interface ProcessDetails {
  pid: number;
  name: string;
  command_line: string;
  executable_path: string;
  memory_usage: number;
  cpu_usage: number;
  parent_pid: number | null; // 修复：与后端 Option<u32> 保持一致
  start_time: number;
}

// 排序相关类型
export type SortColumn =
  | 'process_name'
  | 'pid'
  | 'protocol'
  | 'local_addr'
  | 'local_port'
  | 'remote_addr'
  | 'remote_port'
  | 'state'
  | 'start_time';

export type SortDirection = 'asc' | 'desc';

// 排序值类型（用于类型安全）
export type SortValue = string | number | null;

// 过滤器类型
export interface FilterState {
  protocol: 'all' | 'TCP' | 'UDP';
  state: string;
  searchProcessName: string;
  searchLocalPort: string;
}

// 状态栏信息类型
export interface StatusBarInfo {
  totalConnections: number;
  tcpCount: number;
  udpCount: number;
  lastRefresh: string;
}
