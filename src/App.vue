<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

// 定义连接数据类型
interface TcpConnection {
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
}

// 定义进程详情类型
interface ProcessDetails {
  pid: number;
  name: string;
  command_line: string;
  executable_path: string;
  memory_usage: number;
  cpu_usage: number;
  parent_pid: number;
  start_time: number;
}

const connections = ref<TcpConnection[]>([]);
const isLoading = ref(false);
const refreshInterval = ref<number | null>(null);

// 右键菜单相关状态
const showContextMenu = ref(false);
const contextMenuPosition = ref({ x: 0, y: 0 });
const selectedConnection = ref<TcpConnection | null>(null);

// 选中行相关状态
const clickedConnection = ref<TcpConnection | null>(null);

// 进程详情弹窗相关状态
const showProcessDetails = ref(false);
const processDetails = ref<ProcessDetails | null>(null);


// 排序相关状态
const sortColumn = ref<'process_name' | 'pid' | 'protocol' | 'local_addr' | 'local_port' | 'remote_addr' | 'remote_port' | 'state' | 'start_time' | 'fill_column' | null>(null);
const sortDirection = ref<'asc' | 'desc'>('asc'); // 'asc' 升序, 'desc' 降序

// 列宽状态
const columnWidths = ref({
  process_name: 180, // 缩减进程名称列宽度
  pid: 80,          // 缩减PID列宽度
  protocol: 60,     // 缩减协议列宽度
  local_addr: 120,  // 缩减本地地址列宽度
  local_port: 70,   // 缩减本地端口列宽度
  remote_addr: 120, // 缩减远程地址列宽度
  remote_port: 70,  // 缩减远程端口列宽度
  state: 80,        // 缩减状态列宽度
  start_time: 150,  // 启动时间列宽度
  fill_column: 1    // 填充列，宽度设为1，剩余空间由flex-grow填充
});

// 计算Grid模板列
const gridTemplateColumns = computed(() => {
  return `${columnWidths.value.process_name}px ${columnWidths.value.pid}px ${columnWidths.value.protocol}px ${columnWidths.value.local_addr}px ${columnWidths.value.local_port}px ${columnWidths.value.remote_addr}px ${columnWidths.value.remote_port}px ${columnWidths.value.state}px ${columnWidths.value.start_time}px ${columnWidths.value.fill_column}px`;
});

let resizingColumn = ref<string | null>(null);
let startX = ref(0);
let startWidth = ref(0);

// 菜单栏筛选条件
const filterProtocol = ref<'all' | 'TCP' | 'UDP'>('all');
const filterState = ref('all');
const searchProcessName = ref('');
const searchLocalAddr = ref('');

// 状态栏信息
const statusBarInfo = ref({
  totalConnections: 0,
  tcpConnections: 0,
  udpConnections: 0,
  kernelConnections: 0,
  lastUpdate: new Date().toLocaleTimeString(),
  refreshInterval: null as number | null
});

// 获取网络连接列表
async function loadConnections() {
  isLoading.value = true;
  try {
    const result: TcpConnection[] = await invoke("get_connections");

    // 应用筛选条件
    let filteredResult = result;

    // 协议筛选
    if (filterProtocol.value !== 'all') {
      filteredResult = filteredResult.filter(conn => conn.protocol === filterProtocol.value);
    }

    // 状态筛选
    if (filterState.value !== 'all') {
      filteredResult = filteredResult.filter(conn => conn.state === filterState.value);
    }

    // 进程名搜索
    if (searchProcessName.value.trim() !== '') {
      const searchTerm = searchProcessName.value.toLowerCase().trim();
      filteredResult = filteredResult.filter(conn =>
        conn.process_name && conn.process_name.toLowerCase().includes(searchTerm)
      );
    }

    // 本地地址搜索
    if (searchLocalAddr.value.trim() !== '') {
      const searchTerm = searchLocalAddr.value.toLowerCase().trim();
      filteredResult = filteredResult.filter(conn =>
        conn.local_addr.toLowerCase().includes(searchTerm)
      );
    }

    connections.value = filteredResult;

    // 更新状态栏信息
    updateStatusBarInfo(result); // 注意：这里仍然使用原始结果更新状态栏

    // 应用排序
    applySorting();
  } catch (error) {
    console.error("获取连接列表失败:", error);
    alert(`获取连接列表失败: ${error}`);
  } finally {
    isLoading.value = false;
  }
}

// 设置协议筛选
function setProtocolFilter(protocol: 'all' | 'TCP' | 'UDP') {
  filterProtocol.value = protocol;
  applyFiltersAndSearch();
}

// 应用筛选和搜索
function applyFiltersAndSearch() {
  // 重新获取数据以应用筛选条件
  loadConnections();
}

