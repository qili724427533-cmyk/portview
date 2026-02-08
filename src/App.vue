<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import ConnectionsTable from "./components/ConnectionsTable.vue";
import MenuBar from "./components/MenuBar.vue";
import StatusBar from "./components/StatusBar.vue";
import ContextMenu from "./components/ContextMenu.vue";
import ProcessDetailsModal from "./components/ProcessDetailsModal.vue";
import AboutDialog from "./components/AboutDialog.vue";

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

// 关于对话框相关状态
const showAbout = ref(false);

// 应用版本号
const appVersion = ref('Loading...');

// 通知提示相关状态
const showNotificationBox = ref(false);
const notificationMessage = ref('');
const notificationType = ref<'success' | 'error' | 'info'>('info'); // 'success', 'error', or 'info'

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
const searchLocalPort = ref("");

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

    // 本地端口搜索
    if (searchLocalPort.value.trim() !== "") {
      const searchTerm = searchLocalPort.value.trim();
      filteredResult = filteredResult.filter((conn) => {
        // 检查输入是否为纯数字（端口号）
        const isNumeric = /^\d+$/.test(searchTerm);
        if (isNumeric) {
          // 如果是数字，按端口号精确匹配
          return conn.local_port.toString() === searchTerm;
        } else {
          // 如果不是数字，按正则表达式匹配
          const regex = new RegExp(searchTerm, 'i');
          return regex.test(conn.local_port.toString());
        }
      });
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
    selectedConnection.value = conn; // 保存选中的连接以获取图标信息
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
      showNotification(
        t("alerts.processKilled", {
          name: conn.process_name || "Unknown",
          pid: conn.pid,
        }),
        'success'
      );
    } catch (error) {
      console.error(t("alerts.processKillFailed", { error }), error);
      showNotification(
        t("alerts.processKillFailed", { error }),
        'error'
      );
    }
  }
}

