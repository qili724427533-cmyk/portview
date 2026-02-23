// 网络连接类型定义
export interface TcpConnection {
  protocol: string;
  local_addr: string;
  local_port: number;
  remote_addr: string;
  remote_port: number;
  state: string;
  pid: number | null;
  process_name: string | null;
  icon: string | null; // Base64 encoded icon data
  start_time: number | null; // Process start time in seconds since Unix epoch
  fill_column: string; // Fill column for filling remaining space
  isNew?: boolean; // 标记是否为新连接
  hasChanged?: boolean; // 标记连接状态是否有变化
  isDeleted?: boolean; // 标记是否为即将删除的连接
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