// 更新状态栏信息
function updateStatusBarInfo(connections: TcpConnection[]) {
  statusBarInfo.value.totalConnections = connections.length;
  statusBarInfo.value.tcpConnections = connections.filter(conn => conn.protocol === 'TCP').length;
  statusBarInfo.value.udpConnections = connections.filter(conn => conn.protocol === 'UDP').length;
  statusBarInfo.value.kernelConnections = connections.filter(conn =>
    conn.process_name === '[KERNEL]' || (conn.process_name && conn.process_name.includes('[KERNEL]'))
  ).length;
  statusBarInfo.value.lastUpdate = new Date().toLocaleTimeString();
}

// 获取进程详情
async function loadProcessDetails(pid: number) {
  try {
    const details: ProcessDetails = await invoke("get_process_details", { pid });
    processDetails.value = details;
  } catch (error) {
    console.error("获取进程详情失败:", error);
    alert(`获取进程详情失败: ${error}`);
  }
}

// 显示进程详情弹窗
async function showProcessDetailsDialog(conn: TcpConnection) {
  if (conn.pid) {
    await loadProcessDetails(conn.pid);
    showProcessDetails.value = true;
  }
}

// 杀死进程
async function killProcess(conn: TcpConnection) {
  if (conn.pid) {
    try {
      await invoke("kill_process", { pid: conn.pid });
      // 成功杀死进程后，重新加载连接列表
      await loadConnections();
      alert(`成功杀死进程: ${conn.process_name || 'Unknown'} (PID: ${conn.pid})`);
    } catch (error) {
      console.error("杀死进程失败:", error);
      alert(`杀死进程失败: ${error}`);
    }
  }
}

// 显示右键菜单
function showContextMenuHandler(conn: TcpConnection, event: MouseEvent) {
  event.preventDefault(); // 阻止默认右键菜单

  selectedConnection.value = conn;
  showContextMenu.value = true;

  // 计算菜单位置，确保不会超出屏幕边界
  const menuWidth = 150; // 菜单宽度
  const menuHeight = 60; // 菜单高度

  let posX = event.clientX;
  let posY = event.clientY;

  // 检查是否超出右侧屏幕边界
  if (posX + menuWidth > window.innerWidth) {
    posX = window.innerWidth - menuWidth - 5; // 5px 边距
  }

  // 检查是否超出底部屏幕边界
  if (posY + menuHeight > window.innerHeight) {
    posY = window.innerHeight - menuHeight - 5; // 5px 边距
  }

  contextMenuPosition.value = {
    x: posX,
    y: posY
  };
}

// 计算右键菜单样式
const contextMenuStyle = computed(() => {
  return {
    top: `${contextMenuPosition.value.y}px`,
    left: `${contextMenuPosition.value.x}px`,
    position: 'fixed' as const,
    zIndex: 1000
  };
});

// 隐藏右键菜单
function hideContextMenu() {
  showContextMenu.value = false;
}

// 排序函数
function applySorting() {
  if (!sortColumn.value) return;

  connections.value.sort((a, b) => {
    let valueA: any, valueB: any;

    switch (sortColumn.value) {
      case 'process_name':
        valueA = a.process_name || '';
        valueB = b.process_name || '';
        return sortDirection.value === 'asc'
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case 'pid':
        valueA = a.pid || 0;
        valueB = b.pid || 0;
        return sortDirection.value === 'asc'
          ? valueA - valueB
          : valueB - valueA;
      case 'protocol':
        valueA = a.protocol || '';
        valueB = b.protocol || '';
        return sortDirection.value === 'asc'
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case 'local_addr':
        valueA = a.local_addr || '';
        valueB = b.local_addr || '';
        return sortDirection.value === 'asc'
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case 'local_port':
        valueA = a.local_port || 0;
        valueB = b.local_port || 0;
        return sortDirection.value === 'asc'
          ? valueA - valueB
          : valueB - valueA;
      case 'remote_addr':
        valueA = a.remote_addr || '';
        valueB = b.remote_addr || '';
        return sortDirection.value === 'asc'
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case 'remote_port':
        valueA = a.remote_port || 0;
        valueB = b.remote_port || 0;
        return sortDirection.value === 'asc'
          ? valueA - valueB
          : valueB - valueA;
      case 'state':
        valueA = a.state || '';
        valueB = b.state || '';
        return sortDirection.value === 'asc'
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case 'start_time':
        valueA = a.start_time || 0;
        valueB = b.start_time || 0;
        return sortDirection.value === 'asc'
          ? valueA - valueB
          : valueB - valueA;
      case 'fill_column':
        // 填充列不参与实际排序，总是返回0
        return 0;
      default:
        return 0;
    }
  });
}