// 打开进程所在目录
async function openContainingFolder(conn: TcpConnection) {
  if (conn.pid) {
    try {
      // 获取进程详细信息以获得可执行路径
      const details: ProcessDetails = await invoke("get_process_details", {
        pid: conn.pid,
      });
      
      // 检查可执行路径是否存在且不为空
      if (details.executable_path && details.executable_path.trim() !== '') {
        // 调用打开目录功能
        await invoke('open_folder', { path: details.executable_path });
      } else {
        // 如果可执行路径为空，显示提示信息
        showNotification(
          t("alerts.noExecutablePath", { name: conn.process_name || "Unknown" }),
          'info'
        );
      }
    } catch (error) {
      console.error(t("alerts.openFolderFailed", { error }), error);
      showNotification(
        t("alerts.openFolderFailed", { error }),
        'error'
      );
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
  // 在排序前重新获取最新连接数据，确保排序基于最新数据
  await loadConnections();

  if (sortColumn.value === column) {
    // 如果当前列已经是排序列，则切换排序方向
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    // 如果切换到新列，则默认升序
    sortColumn.value = column;
    sortDirection.value = "asc";
  }

  // 重新应用排序（虽然loadConnections内部已经调用applySorting，但再次确保排序正确）
  applySorting();
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
onMounted(async () => {
  // 设置语言为本地存储的语言或浏览器语言
  const savedLang = localStorage.getItem("locale");
  if (savedLang && (savedLang === "zh" || savedLang === "en")) {
    const { locale } = useI18n();
    locale.value = savedLang as "zh" | "en";
  }

  // 检查并应用主题偏好
  checkSystemThemePreference();

  loadConnections();

  // 获取应用版本
  try {
    appVersion.value = await invoke("get_app_version");
  } catch (error) {
    console.error('Failed to get app version:', error);
    appVersion.value = 'Unknown';
  }

  // 监听文档上的点击事件，用于隐藏右键菜单
  document.addEventListener("click", hideContextMenu);

  // 禁用原生右键菜单
  document.addEventListener("contextmenu", disableNativeContextMenu);

  // 监视主题变化
  watch(
    isDarkMode,
    (newVal) => {
      if (newVal) {
        document.documentElement.classList.add("dark");
        localStorage.setItem("theme", "dark");
      } else {
        document.documentElement.classList.remove("dark");
        localStorage.setItem("theme", "light");
      }
    },
    { immediate: true },
  );
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

  // 移除文档点击事件监听器
  document.removeEventListener("click", hideContextMenu);
  
  // 移除右键菜单事件监听器
  document.removeEventListener("contextmenu", disableNativeContextMenu);
});

// 禁用原生右键菜单的函数
function disableNativeContextMenu(e: MouseEvent) {
  // 总是阻止原生右键菜单，让应用的自定义右键菜单处理所有情况
  e.preventDefault();
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

// 显示通知
function showNotification(message: string, type: 'success' | 'error' | 'info' = 'info') {
  notificationMessage.value = message;
  notificationType.value = type;
  showNotificationBox.value = true;

  // 3秒后自动隐藏通知
  setTimeout(() => {
    showNotificationBox.value = false;
  }, 3000);
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
    <MenuBar
      :filterProtocol="filterProtocol"
      :filterState="filterState"
      :searchProcessName="searchProcessName"
      :searchLocalPort="searchLocalPort"
      :isAutoRefreshEnabled="isAutoRefreshEnabled"
      :selectedRefreshInterval="selectedRefreshInterval"
      :refreshIntervals="refreshIntervals"
      :isDarkMode="isDarkMode"
      @update:filterProtocol="filterProtocol = $event"
      @update:filterState="
        filterState = $event;
        applyFiltersAndSearch();
      "
      @update:searchProcessName="searchProcessName = $event"
      @update:searchLocalPort="searchLocalPort = $event"
      @applyFiltersAndSearch="applyFiltersAndSearch"
      @toggleAutoRefresh="toggleAutoRefresh"
      @changeRefreshInterval="changeRefreshInterval"
      @update:selectedRefreshInterval="selectedRefreshInterval = $event"
      @changeLanguage="changeLanguage"
      @toggleTheme="toggleTheme"
      @setProtocolFilter="setProtocolFilter"
      @showAboutDialog="showAbout = true"
    />

    <!-- 表格 -->
    <ConnectionsTable
      :connections="connections"
      :clickedConnection="clickedConnection"
      :sortColumn="sortColumn"
      :sortDirection="sortDirection"
      :customColumnWidths="customColumnWidths"
      @update:clickedConnection="clickedConnection = $event"
      @toggleSort="toggleSort"
      @showContextMenuHandler="showContextMenuHandler"
      @showProcessDetailsDialog="showProcessDetailsDialog"
      @update:customColumnWidths="customColumnWidths = $event"
    />

    <!-- 右键菜单 -->
    <ContextMenu
      :showContextMenu="showContextMenu"
      :contextMenuPosition="contextMenuPosition"
      :selectedConnection="selectedConnection"
      @update:showContextMenu="showContextMenu = $event"
      @showProcessDetailsDialog="showProcessDetailsDialog"
      @killProcess="killProcess"
      @openContainingFolder="openContainingFolder"
    />

    <!-- 状态栏 -->
    <StatusBar :statusBarInfo="statusBarInfo" />

    <!-- 进程详情弹窗 -->
    <ProcessDetailsModal
      :showProcessDetails="showProcessDetails"
      :processDetails="processDetails"
      :processIcon="selectedConnection?.icon || null"
      @update:showProcessDetails="showProcessDetails = $event"
    />

    <!-- 关于对话框 -->
    <AboutDialog
      :showAbout="showAbout"
      :appVersion="appVersion"
      @update:showAbout="showAbout = $event"
    />

    <!-- 通知提示框 -->
    <div
      v-if="showNotificationBox"
      class="notification-box"
      :class="[notificationType, isDarkMode ? 'dark' : '']"
    >
      <div class="notification-content">
        <span class="notification-message">{{ notificationMessage }}</span>
        <button
          class="notification-close-btn"
          @click="showNotificationBox = false"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* App.vue只需要保留容器的基本样式 */
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

.dark :root {
  color: #f6f6f6;
  background-color: #111827;
}

/* 暗色主题下的链接样式 */
.dark a:hover {
  color: #24c8db;
}

/* 深色模式下的通用样式 */
.dark {
  background-color: #1a202c; /* 深灰蓝 */
  color: #a0aec0; /* 中等亮度的灰蓝，确保可读性 */
}

/* 暗色主题下的滚动条样式 */
.dark ::-webkit-scrollbar {
  width: 12px; /* 纵向滚动条宽度 */
  height: 12px; /* 横向滚动条高度 */
}

.dark ::-webkit-scrollbar-track {
  background: #1a202c; /* 与背景色一致 */
  border-radius: 6px;
}

.dark ::-webkit-scrollbar-thumb {
  background: #2d3748; /* 与菜单背景色相近但略深 */
  border-radius: 6px;
}

.dark ::-webkit-scrollbar-thumb:hover {
  background: #4a5568; /* 悬停时的颜色 */
}

/* 亮色主题下的滚动条样式 */
::-webkit-scrollbar {
  width: 12px; /* 纵向滚动条宽度 */
  height: 12px; /* 横向滚动条高度 */
}

::-webkit-scrollbar-track {
  background: #f3f4f6; /* 与亮色主题背景一致 */
  border-radius: 6px;
}

::-webkit-scrollbar-thumb {
  background: #cbd5e0; /* 与亮色主题边框色相近 */
  border-radius: 6px;
}

::-webkit-scrollbar-thumb:hover {
  background: #a0aec0; /* 悬停时的颜色 */
}

/* 通知提示框样式 */
.notification-box {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 10000;
  min-width: 300px;
  max-width: 500px;
  border-radius: 8px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  animation: slideInRight 0.3s ease-out;
}

.notification-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-radius: 8px;
}

.notification-message {
  flex: 1;
  margin-right: 10px;
  word-break: break-word;
}

.notification-close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: inherit;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.notification-close-btn:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

/* 成功通知样式 */
.notification-box.success .notification-content {
  background-color: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

/* 错误通知样式 */
.notification-box.error .notification-content {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

/* 信息通知样式 */
.notification-box.info .notification-content {
  background-color: #d1ecf1;
  color: #0c5460;
  border: 1px solid #bee5eb;
}

/* 暗色主题下的通知样式 */
.notification-box.dark.success .notification-content {
  background-color: #1d3c25;
  color: #8bd49b;
  border: 1px solid #2d5c45;
}

.notification-box.dark.error .notification-content {
  background-color: #572e37;
  color: #e6a7b3;
  border: 1px solid #7a404d;
}

.notification-box.dark.info .notification-content {
  background-color: #2a4d5c;
  color: #8dd1e1;
  border: 1px solid #3a6a7d;
}

@keyframes slideInRight {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
