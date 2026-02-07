<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";

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
  isNew?: boolean; // 标记是否为新连接
  hasChanged?: boolean; // 标记连接是否有变化
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

// 初始化国际化
const { t, locale } = useI18n();

const connections = ref<TcpConnection[]>([]);
const isLoading = ref(false);
const refreshInterval = ref<number | null>(null);
// 自动刷新相关状态
const autoRefreshInterval = ref<number | null>(null);
const isAutoRefreshEnabled = ref(false);
const refreshIntervals = [1, 2, 3, 5, 10]; // 可选的刷新间隔（秒）

const selectedRefreshInterval = ref(1); // 默认选择1秒

// 主题相关状态
const isDarkMode = ref(false);

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
const sortColumn = ref<
  | "process_name"
  | "pid"
  | "protocol"
  | "local_addr"
  | "local_port"
  | "remote_addr"
  | "remote_port"
  | "state"
  | "start_time"
  | null
>(null);
const sortDirection = ref<"asc" | "desc">("asc"); // 'asc' 升序, 'desc' 降序

// 存储用户自定义的列宽
const customColumnWidths = ref<Record<string, number>>({});

// 菜单栏筛选条件
const filterProtocol = ref<"all" | "TCP" | "UDP">("all");
const filterState = ref("all");
const searchProcessName = ref("");
const searchLocalAddr = ref("");

// 状态栏信息
const statusBarInfo = ref({
  totalConnections: 0,
  tcpConnections: 0,
  udpConnections: 0,
  kernelConnections: 0,
  establishedConnections: 0,
  listenConnections: 0,
  timeWaitConnections: 0,
  closeWaitConnections: 0,
  otherConnections: 0,
  lastUpdate: new Date().toLocaleTimeString(),
  refreshInterval: null as number | null,
});

// 获取网络连接列表
async function loadConnections() {
  isLoading.value = true;
  try {
    const result: TcpConnection[] = await invoke("get_connections");

    // 保存当前连接列表的副本用于比较
    const previousConnections = [...connections.value];

    // 应用筛选条件
    let filteredResult = result;

    // 协议筛选
    if (filterProtocol.value !== "all") {
      filteredResult = filteredResult.filter(
        (conn) => conn.protocol === filterProtocol.value,
      );
    }

    // 状态筛选
    if (filterState.value !== "all") {
      filteredResult = filteredResult.filter(
        (conn) => conn.state === filterState.value,
      );
    }

    // 进程名搜索
    if (searchProcessName.value.trim() !== "") {
      const searchTerm = searchProcessName.value.toLowerCase().trim();
      filteredResult = filteredResult.filter(
        (conn) =>
          conn.process_name &&
          conn.process_name.toLowerCase().includes(searchTerm),
      );
    }

    // 本地地址搜索
    if (searchLocalAddr.value.trim() !== "") {
      const searchTerm = searchLocalAddr.value.toLowerCase().trim();
      filteredResult = filteredResult.filter((conn) =>
        conn.local_addr.toLowerCase().includes(searchTerm),
      );
    }

    // 标记状态变化的连接
    filteredResult.forEach((conn) => {
      // 生成唯一标识符用于比较
      const connId = `${conn.protocol}-${conn.local_addr}-${conn.local_port}-${conn.remote_addr}-${conn.remote_port}-${conn.pid || "null"}`;

      // 查找匹配的旧连接
      const matchingPrevConn = previousConnections.find((prevConn) => {
        const prevConnId = `${prevConn.protocol}-${prevConn.local_addr}-${prevConn.local_port}-${prevConn.remote_addr}-${prevConn.remote_port}-${prevConn.pid || "null"}`;
        return prevConnId === connId;
      });

      // 如果找到了匹配的连接，检查状态是否发生了变化
      if (matchingPrevConn) {
        conn.hasChanged = matchingPrevConn.state !== conn.state;
      } else {
        // 如果没有找到匹配的连接，不标记任何状态
        conn.hasChanged = false;
      }
    });

    // 检查是否有连接状态发生了变化
    const changedConnections = filteredResult.filter((conn) => conn.hasChanged);
    if (changedConnections.length > 0) {
      console.log(
        `${t("alerts.connectionStateChanged", { count: changedConnections.length })}:`,
        changedConnections.map((c) => ({
          protocol: c.protocol,
          local_addr: c.local_addr,
          local_port: c.local_port,
          remote_addr: c.remote_addr,
          remote_port: c.remote_port,
          old_state: previousConnections.find(
            (pc) =>
              `${pc.protocol}-${pc.local_addr}-${pc.local_port}-${pc.remote_addr}-${pc.remote_port}-${pc.pid || "null"}` ===
              `${c.protocol}-${c.local_addr}-${c.local_port}-${c.remote_addr}-${c.remote_port}-${c.pid || "null"}`,
          )?.state,
          new_state: c.state,
        })),
      );
    }

    connections.value = filteredResult;

    // 更新状态栏信息
    updateStatusBarInfo(result); // 注意：这里仍然使用原始结果更新状态栏

    // 应用排序
    applySorting();

    // 3秒后移除变化标记
    setTimeout(() => {
      connections.value = connections.value.map((conn) => ({
        ...conn,
        hasChanged: false,
      }));
    }, 3000);

    // 应用自定义列宽（在排序和标记处理之后）
    applyCustomColumnWidths();
  } catch (error) {
    console.error(t("alerts.getConnectionsFailed", { error }), error);
    alert(t("alerts.getConnectionsFailed", { error }));
  } finally {
    isLoading.value = false;
  }
}

