<template>
  <!-- 状态栏 -->
  <div class="status-bar">
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.totalConnections") }}</span>
      <span class="status-value">{{ statusBarInfo.totalConnections }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.tcpConnections") }}</span>
      <span class="status-value tcp-count">{{
        statusBarInfo.tcpConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.udpConnections") }}</span>
      <span class="status-value udp-count">{{
        statusBarInfo.udpConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.established") }}</span>
      <span class="status-value established-count">{{
        statusBarInfo.establishedConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.listen") }}</span>
      <span class="status-value listen-count">{{
        statusBarInfo.listenConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.wait") }}</span>
      <span class="status-value wait-count">{{
        statusBarInfo.timeWaitConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.closeWait") }}</span>
      <span class="status-value close-wait-count">{{
        statusBarInfo.closeWaitConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.other") }}</span>
      <span class="status-value other-count">{{
        statusBarInfo.otherConnections
      }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.kernel") }}</span>
      <span class="status-value kernel-count">{{
        statusBarInfo.kernelConnections
      }}</span>
    </div>
    <div class="status-item status-item--push">
      <span class="status-label net-down">↓</span>
      <span class="status-value net-down">{{ formatSpeed(statusBarInfo.netDown) }}</span>
      <span class="status-total">{{
        t("statusBar.sessionTotal")
      }}{{ formatBytes(statusBarInfo.netTotalDown) }}</span>
    </div>
    <div class="status-item">
      <span class="status-label net-up">↑</span>
      <span class="status-value net-up">{{ formatSpeed(statusBarInfo.netUp) }}</span>
      <span class="status-total">{{
        t("statusBar.sessionTotal")
      }}{{ formatBytes(statusBarInfo.netTotalUp) }}</span>
    </div>
    <div class="status-item">
      <span class="status-label">{{ t("statusBar.lastUpdate") }}</span>
      <span class="status-value">{{ statusBarInfo.lastUpdate }}</span>
    </div>
    <div class="status-item" v-if="statusBarInfo.refreshInterval">
      <span class="status-label">{{ t("statusBar.refreshInterval") }}</span>
      <span class="status-value"
        >{{ statusBarInfo.refreshInterval
        }}{{ t("menu.refreshInterval") }}</span
      >
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";

// 定义状态栏信息接口
interface StatusBarInfo {
  totalConnections: number;
  tcpConnections: number;
  udpConnections: number;
  kernelConnections: number;
  establishedConnections: number;
  listenConnections: number;
  timeWaitConnections: number;
  closeWaitConnections: number;
  otherConnections: number;
  netDown: number;
  netUp: number;
  netTotalDown: number;
  netTotalUp: number;
  lastUpdate: string;
  refreshInterval: number | null;
}

// 定义组件属性
interface Props {
  statusBarInfo: StatusBarInfo;
}

defineProps<Props>();

// 使用国际化
const { t } = useI18n();

// 速率格式化：B/s → KB/s → MB/s → GB/s
const formatSpeed = (bytesPerSecond: number): string => {
  if (!bytesPerSecond || bytesPerSecond <= 0) {
    return "0 B/s";
  }
  const units = ["B/s", "KB/s", "MB/s", "GB/s"];
  let value = bytesPerSecond;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  const text = value >= 100 ? value.toFixed(0) : value.toFixed(1);
  return `${text} ${units[index]}`;
};

// 字节量格式化（不带速率单位）：B → KB → MB → GB → TB
const formatBytes = (bytes: number): string => {
  if (!bytes || bytes <= 0) {
    return "0 B";
  }
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  const text = value >= 100 ? value.toFixed(0) : value.toFixed(1);
  return `${text} ${units[index]}`;
};
</script>

<style scoped>
/* 状态栏 */
.status-bar {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  background-color: var(--bg-panel);
  border-top: 1px solid var(--border);
  font-size: 12px;
  color: var(--text-2);
  min-height: 26px;
  flex-shrink: 0;
  flex-wrap: nowrap;
  overflow: hidden;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

/* 相邻统计项之间用细分隔线，间距收紧 */
.status-item + .status-item {
  border-left: 1px solid var(--border);
  margin-left: 12px;
  padding-left: 12px;
}

/* 最后更新时间等推到右侧 */
.status-item--push {
  margin-left: auto !important;
  padding-left: 16px !important;
}

.status-label {
  font-weight: 400;
  color: var(--text-3);
}

.status-value {
  font-weight: 600;
  color: var(--text-1);
  font-variant-numeric: tabular-nums;
}

.status-value.tcp-count {
  color: var(--chart-blue);
}

.status-value.udp-count {
  color: var(--chart-orange);
}

.status-value.established-count {
  color: var(--chart-green);
}

.status-value.listen-count {
  color: var(--chart-cyan);
}

.status-value.wait-count {
  color: var(--chart-yellow);
}

.status-value.close-wait-count {
  color: var(--chart-rose);
}

.status-value.other-count {
  color: var(--chart-violet);
}

.status-value.kernel-count {
  color: var(--danger);
}

.status-value.net-down,
.status-label.net-down {
  color: var(--chart-green);
}

.status-value.net-up,
.status-label.net-up {
  color: var(--chart-blue);
}
</style>