// 切换列排序
async function toggleSort(column: 'process_name' | 'pid' | 'protocol' | 'local_addr' | 'local_port' | 'remote_addr' | 'remote_port' | 'state' | 'start_time' | 'fill_column') {
  // 在排序前重新获取最新连接数据
  await loadConnections();

  if (sortColumn.value === column) {
    // 如果当前列已经是排序列，则切换排序方向
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
  } else {
    // 如果切换到新列，则默认升序
    sortColumn.value = column;
    sortDirection.value = 'asc';
  }

  // 重新应用排序
  applySorting();
}



// 通用的鼠标按下处理函数
function handleMouseDown(event: MouseEvent) {
  // 只响应左键点击
  if (event.button !== 0) return;

  // 获取被点击的列的data-column属性
  const target = event.target as HTMLElement;
  const th = target.closest('th') as HTMLElement;
  if (!th) return;

  const columnName = th.getAttribute('data-column') as keyof typeof columnWidths.value;
  if (!columnName) return;

  startResize(columnName, event);
}

// 开始调整列宽
function startResize(columnName: keyof typeof columnWidths.value, event: MouseEvent) {
  resizingColumn.value = columnName;
  startX.value = event.clientX;
  startWidth.value = columnWidths.value[columnName];

  // 添加resizing类到整个表格以提供视觉反馈
  const table = document.querySelector('.connections-table');
  if (table) {
    table.classList.add('resizing');
  }

  const handleMouseMove = (e: MouseEvent) => handleResize(e);
  const handleMouseUp = (_e: MouseEvent) => stopResize(table);

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);

  // 防止文本选择和默认的拖拽行为
  event.preventDefault();

  // 确保在鼠标抬起时移除事件监听器
  const cleanup = () => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.removeEventListener('mouseleave', cleanup);
  };

  document.addEventListener('mouseleave', cleanup);
}

// 调整列宽
function handleResize(event: MouseEvent) {
  if (resizingColumn.value) {
    const diff = event.clientX - startX.value;
    // 根据不同列设置不同的最小宽度，以避免文字重叠
    let minWidth = 40; // 默认最小宽度
    switch(resizingColumn.value) {
      case 'process_name':
        minWidth = 100; // 进程名称需要更多空间（包含图标）
        break;
      case 'local_addr':
      case 'remote_addr':
        minWidth = 100; // IP地址需要更多空间
        break;
      case 'state':
        minWidth = 60; // 状态也需要一定空间
        break;
      case 'protocol':
        minWidth = 50; // 协议列最小宽度
        break;
      case 'pid':
        minWidth = 50; // PID列最小宽度
        break;
      case 'local_port':
      case 'remote_port':
        minWidth = 50; // 端口列最小宽度
        break;
      case 'start_time':
        minWidth = 120; // 启动时间列最小宽度
        break;
      case 'fill_column':
        minWidth = 1; // 填充列最小宽度
        break;
      default:
        minWidth = 40; // 其他列的最小宽度
    }

    const newWidth = Math.max(startWidth.value + diff, minWidth);

    // 更新当前列的宽度
    columnWidths.value[resizingColumn.value as keyof typeof columnWidths.value] = newWidth;
  }
}

// 停止调整列宽
function stopResize(table: Element | null) {
  resizingColumn.value = null;

  // 移除resizing类
  if (table) {
    table.classList.remove('resizing');
  }

  // 触发一次重绘以确保布局正确
  document.body.offsetHeight;
}

// 页面加载完成后自动获取连接列表
onMounted(() => {
  loadConnections();

  // 监听窗口大小变化事件
  window.addEventListener('resize', handleWindowResize);

  // 监听文档上的点击事件，用于隐藏右键菜单
  document.addEventListener('click', hideContextMenu);
});

// 组件卸载时清理定时器和事件监听器
onUnmounted(() => {
  if (refreshInterval.value !== null) {
    clearInterval(refreshInterval.value);
  }

  // 移除窗口大小变化事件监听器
  window.removeEventListener('resize', handleWindowResize);

  // 移除文档点击事件监听器
  document.removeEventListener('click', hideContextMenu);
});

// 处理窗口大小变化
function handleWindowResize() {
  // 在窗口大小变化时，重新应用当前的列宽设置
  setTimeout(() => {
    // 强制浏览器重新计算布局
    document.body.offsetHeight;

    // 根据窗口大小调整表格行为
    adjustTableBehavior();
  }, 150); // 增加延时以确保窗口大小变化完全结束
}