// 设置协议筛选
function setProtocolFilter(protocol: "all" | "TCP" | "UDP") {
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
  statusBarInfo.value.tcpConnections = connections.filter(
    (conn) => conn.protocol === "TCP",
  ).length;
  statusBarInfo.value.udpConnections = connections.filter(
    (conn) => conn.protocol === "UDP",
  ).length;
  statusBarInfo.value.kernelConnections = connections.filter(
    (conn) =>
      conn.process_name === "[KERNEL]" ||
      (conn.process_name && conn.process_name.includes("[KERNEL]")),
  ).length;

  // 统计各种连接状态的数量
  statusBarInfo.value.establishedConnections = connections.filter(
    (conn) => conn.state === "ESTABLISHED",
  ).length;
  statusBarInfo.value.listenConnections = connections.filter(
    (conn) => conn.state === "LISTEN",
  ).length;
  statusBarInfo.value.timeWaitConnections = connections.filter(
    (conn) => conn.state === "TIME_WAIT",
  ).length;
  statusBarInfo.value.closeWaitConnections = connections.filter(
    (conn) => conn.state === "CLOSE_WAIT",
  ).length;
  statusBarInfo.value.otherConnections = connections.filter(
    (conn) =>
      conn.state !== "ESTABLISHED" &&
      conn.state !== "LISTEN" &&
      conn.state !== "TIME_WAIT" &&
      conn.state !== "CLOSE_WAIT",
  ).length;

  statusBarInfo.value.lastUpdate = new Date().toLocaleTimeString();
}

// 获取进程详情
async function loadProcessDetails(pid: number) {
  try {
    const details: ProcessDetails = await invoke("get_process_details", {
      pid,
    });
    processDetails.value = details;
  } catch (error) {
    console.error(t("alerts.getProcessDetailsFailed", { error }), error);
    alert(t("alerts.getProcessDetailsFailed", { error }));
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
      alert(
        t("alerts.processKilled", {
          name: conn.process_name || "Unknown",
          pid: conn.pid,
        }),
      );
    } catch (error) {
      console.error(t("alerts.processKillFailed", { error }), error);
      alert(t("alerts.processKillFailed", { error }));
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
    y: posY,
  };
}

// 计算右键菜单样式
const contextMenuStyle = computed(() => {
  return {
    top: `${contextMenuPosition.value.y}px`,
    left: `${contextMenuPosition.value.x}px`,
    position: "fixed" as const,
    zIndex: 1000,
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
      case "process_name":
        valueA = a.process_name || "";
        valueB = b.process_name || "";
        return sortDirection.value === "asc"
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case "pid":
        valueA = a.pid || 0;
        valueB = b.pid || 0;
        return sortDirection.value === "asc"
          ? valueA - valueB
          : valueB - valueA;
      case "protocol":
        valueA = a.protocol || "";
        valueB = b.protocol || "";
        return sortDirection.value === "asc"
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case "local_addr":
        valueA = a.local_addr || "";
        valueB = b.local_addr || "";
        return sortDirection.value === "asc"
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case "local_port":
        valueA = a.local_port || 0;
        valueB = b.local_port || 0;
        return sortDirection.value === "asc"
          ? valueA - valueB
          : valueB - valueA;
      case "remote_addr":
        valueA = a.remote_addr || "";
        valueB = b.remote_addr || "";
        return sortDirection.value === "asc"
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case "remote_port":
        valueA = a.remote_port || 0;
        valueB = b.remote_port || 0;
        return sortDirection.value === "asc"
          ? valueA - valueB
          : valueB - valueA;
      case "state":
        valueA = a.state || "";
        valueB = b.state || "";
        return sortDirection.value === "asc"
          ? valueA.localeCompare(valueB)
          : valueB.localeCompare(valueA);
      case "start_time":
        valueA = a.start_time || 0;
        valueB = b.start_time || 0;
        return sortDirection.value === "asc"
          ? valueA - valueB
          : valueB - valueA;
      default:
        return 0;
    }
  });
}

// 切换列排序
async function toggleSort(
  column:
    | "process_name"
    | "pid"
    | "protocol"
    | "local_addr"
    | "local_port"
    | "remote_addr"
    | "remote_port"
    | "state"
    | "start_time",
) {
  // 在排序前重新获取最新连接数据
  await loadConnections();

  if (sortColumn.value === column) {
    // 如果当前列已经是排序列，则切换排序方向
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    // 如果切换到新列，则默认升序
    sortColumn.value = column;
    sortDirection.value = "asc";
  }

  // 重新应用排序
  applySorting();
}

// 列宽拖拽相关变量
let isDragging = false;
let dragStartX = 0;
let dragStartWidth = 0;
let currentColumnIndex = -1;
let initialColumnWidths: number[] = [];

// 开始拖拽列宽
function startColumnResize(event: MouseEvent, columnIndex: number) {
  // 检查是否在右边框区域（10像素范围内）
  const thElement = event.target as HTMLElement;
  const rect = thElement.getBoundingClientRect();
  const rightEdgeThreshold = 10; // 10像素的边框区域（包括margin）

  // 计算鼠标相对于元素右边的距离
  const distanceFromRight = rect.right - event.clientX;

  // 只有当鼠标在右边框区域内才允许拖拽
  if (distanceFromRight > rightEdgeThreshold) {
    return; // 不在边框区域，不执行拖拽
  }

  isDragging = true;
  dragStartX = event.clientX;
  currentColumnIndex = columnIndex;

  // 获取当前列的宽度
  dragStartWidth = thElement.offsetWidth;

  // 获取所有列的初始宽度
  const thElements = document.querySelectorAll(".connections-table th");
  initialColumnWidths = [];
  thElements.forEach((th) => {
    initialColumnWidths.push((th as HTMLElement).offsetWidth);
  });

  // 获取表格的初始宽度
  const table = document.querySelector(".connections-table") as HTMLElement;
  if (table) {
    // initialTableWidth = table.offsetWidth; // 暂时未使用
  }

  // 添加resizing类到表格
  if (table) {
    table.classList.add("resizing");
  }

  // 添加current-resizing类到当前列
  thElement.classList.add("current-resizing");

  // 添加鼠标移动和释放事件监听器
  document.addEventListener("mousemove", handleColumnResize);
  document.addEventListener("mouseup", stopColumnResize);

  // 阻止默认行为，防止选中文本
  event.preventDefault();
}

