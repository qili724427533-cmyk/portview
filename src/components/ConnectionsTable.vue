<template>
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
</template>

<script setup lang="ts">
import { computed, watch, nextTick, onMounted } from "vue";
import { useI18n } from "vue-i18n";

// 定义连接数据类型
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
  hasChanged?: boolean; // 标记连接是否有变化
}

// 定义组件属性
interface Props {
  connections: TcpConnection[];
  clickedConnection: TcpConnection | null;
  sortColumn: string | null;
  sortDirection: "asc" | "desc";
  customColumnWidths: Record<string, number>;
}

const props = defineProps<Props>();

// 定义事件发射器
interface Emits {
  (e: "update:clickedConnection", value: TcpConnection): void;
  (e: "toggleSort", column: "process_name" | "pid" | "protocol" | "local_addr" | "local_port" | "remote_addr" | "remote_port" | "state" | "start_time"): void;
  (e: "showContextMenuHandler", conn: TcpConnection, event: MouseEvent): void;
  (e: "showProcessDetailsDialog", conn: TcpConnection): void;
  (e: "update:customColumnWidths", value: Record<string, number>): void;
}

const emit = defineEmits<Emits>();

// 使用国际化
const { t } = useI18n();

// 从props获取数据
const connections = computed(() => props.connections);
const clickedConnection = computed(() => props.clickedConnection);

// 排序相关
const sortColumn = computed(() => props.sortColumn);
const sortDirection = computed(() => props.sortDirection);

// 列宽相关
const customColumnWidths = computed(() => props.customColumnWidths);

// 在组件挂载后应用列宽
onMounted(() => {
  nextTick(() => {
    applyCustomColumnWidths();
  });
});

// 监听列宽变化并应用
watch(customColumnWidths, () => {
  nextTick(() => {
    applyCustomColumnWidths();
  });
}, { deep: true });

// 监听连接数据变化并应用列宽
watch(connections, () => {
  nextTick(() => {
    applyCustomColumnWidths();
  });
}, { deep: true });

// 方法
const toggleSort = (column: "process_name" | "pid" | "protocol" | "local_addr" | "local_port" | "remote_addr" | "remote_port" | "state" | "start_time") => {
  emit('toggleSort', column);
};

// 列宽拖拽相关变量
let isDragging = false;
let dragStartX = 0;
let dragStartWidth = 0;
let currentColumnIndex = -1;

const startColumnResize = (event: MouseEvent, columnIndex: number) => {
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
  const initialColumnWidths: number[] = [];
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
};

// 处理列宽调整
const handleColumnResize = (event: MouseEvent) => {
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
      const updatedWidths = { ...props.customColumnWidths };
      updatedWidths[columnOrder[currentColumnIndex]] = newWidth;
      emit('update:customColumnWidths', updatedWidths);
    }
  }
};

// 结束列宽调整
const stopColumnResize = () => {
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
};

const showContextMenuHandler = (conn: TcpConnection, event: MouseEvent) => {
  emit('showContextMenuHandler', conn, event);
};

const showProcessDetailsDialog = (conn: TcpConnection) => {
  emit('showProcessDetailsDialog', conn);
};

const applyCustomColumnWidths = () => {
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

        if (props.customColumnWidths[col]) {
          // 使用自定义宽度
          colWidth = props.customColumnWidths[col];
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
};

// 判断是否为内核进程
const isKernelProcess = (processName: string | null): boolean => {
  return (
    processName === "[KERNEL]" ||
    (processName !== null && processName.includes("[KERNEL]"))
  );
};

// 格式化日期时间显示
const formatDate = (timestamp: number | null): string => {
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
};
</script>

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

/* 内核进程样式 */
.kernel-process {
  font-weight: bold;
  color: #7c2d12; /* 深红棕色，区别于普通进程 */
  background-color: #fef2f2; /* 浅红背景 */
  padding: 2px 4px;
  border-radius: 3px;
}

/* 表格暗色主题样式 */
.dark .connections-table thead tr {
  background-color: #1a202c; /* 深灰蓝 */
  color: #a0aec0; /* 中等亮度的灰蓝 */
  border-bottom: 2px solid #2d3748; /* 深中灰蓝 */
}

.dark .connections-table th,
.dark .connections-table td {
  border-bottom: 1px solid #2d3748; /* 深中灰蓝 */
  color: #a0aec0; /* 中等亮度的灰蓝 */
}

.dark .connections-table th:not(:last-child),
.dark .connections-table td:not(:last-child) {
  border-right: 1px solid #2d3748; /* 深中灰蓝 */
}

.dark .connections-table tbody tr:nth-of-type(even) {
  background-color: #2d3748; /* 深中灰蓝 */
}

.dark .connections-table tbody tr:nth-of-type(odd) {
  background-color: #1a202c; /* 深灰蓝 */
}

.dark .connections-table tbody tr:hover {
  background-color: #4a5568; /* 中灰蓝 */
}

.dark .connections-table tbody tr.selected-row {
  background-color: #3c4bcb !important; /* 深蓝紫色 */
}

.dark .connections-table tbody tr.selected-row td,
.dark .connections-table tbody tr.selected-row th {
  color: #e2e8f0 !important; /* 浅灰蓝 */
}

/* 状态变化的连接项在暗色主题下的样式 */
.dark .connections-table tbody tr.changed-connection {
  background-color: #b7791f !important; /* 深金黄色 */
}

.dark .connections-table tbody tr.changed-connection td,
.dark .connections-table tbody tr.changed-connection th {
  color: #e2e8f0 !important; /* 浅灰蓝 */
}

/* 内核进程在暗色主题下的样式 */
.dark .kernel-process {
  color: #f8b4b4; /* 浅粉红 */
  background-color: #8b2525; /* 深红棕 */
}
</style>