// 格式化内存使用量显示
function formatMemoryUsage(memoryInBytes: number): string {
  // memoryInBytes 是以字节为单位的数值
  if (memoryInBytes < 1024) {
    return `${memoryInBytes} B`; // 小于1KB，显示字节数
  } else if (memoryInBytes < 1024 * 1024) {
    const memoryInKB = (memoryInBytes / 1024).toFixed(2);
    return `${memoryInKB} KB`;
  } else if (memoryInBytes < 1024 * 1024 * 1024) {
    const memoryInMB = (memoryInBytes / (1024 * 1024)).toFixed(2);
    return `${memoryInMB} MB`;
  } else {
    const memoryInGB = (memoryInBytes / (1024 * 1024 * 1024)).toFixed(2);
    return `${memoryInGB} GB`;
  }
}

// 格式化日期时间显示
function formatDate(timestamp: number | null): string {
  if (timestamp === null || timestamp === 0) {
    return '-';
  }

  // 将秒级时间戳转换为毫秒级时间戳
  const date = new Date(timestamp * 1000);

  // 获取年、月、日、小时、分钟和秒
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0'); // 月份从0开始，需要+1
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');

  // 返回格式化的日期时间字符串
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}


// 判断是否为内核进程
function isKernelProcess(processName: string | null): boolean {
  return processName === '[KERNEL]' || (processName !== null && processName.includes('[KERNEL]'));
}

// 根据窗口大小调整表格行为
function adjustTableBehavior() {
  const tableWrapper = document.querySelector('.table-wrapper') as HTMLElement;

  if (tableWrapper) {
    // 无论窗口大小，都让表格填充可用空间
    tableWrapper.style.width = '100%';
    tableWrapper.style.display = 'block';
  }
}
</script>