// 处理列宽调整
function handleColumnResize(event: MouseEvent) {
  if (!isDragging) return;

  const deltaX = event.clientX - dragStartX;
  const newWidth = Math.max(dragStartWidth + deltaX, 50); // 最小宽度50px

  // 获取所有表头元素
  const thElements = document.querySelectorAll(".connections-table th");
  if (thElements[currentColumnIndex]) {
    const th = thElements[currentColumnIndex] as HTMLElement;

    // 设置新的宽度
    th.style.width = `${newWidth}px`;
    th.style.maxWidth = `${newWidth}px`;
    th.style.minWidth = `${newWidth}px`;

    th.style.setProperty("width", `${newWidth}px`, "important");
    th.style.setProperty("max-width", `${newWidth}px`, "important");
    th.style.setProperty("min-width", `${newWidth}px`, "important");

    // 同时设置对应列的td元素以确保列宽一致
    const tdElements = document.querySelectorAll(
      `.connections-table td:nth-child(${currentColumnIndex + 1})`,
    );
    tdElements.forEach((td) => {
      const tdElement = td as HTMLElement;
      tdElement.style.width = `${newWidth}px`;
      tdElement.style.maxWidth = `${newWidth}px`;
      tdElement.style.minWidth = `${newWidth}px`;

      tdElement.style.setProperty("width", `${newWidth}px`, "important");
      tdElement.style.setProperty("max-width", `${newWidth}px`, "important");
      tdElement.style.setProperty("min-width", `${newWidth}px`, "important");
    });

    // 更新自定义列宽存储
    const columnOrder: (keyof TcpConnection)[] = [
      "process_name",
      "pid",
      "protocol",
      "local_addr",
      "local_port",
      "remote_addr",
      "remote_port",
      "state",
      "start_time",
    ];
    if (columnOrder[currentColumnIndex]) {
      customColumnWidths.value[columnOrder[currentColumnIndex]] = newWidth;
    }
  }
}

// 结束列宽调整
function stopColumnResize() {
  isDragging = false;

  // 移除resizing类
  const table = document.querySelector(".connections-table") as HTMLElement;
  if (table) {
    table.classList.remove("resizing");
  }

  // 移除current-resizing类
  const thElements = document.querySelectorAll(".connections-table th");
  thElements.forEach((th) => {
    th.classList.remove("current-resizing");
  });

  // 恢复鼠标指针样式
  thElements.forEach((th) => {
    (th as HTMLElement).style.cursor = "";
  });

  // 移除事件监听器
  document.removeEventListener("mousemove", handleColumnResize);
  document.removeEventListener("mouseup", stopColumnResize);
}

// 应用自定义列宽到DOM
function applyCustomColumnWidths() {
  // 使用 setTimeout 确保在 DOM 更新后执行
  setTimeout(() => {
    const thElements = document.querySelectorAll(".connections-table th");
    const columnOrder: (keyof TcpConnection)[] = [
      "process_name",
      "pid",
      "protocol",
      "local_addr",
      "local_port",
      "remote_addr",
      "remote_port",
      "state",
      "start_time",
    ];

    columnOrder.forEach((col, index) => {
      if (thElements[index]) {
        const th = thElements[index] as HTMLElement;
        let colWidth;

        if (customColumnWidths.value[col]) {
          // 使用自定义宽度
          colWidth = customColumnWidths.value[col];
        } else {
          // 使用默认宽度
          switch (col) {
            case "process_name":
              colWidth = 150; // 进程名称列较宽
              break;
            case "pid":
              colWidth = 80; // PID列较窄
              break;
            case "protocol":
              colWidth = 60; // 协议列较窄
              break;
            case "local_addr":
              colWidth = 120; // 本地地址列中等
              break;
            case "local_port":
              colWidth = 80; // 本地端口列较窄
              break;
            case "remote_addr":
              colWidth = 120; // 远程地址列中等
              break;
            case "remote_port":
              colWidth = 80; // 远程端口列较窄
              break;
            case "state":
              colWidth = 100; // 状态列中等
              break;
            case "start_time":
              colWidth = 150; // 启动时间列较宽
              break;
            default:
              colWidth = 100; // 默认宽度
          }
        }

        // 设置列宽并使用 !important 确保优先级
        th.style.width = `${colWidth}px`;
        th.style.maxWidth = `${colWidth}px`;
        th.style.minWidth = `${colWidth}px`;

        th.style.setProperty("width", `${colWidth}px`, "important");
        th.style.setProperty("max-width", `${colWidth}px`, "important");
        th.style.setProperty("min-width", `${colWidth}px`, "important");

        // 同时设置td元素以确保列宽一致
        const tdElements = document.querySelectorAll(
          `.connections-table td:nth-child(${index + 1})`,
        );
        tdElements.forEach((td) => {
          const tdElement = td as HTMLElement;
          tdElement.style.width = `${colWidth}px`;
          tdElement.style.maxWidth = `${colWidth}px`;
          tdElement.style.minWidth = `${colWidth}px`;

          tdElement.style.setProperty("width", `${colWidth}px`, "important");
          tdElement.style.setProperty(
            "max-width",
            `${colWidth}px`,
            "important",
          );
          tdElement.style.setProperty(
            "min-width",
            `${colWidth}px`,
            "important",
          );
        });
      }
    });

    // 确保表格使用固定布局
    const table = document.querySelector(".connections-table") as HTMLElement;
    if (table) {
      table.style.tableLayout = "fixed";
    }

    // 设置冗余列填充剩余空间
    const fillerColumns = document.querySelectorAll(
      ".connections-table th.filler-column, .connections-table td.filler-cell",
    );
    fillerColumns.forEach((filler) => {
      const fillerElement = filler as HTMLElement;
      fillerElement.style.width = "100%";
      fillerElement.style.minWidth = "0";
      fillerElement.style.maxWidth = "none";
    });
  }, 0);
}