<template>
  <div class="container">
    <!-- 菜单栏 -->
    <div class="menu-bar">
      <div class="menu-group">
        <label class="menu-label">协议:</label>
        <div class="protocol-buttons">
          <button
            :class="['protocol-btn', { active: filterProtocol === 'all' }]"
            @click="setProtocolFilter('all')"
          >
            全部
          </button>
          <button
            :class="['protocol-btn', { active: filterProtocol === 'TCP' }]"
            @click="setProtocolFilter('TCP')"
          >
            TCP
          </button>
          <button
            :class="['protocol-btn', { active: filterProtocol === 'UDP' }]"
            @click="setProtocolFilter('UDP')"
          >
            UDP
          </button>
        </div>
      </div>

      <div class="menu-group">
        <label class="menu-label">状态:</label>
        <select
          v-model="filterState"
          @change="applyFiltersAndSearch"
          class="menu-select"
        >
          <option value="all">全部</option>
          <option value="LISTEN">监听</option>
          <option value="ESTABLISHED">已建立</option>
          <option value="TIME_WAIT">等待</option>
          <option value="CLOSE_WAIT">关闭等待</option>
          <option value="SYN_SENT">同步发送</option>
          <option value="SYN_RECV">同步接收</option>
          <option value="FIN_WAIT1">结束等待1</option>
          <option value="FIN_WAIT2">结束等待2</option>
          <option value="LAST_ACK">最后确认</option>
          <option value="CLOSING">关闭中</option>
          <option value="UNCONN">未连接</option>
        </select>
      </div>

      <div class="menu-group">
        <label class="menu-label">搜索进程:</label>
        <input
          type="text"
          v-model="searchProcessName"
          @input="applyFiltersAndSearch"
          placeholder="输入进程名称..."
          class="menu-search"
        />
      </div>

      <div class="menu-group">
        <label class="menu-label">本地地址:</label>
        <input
          type="text"
          v-model="searchLocalAddr"
          @input="applyFiltersAndSearch"
          placeholder="输入本地地址..."
          class="menu-search"
        />
      </div>
    </div>

    <div class="connections-table-container">
      <div class="table-wrapper">
        <table class="connections-table">
          <thead>
            <tr>
              <th :style="{ width: columnWidths.process_name + 'px', minWidth: columnWidths.process_name + 'px' }" class="resizable-th" data-column="process_name" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('process_name')">
                  <span class="sortable-header">
                    进程名称
                    <span v-if="sortColumn === 'process_name'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.pid + 'px', minWidth: columnWidths.pid + 'px' }" class="resizable-th" data-column="pid" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('pid')">
                  <span class="sortable-header">
                    PID
                    <span v-if="sortColumn === 'pid'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.protocol + 'px', minWidth: columnWidths.protocol + 'px' }" class="resizable-th" data-column="protocol" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('protocol')">
                  <span class="sortable-header">
                    协议
                    <span v-if="sortColumn === 'protocol'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.local_addr + 'px', minWidth: columnWidths.local_addr + 'px' }" class="resizable-th" data-column="local_addr" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('local_addr')">
                  <span class="sortable-header">
                    本地地址
                    <span v-if="sortColumn === 'local_addr'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.local_port + 'px', minWidth: columnWidths.local_port + 'px' }" class="resizable-th" data-column="local_port" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('local_port')">
                  <span class="sortable-header">
                    本地端口
                    <span v-if="sortColumn === 'local_port'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.remote_addr + 'px', minWidth: columnWidths.remote_addr + 'px' }" class="resizable-th" data-column="remote_addr" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('remote_addr')">
                  <span class="sortable-header">
                    远程地址
                    <span v-if="sortColumn === 'remote_addr'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.remote_port + 'px', minWidth: columnWidths.remote_port + 'px' }" class="resizable-th" data-column="remote_port" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('remote_port')">
                  <span class="sortable-header">
                    远程端口
                    <span v-if="sortColumn === 'remote_port'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.state + 'px', minWidth: columnWidths.state + 'px' }" class="resizable-th" data-column="state" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('state')">
                  <span class="sortable-header">
                    状态
                    <span v-if="sortColumn === 'state'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.start_time + 'px', minWidth: columnWidths.start_time + 'px' }" class="resizable-th" data-column="start_time" @mousedown.left="handleMouseDown">
                <div class="column-header" @click="toggleSort('start_time')">
                  <span class="sortable-header">
                    启动时间
                    <span v-if="sortColumn === 'start_time'" class="sort-indicator">
                      {{ sortDirection === 'asc' ? ' ▲' : ' ▼' }}
                    </span>
                  </span>
                </div>
              </th>
              <th :style="{ width: columnWidths.fill_column + 'px', minWidth: columnWidths.fill_column + 'px' }" class="resizable-th" data-column="fill_column" @mousedown.left="handleMouseDown" style="flex-grow: 1;">
                <div class="column-header">
                  <span class="sortable-header">
                    <!-- 空白列标题，用于填充剩余空间 -->
                  </span>
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(conn, index) in connections"
              :key="index"
              @contextmenu="showContextMenuHandler(conn, $event)"
              @click="clickedConnection = conn"
              @dblclick="showProcessDetailsDialog(conn)"
              :class="{ 'selected-row': clickedConnection === conn }"
            >
              <td :style="{ width: columnWidths.process_name + 'px' }" class="process-name-cell">
                <div class="process-with-icon">
                  <img v-if="conn.icon && !isKernelProcess(conn.process_name)" :src="'data:image/png;base64,' + conn.icon" :alt="conn.process_name || 'Process Icon'" class="process-icon" />
                  <span :class="{ 'kernel-process': isKernelProcess(conn.process_name) }">{{ conn.process_name || '-' }}</span>
                </div>
              </td>
              <td :style="{ width: columnWidths.pid + 'px' }">{{ conn.pid || '-' }}</td>
              <td :style="{ width: columnWidths.protocol + 'px' }">{{ conn.protocol }}</td>
              <td :style="{ width: columnWidths.local_addr + 'px' }">{{ conn.local_addr }}</td>
              <td :style="{ width: columnWidths.local_port + 'px' }">{{ conn.local_port }}</td>
              <td :style="{ width: columnWidths.remote_addr + 'px' }">{{ conn.remote_addr }}</td>
              <td :style="{ width: columnWidths.remote_port + 'px' }">{{ conn.remote_port }}</td>
              <td :style="{ width: columnWidths.state + 'px' }">{{ conn.state }}</td>
              <td :style="{ width: columnWidths.start_time + 'px' }">{{ formatDate(conn.start_time) }}</td>
              <td :style="{ width: columnWidths.fill_column + 'px' }" style="flex-grow: 1;"></td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 右键菜单 -->
    <div
      v-if="showContextMenu"
      class="context-menu"
      :style="contextMenuStyle"
    >
      <ul>
        <li @click="showProcessDetailsDialog(selectedConnection!)">进程详情</li>
        <li @click="killProcess(selectedConnection!)">杀死进程</li>
      </ul>
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <div class="status-item">
        <span class="status-label">总计连接:</span>
        <span class="status-value">{{ statusBarInfo.totalConnections }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">TCP连接:</span>
        <span class="status-value tcp-count">{{ statusBarInfo.tcpConnections }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">UDP连接:</span>
        <span class="status-value udp-count">{{ statusBarInfo.udpConnections }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">内核连接:</span>
        <span class="status-value kernel-count">{{ statusBarInfo.kernelConnections }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">上次更新:</span>
        <span class="status-value">{{ statusBarInfo.lastUpdate }}</span>
      </div>
      <div class="status-item" v-if="statusBarInfo.refreshInterval">
        <span class="status-label">自动刷新:</span>
        <span class="status-value">{{ statusBarInfo.refreshInterval }}秒</span>
      </div>
    </div>

    <!-- 进程详情弹窗 -->
    <div v-if="showProcessDetails" class="modal-overlay" @click="showProcessDetails = false">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>进程详情</h3>
          <button class="close-button" @click="showProcessDetails = false">×</button>
        </div>
        <div v-if="processDetails" class="process-details">
          <p><strong>PID:</strong> <span>{{ processDetails.pid }}</span></p>
          <p><strong>名称:</strong> <span>{{ processDetails.name }}</span></p>
          <p><strong>命令行:</strong> <span>{{ processDetails.command_line }}</span></p>
          <p><strong>执行路径:</strong> <span>{{ processDetails.executable_path }}</span></p>
          <p><strong>内存使用:</strong> <span>{{ formatMemoryUsage(processDetails.memory_usage) }}</span></p>
          <p><strong>CPU使用率:</strong> <span>{{ processDetails.cpu_usage }}%</span></p>
          <p><strong>父进程PID:</strong> <span>{{ processDetails.parent_pid }}</span></p>
          <p><strong>启动时间:</strong> <span>{{ formatDate(processDetails.start_time) }}</span></p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.connections-table-container {
  width: 100%;
  overflow-x: auto;  /* 当内容超出宽度时显示横向滚动条 */
  overflow-y: auto;
  flex: 1 1 auto;   /* 允许增长、收缩，基础大小为自动 */
  margin-top: 0;
  min-height: 0;    /* 允许容器收缩 */
  display: flex;
  flex-direction: column;
}

.table-grid-wrapper {
  width: 100%;     /* 填满容器宽度 */
  display: flex;
  flex-direction: column;
  overflow-x: auto; /* 确保水平滚动可用 */
}

.grid-header,
.grid-data-row {
  display: grid;
  grid-template-columns: v-bind(gridTemplateColumns); /* 使用动态Grid模板 */
  min-width: max-content; /* 确保网格至少适应内容宽度 */
}

.grid-header {
  background-color: #f9fafb;
  color: #111827;
  text-align: left;
  font-weight: 600;
  border-bottom: 2px solid #e5e7eb;
  height: 24px;
  flex-shrink: 0; /* 防止头部收缩 */
}

.grid-header-cell,
.grid-data-cell {
  padding: 2px 6px;
  text-align: left;
  border-bottom: 1px solid #e5e7eb;
  color: #111827;
  line-height: 1.2;
  height: 24px;
  display: flex;
  align-items: center;
  overflow: hidden; /* 防止内容溢出 */
  white-space: nowrap; /* 防止文本换行 */
  box-sizing: border-box; /* 确保padding和border包含在元素宽度内 */
}

.connections-table-container {
  width: 100%;
  overflow-x: auto;  /* 当内容超出宽度时显示横向滚动条 */
  overflow-y: auto;
  flex: 1 1 auto;   /* 允许增长、收缩，基础大小为自动 */
  margin-top: 0;
  min-height: 0;    /* 允许容器收缩 */
  display: flex;
  flex-direction: column;
}

.table-wrapper {
  width: 100%;
  min-width: fit-content;  /* 使表格宽度适应内容 */
  display: inline-block;   /* 使表格可以超出父容器 */
}

.connections-table {
  width: 100%;  /* 改为100%宽度，以填充容器 */
  border-collapse: collapse;
  font-size: 0.85em;  /* 略微减小字体以适应紧凑设计 */
  min-width: 850px;  /* 调整最小宽度以适应更紧凑的列和新增的启动时间列 */
  border: none;
  border-radius: 0;
  overflow: hidden;
  height: 100%;
  table-layout: auto;  /* 使用auto布局以实现表格宽度随内容变化 */
  flex-shrink: 0;  /* 防止表格被压缩 */
}

.connections-table thead tr {
  background-color: #f9fafb;
  color: #111827;
  text-align: left;
  font-weight: 600;
  border-bottom: 2px solid #e5e7eb;
  height: 24px;
}

.connections-table th,
.connections-table td {
  padding: 2px 6px;
  text-align: left;
  border-bottom: 1px solid #e5e7eb;
  color: #111827;
  line-height: 1.2;
  height: 24px;
  vertical-align: middle;
}

.connections-table th {
  white-space: nowrap; /* 防止表头文字换行 */
}

.connections-table tbody tr:nth-of-type(even) {
  background-color: #f8fafc;
}

.connections-table tbody tr:nth-of-type(odd) {
  background-color: #ffffff;
}

.connections-table tbody tr:hover {
  background-color: #f1f5f9;
}

.connections-table tbody tr.selected-row {
  background-color: #3b82f6 !important; /* 蓝色背景 */
}

.connections-table tbody tr.selected-row td,
.connections-table tbody tr.selected-row th {
  color: white !important; /* 白色文字以提高对比度 */
}

/* 为除了最后一列之外的所有列添加右边框作为分割线 */
.connections-table th:not(:last-child),
.connections-table td:not(:last-child) {
  border-right: 1px solid #d1d5db;
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

/* 为可调整大小的列添加调整手柄 */
.resizable-th {
  position: relative;
}

/* 创建一个透明的拖动区域，覆盖列的右边缘 */
.resizable-th::after {
  content: '';
  position: absolute;
  top: 0;
  right: -4px;  /* 调整位置，使拖动区域与边框重合 */
  width: 8px;  /* 拖动区域的总宽度 */
  height: 100%;
  background-color: transparent;
  cursor: col-resize;
  z-index: 20;  /* 提高z-index确保在最顶层 */
  pointer-events: auto;  /* 确保接收鼠标事件 */
}

/* 当正在调整大小时，显示更明显的视觉反馈 */
.connections-table.resizing::after {
  background-color: #3b82f6;
  opacity: 0.7;
}

/* 为调整手柄添加激活状态 */
.resizing {
  background-color: #3b82f6;
  opacity: 0.7 !important;
}

.sortable-header {
  cursor: pointer;
  user-select: none;
  position: relative;
  padding-right: 20px; /* 为排序指示器留出空间 */
}

.sort-indicator {
  position: absolute;
  right: 5px;
  font-size: 0.8em;
  color: #6b7280;
}

/* 进程名称单元格样式 */
.process-name-cell {
  padding: 0 10px !important;
}

.process-with-icon {
  display: flex;
  align-items: center;
  gap: 8px; /* 图标与文本之间的间距 */
}

.process-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0; /* 防止图标被压缩 */
}

/* 为除了最后一列之外的所有列添加右边框作为分割线 */
.connections-table th:not(:last-child),
.connections-table td:not(:last-child) {
  border-right: 1px solid #d1d5db;
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

/* 为可调整大小的列添加调整手柄 */
.resizable-th {
  position: relative;
}

/* 创建一个透明的拖动区域，覆盖列的右边缘 */
.resizable-th::after {
  content: '';
  position: absolute;
  top: 0;
  right: -4px;  /* 调整位置，使拖动区域与边框重合 */
  width: 8px;  /* 拖动区域的总宽度 */
  height: 100%;
  background-color: transparent;
  cursor: col-resize;
  z-index: 20;  /* 提高z-index确保在最顶层 */
  pointer-events: auto;  /* 确保接收鼠标事件 */
}

/* 当正在调整大小时，显示更明显的视觉反馈 */
.connections-table.resizing::after {
  background-color: #3b82f6;
  opacity: 0.7;
}

/* 为调整手柄添加激活状态 */
.resizing {
  background-color: #3b82f6;
  opacity: 0.7 !important;
}

.sortable-header {
  cursor: pointer;
  user-select: none;
  position: relative;
  padding-right: 20px; /* 为排序指示器留出空间 */
}

.sort-indicator {
  position: absolute;
  right: 5px;
  font-size: 0.8em;
  color: #6b7280;
}

/* 进程名称单元格样式 */
.process-name-cell {
  padding: 0 10px !important;
}

.process-with-icon {
  display: flex;
  align-items: center;
  gap: 8px; /* 图标与文本之间的间距 */
}

.process-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0; /* 防止图标被压缩 */
}

/* 内核进程样式 */
.kernel-process {
  font-weight: bold;
  color: #7c2d12; /* 深红棕色，区别于普通进程 */
  background-color: #fef2f2; /* 浅红背景 */
  padding: 2px 4px;
  border-radius: 3px;
}

/* 右键菜单样式 */
.context-menu {
  position: fixed;
  background: #ffffff;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  overflow: hidden;
  z-index: 10000;
  min-width: 150px;
}

.context-menu ul {
  list-style: none;
  margin: 0;
  padding: 4px 0;
}

.context-menu li {
  padding: 6px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
  font-size: 13px;
  color: #1f2937;
  display: flex;
  align-items: center;
  line-height: 1.2;
}

.context-menu li:hover {
  background-color: #e5e7eb;
}

.context-menu li:active {
  background-color: #d1d5db;
}

.context-menu li:not(:last-child) {
  border-bottom: 1px solid #e5e7eb;
}

/* 弹窗样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10001;
}

.modal-content {
  background: white;
  border-radius: 8px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  width: 500px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
}

.close-button {
  background: #ef4444; /* 红色背景 */
  border: none;
  font-size: 20px; /* 稍微减小字体 */
  cursor: pointer;
  color: white; /* 白色文字 */
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%; /* 圆形 */
  line-height: 1; /* 调整垂直对齐 */
}

.close-button:hover {
  background-color: #dc2626; /* 更深的红色背景 */
}

.process-details {
  padding: 16px 20px;
  overflow-y: auto;
  flex-grow: 1;
  padding-bottom: 20px; /* 确保内容与底部有足够的间距 */
}

.process-details p {
  margin: 8px 0;
  font-size: 14px;
  color: #374151;
  line-height: 1.4;
  display: flex;
  flex-direction: column; /* 将文本排列改为垂直方向 */
}

.process-details strong {
  color: #1f2937;
  min-width: 100px;
  display: inline-block;
  word-break: break-all; /* 允许在任意字符间换行 */
  overflow-wrap: break-word; /* 在长单词或URL地址内部进行换行 */
}

.process-details span {
  word-break: break-all; /* 允许在任意字符间换行 */
  overflow-wrap: break-word; /* 在长单词或URL地址内部进行换行 */
  white-space: pre-wrap; /* 保留空白符序列，但是正常换行 */
}


</style>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body, html {
  margin: 0;
  padding: 0;
  height: 100%;
  width: 100%;
  overflow: hidden;  /* 防止出现全局滚动条 */
}

.container {
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;  /* 使用确切的高度 */
  width: 100vw;
  flex: 1;
  min-height: 0;  /* 允许flex子项收缩 */
  min-width: 0;   /* 允许flex子项收缩 */
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
  margin-bottom: 20px;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #111827;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }

  .header-row-div {
    background-color: #1f2937;
    color: #f9fafb;
    border-bottom: 2px solid #374151;
  }

  .header-cell-div,
  .data-cell-div {
    border-bottom: 1px solid #374151;
    color: #f9fafb;
  }

  .data-row-div.even {
    background-color: #374151;
    color: #f9fafb;
  }

  .data-row-div.odd {
    background-color: #1f2937;
    color: #f9fafb;
  }

  .data-row-div:hover {
    background-color: #4b5563;
  }
}

/* 菜单栏样式 */
.menu-bar {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  padding: 6px 10px;
  background-color: #e2e8f0;
  border-bottom: 1px solid #cbd5e1;
  gap: 15px;
  flex-shrink: 0; /* 防止菜单栏被压缩 */
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12), 0 1px 2px rgba(0, 0, 0, 0.24);
  min-height: 32px;
}

.menu-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.menu-label {
  font-size: 0.8rem;
  color: #334155;
  font-weight: 500;
  white-space: nowrap;
}

.protocol-buttons {
  display: flex;
  gap: 2px;
}

.protocol-btn {
  padding: 3px 8px;
  border: 1px solid #cbd5e1;
  background-color: #e2e8f0;
  color: #475569;
  font-size: 0.75rem;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 40px;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 500;
}

.protocol-btn:hover {
  background-color: #f1f5f9;
  border-color: #94a3b8;
  color: #334155;
}

.protocol-btn.active {
  background-color: #f8fafc;
  color: #1e293b;
  border: 1px solid #94a3b8;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
  font-weight: 600;
}

.menu-search {
  padding: 3px 6px;
  border: 1px solid #94a3b8;
  border-radius: 3px;
  font-size: 0.75rem;
  background-color: #ffffff;
  color: #1e293b;
  min-width: 120px;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
}

.menu-search:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

/* 状态栏样式 */
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 10px;
  background-color: #f3f4f6;
  border-top: 1px solid #d1d5db;
  font-size: 0.7rem;
  color: #374151;
  min-height: 14px;
  flex-shrink: 0; /* 防止状态栏被压缩 */
}

.status-item {
  display: flex;
  align-items: center;
  margin-right: 20px;
}

.status-label {
  margin-right: 6px;
  font-weight: 500;
  color: #4b5563;
}

.status-value {
  font-weight: 600;
  color: #1f2937;
}

.status-value.tcp-count {
  color: #1d4ed8; /* 蓝色 */
}

.status-value.udp-count {
  color: #c2410c; /* 橙色 */
}

.status-value.kernel-count {
  color: #7c2d12; /* 深红棕色 */
}

/* 深色模式下的状态栏样式 */
@media (prefers-color-scheme: dark) {
  .status-bar {
    background-color: #1f2937;
    border-top: 1px solid #374151;
    color: #d1d5db;
  }

  .status-label {
    color: #9ca3af;
  }

  .status-value {
    color: #f9fafb;
  }

  .status-value.tcp-count {
    color: #60a5fa; /* 浅蓝色 */
  }

  .status-value.udp-count {
    color: #fb923c; /* 浅橙色 */
  }

  .status-value.kernel-count {
    color: #fda4af; /* 浅红粉色 */
  }
}
</style>