// 启用自动刷新
function enableAutoRefresh() {
  if (autoRefreshInterval.value !== null) {
    clearInterval(autoRefreshInterval.value);
  }

  isAutoRefreshEnabled.value = true;
  autoRefreshInterval.value = window.setInterval(() => {
    loadConnections();
  }, selectedRefreshInterval.value * 1000); // 转换为毫秒

  // 更新状态栏信息
  statusBarInfo.value.refreshInterval = selectedRefreshInterval.value;
}

// 禁用自动刷新
function disableAutoRefresh() {
  if (autoRefreshInterval.value !== null) {
    clearInterval(autoRefreshInterval.value);
    autoRefreshInterval.value = null;
  }

  isAutoRefreshEnabled.value = false;
  statusBarInfo.value.refreshInterval = null;
}

// 切换自动刷新
function toggleAutoRefresh() {
  if (isAutoRefreshEnabled.value) {
    disableAutoRefresh();
  } else {
    enableAutoRefresh();
  }
}

// 更改刷新间隔
function changeRefreshInterval(interval: number) {
  selectedRefreshInterval.value = interval;

  // 如果自动刷新已启用，重新设置间隔
  if (isAutoRefreshEnabled.value) {
    enableAutoRefresh();
  }

  // 更新状态栏信息
  if (isAutoRefreshEnabled.value) {
    statusBarInfo.value.refreshInterval = selectedRefreshInterval.value;
  }
}

// 页面加载完成后自动获取连接列表
onMounted(() => {
  // 设置语言为本地存储的语言或浏览器语言
  const savedLang = localStorage.getItem("locale");
  if (savedLang && (savedLang === "zh" || savedLang === "en")) {
    const { locale } = useI18n();
    locale.value = savedLang as "zh" | "en";
  }

  // 检查并应用主题偏好
  checkSystemThemePreference();

  loadConnections();

  // 监听窗口大小变化事件
  window.addEventListener("resize", handleWindowResize);

  // 监听文档上的点击事件，用于隐藏右键菜单
  document.addEventListener("click", hideContextMenu);
});

// 组件卸载时清理定时器和事件监听器
onUnmounted(() => {
  if (refreshInterval.value !== null) {
    clearInterval(refreshInterval.value);
  }

  // 清除自动刷新定时器
  if (autoRefreshInterval.value !== null) {
    clearInterval(autoRefreshInterval.value);
  }

  // 移除窗口大小变化事件监听器
  window.removeEventListener("resize", handleWindowResize);

  // 移除文档点击事件监听器
  document.removeEventListener("click", hideContextMenu);

  // 确保移除可能存在的列宽调整事件监听器
  document.removeEventListener("mousemove", handleColumnResize);
  document.removeEventListener("mouseup", stopColumnResize);
});

// 处理窗口大小变化
function handleWindowResize() {
  // 在窗口大小变化时，重新应用列宽
  setTimeout(() => {
    // 重新应用自定义列宽
    applyCustomColumnWidths();

    // 强制浏览器重新计算布局
    document.body.offsetHeight;
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
    return "-";
  }

  // 将秒级时间戳转换为毫秒级时间戳
  const date = new Date(timestamp * 1000);

  // 获取年、月、日、小时、分钟和秒
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0"); // 月份从0开始，需要+1
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");

  // 返回格式化的日期时间字符串
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

// 判断是否为内核进程
function isKernelProcess(processName: string | null): boolean {
  return (
    processName === "[KERNEL]" ||
    (processName !== null && processName.includes("[KERNEL]"))
  );
}

// 语言切换功能
function changeLanguage(lang: "zh" | "en") {
  locale.value = lang;
  localStorage.setItem("locale", lang); // 保存用户选择的语言到localStorage
}

// 主题切换功能
function toggleTheme() {
  isDarkMode.value = !isDarkMode.value;
  if (isDarkMode.value) {
    document.documentElement.classList.add("dark");
    localStorage.setItem("theme", "dark");
  } else {
    document.documentElement.classList.remove("dark");
    localStorage.setItem("theme", "light");
  }
}

// 检查系统主题偏好
function checkSystemThemePreference() {
  const savedTheme = localStorage.getItem("theme");
  const systemPrefersDark = window.matchMedia(
    "(prefers-color-scheme: dark)",
  ).matches;

  if (savedTheme) {
    isDarkMode.value = savedTheme === "dark";
  } else {
    isDarkMode.value = systemPrefersDark;
  }

  // 应用主题到页面
  if (isDarkMode.value) {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}
</script>

<template>
  <div class="container">
    <!-- 菜单栏 -->
    <!-- 主菜单容器，分为左右两组，两端对齐 -->
    <div class="main-menu-container">
      <div class="menu-group">
        <label class="menu-label">{{ t("menu.protocol") }}</label>
        <div class="protocol-buttons">
          <button
            :class="['protocol-btn', { active: filterProtocol === 'all' }]"
            @click="setProtocolFilter('all')"
          >
            {{ t("menu.protocolAll") }}
          </button>
          <button
            :class="['protocol-btn', { active: filterProtocol === 'TCP' }]"
            @click="setProtocolFilter('TCP')"
          >
            {{ t("menu.protocolTCP") }}
          </button>
          <button
            :class="['protocol-btn', { active: filterProtocol === 'UDP' }]"
            @click="setProtocolFilter('UDP')"
          >
            {{ t("menu.protocolUDP") }}
          </button>
        </div>
      </div>

      <div class="menu-group">
        <label class="menu-label">{{ t("menu.state") }}</label>
        <select
          v-model="filterState"
          @change="applyFiltersAndSearch"
          class="state-select"
        >
          <option value="all">{{ t("menu.stateAll") }}</option>
          <option value="LISTEN">{{ t("menu.stateListen") }}</option>
          <option value="ESTABLISHED">
            {{ t("menu.stateEstablished") }}
          </option>
          <option value="TIME_WAIT">{{ t("menu.stateTimeWait") }}</option>
          <option value="CLOSE_WAIT">{{ t("menu.stateCloseWait") }}</option>
          <option value="SYN_SENT">{{ t("menu.stateSynSent") }}</option>
          <option value="SYN_RECV">{{ t("menu.stateSynRecv") }}</option>
          <option value="FIN_WAIT1">{{ t("menu.stateFinWait1") }}</option>
          <option value="FIN_WAIT2">{{ t("menu.stateFinWait2") }}</option>
          <option value="LAST_ACK">{{ t("menu.stateLastAck") }}</option>
          <option value="CLOSING">{{ t("menu.stateClosing") }}</option>
          <option value="UNCONN">{{ t("menu.stateUnconn") }}</option>
        </select>
      </div>

      <div class="menu-group">
        <label class="menu-label">{{ t("menu.searchProcess") }}</label>
        <input
          type="text"
          v-model="searchProcessName"
          @input="applyFiltersAndSearch"
          :placeholder="t('menu.searchPlaceholder')"
          class="menu-search"
        />
      </div>

      <div class="menu-group">
        <label class="menu-label">{{ t("menu.searchLocalAddr") }}</label>
        <input
          type="text"
          v-model="searchLocalAddr"
          @input="applyFiltersAndSearch"
          :placeholder="t('menu.localAddrPlaceholder')"
          class="menu-search"
        />
      </div>

      <div class="menu-group">
        <label class="menu-label">{{ t("menu.autoRefresh") }}</label>
        <div class="refresh-controls">
          <button
            :class="['refresh-toggle-btn', { active: isAutoRefreshEnabled }]"
            @click="toggleAutoRefresh"
          >
            {{
              isAutoRefreshEnabled
                ? t("menu.refreshStop")
                : t("menu.refreshStart")
            }}
          </button>
          <select
            v-model="selectedRefreshInterval"
            @change="changeRefreshInterval(selectedRefreshInterval)"
            class="refresh-interval-select"
            :disabled="!isAutoRefreshEnabled"
          >
            <option
              v-for="interval in refreshIntervals"
              :key="interval"
              :value="interval"
            >
              {{ interval }}{{ t("menu.refreshInterval") }}
            </option>
          </select>
        </div>
      </div>
      <div class="menu-group">
        <label class="menu-label">{{ t("menu.language") }}</label>
        <select
          @change="
            changeLanguage(
              ($event.target as HTMLSelectElement).value as 'zh' | 'en',
            )
          "
          :value="$i18n.locale"
          class="lang-select"
        >
          <option value="zh">{{ t("zh") }}</option>
          <option value="en">{{ t("en") }}</option>
        </select>
      </div>
      <div class="menu-group">
        <label class="menu-label">{{ t("menu.theme") }}</label>
        <button
          class="theme-toggle-btn"
          @click="toggleTheme"
          :title="isDarkMode ? t('menu.lightTheme') : t('menu.darkTheme')"
        >
          {{ isDarkMode ? t("menu.lightTheme") : t("menu.darkTheme") }}
        </button>
      </div>
    </div>

    <div class="connections-table-container">
      <div class="table-wrapper">
        <table class="connections-table">
          <thead>
            <tr>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 0)"
              >
                <div class="column-header" @click="toggleSort('process_name')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.processName") }}
                    <span
                      v-if="sortColumn === 'process_name'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 1)"
              >
                <div class="column-header" @click="toggleSort('pid')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.pid") }}
                    <span v-if="sortColumn === 'pid'" class="sort-indicator">
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 2)"
              >
                <div class="column-header" @click="toggleSort('protocol')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.protocol") }}
                    <span
                      v-if="sortColumn === 'protocol'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 3)"
              >
                <div class="column-header" @click="toggleSort('local_addr')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.localAddr") }}
                    <span
                      v-if="sortColumn === 'local_addr'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 4)"
              >
                <div class="column-header" @click="toggleSort('local_port')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.localPort") }}
                    <span
                      v-if="sortColumn === 'local_port'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 5)"
              >
                <div class="column-header" @click="toggleSort('remote_addr')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.remoteAddr") }}
                    <span
                      v-if="sortColumn === 'remote_addr'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 6)"
              >
                <div class="column-header" @click="toggleSort('remote_port')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.remotePort") }}
                    <span
                      v-if="sortColumn === 'remote_port'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 7)"
              >
                <div class="column-header" @click="toggleSort('state')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.state") }}
                    <span v-if="sortColumn === 'state'" class="sort-indicator">
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th
                class="resizable-th"
                @mousedown="startColumnResize($event, 8)"
              >
                <div class="column-header" @click="toggleSort('start_time')">
                  <span class="sortable-header">
                    {{ t("tableHeaders.startTime") }}
                    <span
                      v-if="sortColumn === 'start_time'"
                      class="sort-indicator"
                    >
                      {{ sortDirection === "asc" ? " ▲" : " ▼" }}
                    </span>
                  </span>
                </div>
              </th>
              <th class="filler-column">
                <!-- 冗余列，用于填充剩余空间 -->
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
              :class="{
                'selected-row': clickedConnection === conn,
                'changed-connection': conn.hasChanged,
              }"
            >
              <td class="process-name-cell">
                <div class="process-with-icon">
                  <img
                    v-if="conn.icon && !isKernelProcess(conn.process_name)"
                    :src="'data:image/png;base64,' + conn.icon"
                    :alt="conn.process_name || 'Process Icon'"
                    class="process-icon"
                  />
                  <span
                    :class="{
                      'kernel-process': isKernelProcess(conn.process_name),
                    }"
                    >{{ conn.process_name || "-" }}</span
                  >
                </div>
              </td>
              <td>{{ conn.pid || "-" }}</td>
              <td>{{ conn.protocol }}</td>
              <td>{{ conn.local_addr }}</td>
              <td>{{ conn.local_port }}</td>
              <td>{{ conn.remote_addr }}</td>
              <td>{{ conn.remote_port }}</td>
              <td>{{ conn.state }}</td>
              <td>{{ formatDate(conn.start_time) }}</td>
              <td class="filler-cell"></td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- 右键菜单 -->
    <div v-if="showContextMenu" class="context-menu" :style="contextMenuStyle">
      <ul>
        <li @click="showProcessDetailsDialog(selectedConnection!)">
          {{ t("contextMenu.processDetails") }}
        </li>
        <li @click="killProcess(selectedConnection!)">
          {{ t("contextMenu.killProcess") }}
        </li>
      </ul>
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.totalConnections") }}:</span>
        <span class="status-value">{{ statusBarInfo.totalConnections }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.tcpConnections") }}:</span>
        <span class="status-value tcp-count">{{
          statusBarInfo.tcpConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.udpConnections") }}:</span>
        <span class="status-value udp-count">{{
          statusBarInfo.udpConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.established") }}:</span>
        <span class="status-value established-count">{{
          statusBarInfo.establishedConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.listen") }}:</span>
        <span class="status-value listen-count">{{
          statusBarInfo.listenConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.wait") }}:</span>
        <span class="status-value wait-count">{{
          statusBarInfo.timeWaitConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.closeWait") }}:</span>
        <span class="status-value close-wait-count">{{
          statusBarInfo.closeWaitConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.other") }}:</span>
        <span class="status-value other-count">{{
          statusBarInfo.otherConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.kernel") }}:</span>
        <span class="status-value kernel-count">{{
          statusBarInfo.kernelConnections
        }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">{{ t("statusBar.lastUpdate") }}:</span>
        <span class="status-value">{{ statusBarInfo.lastUpdate }}</span>
      </div>
      <div class="status-item" v-if="statusBarInfo.refreshInterval">
        <span class="status-label">{{ t("statusBar.refreshInterval") }}:</span>
        <span class="status-value"
          >{{ statusBarInfo.refreshInterval
          }}{{ t("menu.refreshInterval") }}</span
        >
      </div>
    </div>

    <!-- 进程详情弹窗 -->
    <div
      v-if="showProcessDetails"
      class="modal-overlay"
      @click="showProcessDetails = false"
    >
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>{{ t("modal.processDetails") }}</h3>
          <button class="close-button" @click="showProcessDetails = false">
            ×
          </button>
        </div>
        <div v-if="processDetails" class="process-details">
          <p>
            <strong>{{ t("modal.pid") }}:</strong>
            <span>{{ processDetails.pid }}</span>
          </p>
          <p>
            <strong>{{ t("modal.name") }}:</strong>
            <span>{{ processDetails.name }}</span>
          </p>
          <p>
            <strong>{{ t("modal.commandLine") }}:</strong>
            <span>{{ processDetails.command_line }}</span>
          </p>
          <p>
            <strong>{{ t("modal.executablePath") }}:</strong>
            <span>{{ processDetails.executable_path }}</span>
          </p>
          <p>
            <strong>{{ t("modal.memoryUsage") }}:</strong>
            <span>{{ formatMemoryUsage(processDetails.memory_usage) }}</span>
          </p>
          <p>
            <strong>{{ t("modal.cpuUsage") }}:</strong>
            <span>{{ processDetails.cpu_usage }}%</span>
          </p>
          <p>
            <strong>{{ t("modal.parentPid") }}:</strong>
            <span>{{ processDetails.parent_pid }}</span>
          </p>
          <p>
            <strong>{{ t("modal.startTime") }}:</strong>
            <span>{{ formatDate(processDetails.start_time) }}</span>
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.connections-table-container {
  width: 100%;
  overflow-x: auto; /* 当内容超出宽度时显示横向滚动条 */
  overflow-y: auto; /* 当内容超出高度时显示纵向滚动条 */
  flex: 1 1 auto; /* 允许增长、收缩，基础大小为自动 */
  margin-top: 0;
  min-height: 0; /* 允许容器收缩 */
  display: flex;
  flex-direction: column;
}

.table-wrapper {
  width: 100%;
  display: block;
  flex: 1; /* 让表格填充可用空间 */
  min-height: 0; /* 允许内容收缩 */
  overflow-y: auto; /* 垂直滚动 */
  min-width: max-content; /* 确保表格至少适应内容宽度 */
  overflow-x: auto; /* 水平滚动，允许表格不完全填充容器 */
}

.connections-table {
  width: fit-content; /* 根据内容调整宽度 */
  min-width: fit-content; /* 不强制占满容器宽度 */
  border-collapse: collapse;
  font-size: 0.85em; /* 略微减小字体以适应紧凑设计 */
  border: none;
  border-radius: 0;
  table-layout: fixed; /* 使用fixed布局以精确控制列宽 */
  flex-shrink: 0; /* 防止表格被压缩 */
  margin-bottom: 0; /* 确保表格紧贴容器底部 */
  display: table; /* 使用表格显示 */
  max-width: 100%; /* 限制表格最大宽度不超过容器 */
}

.connections-table thead tr {
  background-color: #f9fafb;
  color: #111827;
  text-align: left;
  font-weight: 600;
  border-bottom: 2px solid #e5e7eb;
  height: 24px;
  position: sticky; /* 固定表头 */
  top: 0; /* 固定在顶部 */
  z-index: 10; /* 确保表头在内容之上 */
  display: table-row; /* 确保sticky在表格行上正确工作 */
}

.connections-table tbody {
  display: table-row-group; /* 确保tbody正确显示 */
}

.connections-table th,
.connections-table td {
  padding: 2px 3px; /* 左右padding 3px */
  text-align: left;
  border-bottom: 1px solid #e5e7eb;
  color: #111827;
  line-height: 1.2;
  height: 24px;
  vertical-align: middle;
  white-space: nowrap; /* 防止文本换行 */
  overflow: hidden; /* 防止内容溢出 */
  word-break: keep-all; /* 防止单词内断行 */
}

.connections-table th {
  min-width: max-content; /* 表头列宽自适应内容 */
}

.connections-table td {
  min-width: max-content; /* 数据列宽自适应内容 */
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

/* 状态变化的连接项样式 */
.connections-table tbody tr.changed-connection {
  background-color: #fbbf24 !important; /* 琥珀色背景表示状态变化 */
  transition: background-color 3s ease; /* 3秒过渡效果 */
}

.connections-table tbody tr.changed-connection td,
.connections-table tbody tr.changed-connection th {
  color: #78350f !important; /* 深琥珀色文字 */
}

/* 为除了最后一列之外的所有列添加右边框作为分割线 */
.connections-table th:not(:last-child),
.connections-table td:not(:last-child) {
  border-right: 1px solid #d1d5db;
}

/* 冗余列样式 - 填充剩余空间 */
.filler-column {
  width: 100%; /* 填充剩余空间 */
  min-width: 0; /* 允许缩小到内容宽度 */
  max-width: none; /* 不限制最大宽度 */
  border: none; /* 不显示边框 */
}

.filler-cell {
  width: 100%; /* 填充剩余空间 */
  min-width: 0; /* 允许缩小到内容宽度 */
  max-width: none; /* 不限制最大宽度 */
  border: none; /* 不显示边框 */
}

.grid-data-cell:last-child {
  border-right: none; /* 最后一列不需要右边框 */
}

.grid-data-row:nth-of-type(even) {
  background-color: #f8fafc;
}

.grid-data-row:nth-of-type(odd) {
  background-color: #ffffff;
}

.grid-data-row:hover {
  background-color: #f1f5f9;
}

.grid-data-row.selected-row {
  background-color: #3b82f6 !important; /* 蓝色背景 */
}

.grid-data-row.selected-row .grid-data-cell {
  color: white !important; /* 白色文字以提高对比度 */
}

/* 状态变化的连接项样式 */
.grid-data-row.changed-connection {
  background-color: #fbbf24 !important; /* 琥珀色背景表示状态变化 */
  transition: background-color 3s ease; /* 3秒过渡效果 */
}

.grid-data-row.changed-connection .grid-data-cell {
  color: #78350f !important; /* 深琥珀色文字 */
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
  cursor: default; /* 默认情况下光标为默认样式 */
}

/* 添加拖动区域 */
.resizable-th::after {
  content: "";
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 10px; /* 拖动区域宽度，增加可点击区域 */
  cursor: col-resize;
  background: transparent;
  z-index: 10;
  margin-right: -5px; /* 扩大可点击区域 */
}

.resizable-th::after:hover {
  background: #94a3b8; /* 悬停时显示灰色线条 */
  opacity: 0.7;
}

/* 当正在调整大小时，显示更明显的视觉反馈 */
.connections-table.resizing {
  user-select: none; /* 防止在拖拽过程中选中文本 */
}

/* 为调整手柄添加激活状态 */
.connections-table.resizing .resizable-th.current-resizing::after {
  background: #3b82f6; /* 调整大小时显示蓝色线条 */
  opacity: 0.8;
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

/* 确保表格容器有相对定位以便sticky定位正常工作 */
.table-wrapper {
  position: relative;
}

/* 进程名称单元格样式 */
.process-name-cell {
  padding: 0 10px !important;
}

.process-with-icon {
  display: flex;
  align-items: center;
  gap: 3px; /* 图标与文本之间的间距 */
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

/* 冗余列样式 - 填充剩余空间 */
.filler-column {
  width: 100%; /* 填充剩余空间 */
  min-width: 0; /* 允许缩小到内容宽度 */
  max-width: none; /* 不限制最大宽度 */
  border: none; /* 不显示边框 */
}

.filler-cell {
  width: 100%; /* 填充剩余空间 */
  min-width: 0; /* 允许缩小到内容宽度 */
  max-width: none; /* 不限制最大宽度 */
  border: none; /* 不显示边框 */
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
  cursor: default; /* 默认情况下光标为默认样式 */
}

/* 添加拖动区域 */
.resizable-th::after {
  content: "";
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 10px; /* 拖动区域宽度，增加可点击区域 */
  cursor: col-resize;
  background: transparent;
  z-index: 10;
  margin-right: -5px; /* 扩大可点击区域 */
}

.resizable-th::after:hover {
  background: #94a3b8; /* 悬停时显示灰色线条 */
  opacity: 0.7;
}

/* 当正在调整大小时，显示更明显的视觉反馈 */
.connections-table.resizing {
  user-select: none; /* 防止在拖拽过程中选中文本 */
}

/* 为调整手柄添加激活状态 */
.connections-table.resizing .resizable-th.current-resizing::after {
  background: #3b82f6; /* 调整大小时显示蓝色线条 */
  opacity: 0.8;
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

/* 确保表格容器有相对定位以便sticky定位正常工作 */
.table-wrapper {
  position: relative;
}

/* 进程名称单元格样式 */
.process-name-cell {
  padding: 0 10px !important;
}

.process-with-icon {
  display: flex;
  align-items: center;
  gap: 3px; /* 图标与文本之间的间距 */
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
  box-shadow:
    0 10px 15px -3px rgba(0, 0, 0, 0.1),
    0 4px 6px -2px rgba(0, 0, 0, 0.05);
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
  box-shadow:
    0 20px 25px -5px rgba(0, 0, 0, 0.1),
    0 10px 10px -5px rgba(0, 0, 0, 0.04);
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

body,
html {
  margin: 0;
  padding: 0;
  height: 100%;
  width: 100%;
  overflow: hidden; /* 防止出现全局滚动条 */
}

.container {
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  height: 100vh; /* 使用确切的高度 */
  width: 100%; /* 使用百分比宽度，避免滚动条问题 */
  flex: 1 1 auto;
  min-height: 0; /* 允许flex子项收缩 */
  min-width: 0; /* 允许flex子项收缩 */
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

/* 主菜单容器样式 - 用于将左右菜单组两端对齐 */
.main-menu-container {
  display: flex;
  justify-content: flex-start; /* 改为flex-start，使用margin-left: auto来分离元素 */
  align-items: center;
  width: 100%;
  padding: 6px 10px; /* 恢复左右padding以提供内边距 */
  background-color: #e2e8f0;
  border-bottom: 1px solid #cbd5e1;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.12),
    0 1px 2px rgba(0, 0, 0, 0.24);
  min-height: 32px;
  position: sticky; /* 使菜单栏固定在顶部 */
  top: 0; /* 固定在顶部 */
  z-index: 100; /* 确保菜单栏在其他内容之上 */
  flex-wrap: nowrap; /* 防止换行 */
  min-width: fit-content; /* 确保菜单栏宽度适应内容 */
}

.menu-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0; /* 防止菜单组被压缩 */
  flex-wrap: nowrap; /* 防止组内元素换行 */
  white-space: nowrap; /* 防止文字换行 */
  min-width: fit-content; /* 确保内容适应其内容 */
  padding-right: 10px;
}

/* 特别针对包含主题和语言选择的菜单组 */
.menu-group:last-child {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  flex-wrap: nowrap;
  white-space: nowrap;
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
  min-width: 100px; /* 减少最小宽度以节省空间 */
  max-width: 150px; /* 限制最大宽度 */
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
  flex-shrink: 0; /* 防止搜索框被压缩 */
}

.menu-search:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

.refresh-controls {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-shrink: 0; /* 防止刷新控件被压缩 */
  flex: none; /* 禁止伸缩 */
}

.refresh-toggle-btn {
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

.refresh-toggle-btn:hover {
  background-color: #f1f5f9;
  border-color: #94a3b8;
  color: #334155;
}

.refresh-toggle-btn.active {
  background-color: #10b981; /* 绿色表示激活状态 */
  color: white;
  border: 1px solid #059669;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
  font-weight: 600;
}

.refresh-interval-select {
  padding: 2px 4px;
  border: 1px solid #94a3b8;
  border-radius: 3px;
  font-size: 0.75rem;
  background-color: #ffffff;
  color: #1e293b;
  min-width: 60px;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
}

.refresh-interval-select:disabled {
  background-color: #e2e8f0;
  color: #94a3b8;
  cursor: not-allowed;
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
  flex-wrap: nowrap; /* 防止换行 */
  overflow: hidden; /* 隐藏溢出内容，不显示滚动条 */
}

.status-item {
  display: flex;
  align-items: center;
  margin-right: 20px;
  white-space: nowrap; /* 防止内容换行 */
  position: relative; /* 为添加分隔符做准备 */
}

/* 为每个状态项添加右侧分隔符（最后一个除外） */
.status-item:not(:last-child)::after {
  content: "|";
  margin-left: 25px; /* 在分隔符左侧添加一些间距 */
  color: #9ca3af; /* 分隔符颜色 */
  opacity: 0.7; /* 稍微降低分隔符的透明度 */
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

.status-value.established-count {
  color: #16a34a; /* 绿色 - 表示已建立的连接 */
}

.status-value.listen-count {
  color: #3b82f6; /* 蓝色 - 表示监听状态 */
}

.status-value.wait-count {
  color: #eab308; /* 黄色 - 表示等待状态 */
}

.status-value.close-wait-count {
  color: #f97316; /* 橙色 - 表示关闭等待 */
}

.status-value.other-count {
  color: #8b5cf6; /* 紫色 - 表示其他状态 */
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

  .status-value.established-count {
    color: #4ade80; /* 浅绿色 */
  }

  .status-value.listen-count {
    color: #93c5fd; /* 浅蓝色 */
  }

  .status-value.wait-count {
    color: #facc15; /* 浅黄色 */
  }

  .status-value.close-wait-count {
    color: #fb9467; /* 浅橙色 */
  }

  .status-value.other-count {
    color: #c4b5fd; /* 浅紫色 */
  }

  .status-value.kernel-count {
    color: #fda4af; /* 浅红粉色 */
  }
}

/* 语言切换下拉框样式 */
.lang-select {
  padding: 3px 8px;
  border: 1px solid #cbd5e1;
  color: #475569;
  font-size: 0.75rem;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 80px;
  font-weight: 500;
  background-color: #e2e8f0;
}

.lang-select:hover {
  background-color: #f1f5f9;
  border-color: #94a3b8;
  color: #334155;
}

.lang-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}
.state-select {
  padding: 3px 8px;
  border: 1px solid #cbd5e1;
  color: #475569;
  font-size: 0.75rem;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 80px;
  font-weight: 500;
  background-color: #e2e8f0;
}

.state-select:hover {
  background-color: #f1f5f9;
  border-color: #94a3b8;
  color: #334155;
}

.state-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

/* 主题切换按钮样式 */
.theme-toggle-btn {
  padding: 3px 8px;
  border: 1px solid #cbd5e1;
  background-color: #e2e8f0;
  color: #475569;
  font-size: 0.75rem;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: fit-content;
  text-align: center;
  font-weight: 500;
}

.theme-toggle-btn:hover {
  background-color: #f1f5f9;
  border-color: #94a3b8;
  color: #334155;
}

/* 暗色主题样式 */
.dark-theme {
  background-color: #1f2937;
  color: #f9fafb;
}

.dark-theme .main-menu-container {
  background-color: #1f2937;
  color: #f9fafb;
  border-bottom: 2px solid #374151;
}

.dark-theme .protocol-btn {
  background-color: #374151;
  color: #d1d5db;
  border: 1px solid #4b5563;
}

.dark-theme .protocol-btn:hover {
  background-color: #4b5563;
  border-color: #6b7280;
  color: #e5e7eb;
}

.dark-theme .protocol-btn.active {
  background-color: #4f46e5;
  color: #f9fafb;
  border: 1px solid #6366f1;
}

.dark-theme .menu-search {
  background-color: #111827;
  color: #f9fafb;
  border: 1px solid #4b5563;
}

.dark-theme .refresh-toggle-btn {
  background-color: #374151;
  color: #d1d5db;
  border: 1px solid #4b5563;
}

.dark-theme .refresh-toggle-btn:hover {
  background-color: #4b5563;
  border-color: #6b7280;
  color: #e5e7eb;
}

.dark-theme .refresh-toggle-btn.active {
  background-color: #10b981; /* 绿色表示激活状态 */
  color: white;
  border: 1px solid #059669;
}
</style>
