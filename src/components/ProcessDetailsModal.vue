<template>
  <div
    v-if="showProcessDetails"
    class="modal-overlay"
    @click="emit('update:showProcessDetails', false)"
  >
    <div class="modal-content" @click.stop>
      <!-- 头部：进程身份 -->
      <div class="modal-header">
        <img
          :src="
            processIcon ? 'data:image/png;base64,' + processIcon : '/exe.svg'
          "
          :alt="processDetails?.name || 'Process Icon'"
          class="process-icon-large"
        />
        <div class="header-info">
          <h3 class="process-name">
            {{ processDetails?.name || t("modal.processDetails") }}
          </h3>
          <span v-if="processDetails" class="pid-chip">
            PID {{ processDetails.pid }}
          </span>
        </div>
        <div class="header-actions">
          <button
            class="icon-btn"
            :class="{ spinning: refreshing }"
            @click="emit('refresh')"
            :title="t('modal.refresh')"
          >
            <RefreshCw :size="15" />
          </button>
          <button
            class="icon-btn"
            @click="emit('update:showProcessDetails', false)"
          >
            <X :size="16" />
          </button>
        </div>
      </div>

      <div v-if="processDetails" class="process-details">
        <!-- 进程已退出提示 -->
        <div v-if="processGone" class="gone-banner">
          <AlertTriangle :size="14" />
          <span>{{ t("modal.processGone") }}</span>
        </div>

        <!-- 概览指标卡片 -->
        <div class="stats-grid">
          <div class="stat-card">
            <span class="stat-icon"><MemoryStick :size="15" /></span>
            <div class="stat-info">
              <span class="stat-label">{{ t("modal.memoryUsage") }}</span>
              <span class="stat-value">{{
                formatMemoryUsage(processDetails.memory_usage)
              }}</span>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-icon"><Cpu :size="15" /></span>
            <div class="stat-info">
              <span class="stat-label">{{ t("modal.cpuUsage") }}</span>
              <span class="stat-value">{{ processDetails.cpu_usage.toFixed(1) }}%</span>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-icon"><Clock :size="15" /></span>
            <div class="stat-info">
              <span class="stat-label">{{ t("modal.startTime") }}</span>
              <span class="stat-value">{{ formatDate(processDetails.start_time) }}</span>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-icon"><GitFork :size="15" /></span>
            <div class="stat-info">
              <span class="stat-label">{{ t("modal.parentPid") }}</span>
              <span class="stat-value">{{ processDetails.parent_pid ?? "-" }}</span>
            </div>
          </div>
        </div>

        <!-- 命令行 -->
        <div class="detail-section">
          <div class="detail-label-row">
            <span class="detail-label">{{ t("modal.commandLine") }}</span>
            <button
              v-if="processDetails.command_line"
              class="mini-btn"
              :class="{ copied: copiedKey === 'cmd' }"
              @click="copyText(processDetails.command_line, 'cmd')"
              :title="copiedKey === 'cmd' ? t('modal.copied') : t('modal.copy')"
            >
              <Check v-if="copiedKey === 'cmd'" :size="13" />
              <Copy v-else :size="13" />
            </button>
          </div>
          <div class="code-block">{{ processDetails.command_line || "-" }}</div>
        </div>

        <!-- 执行路径 -->
        <div class="detail-section">
          <div class="detail-label-row">
            <span class="detail-label">{{ t("modal.executablePath") }}</span>
            <span class="label-actions">
              <button
                v-if="processDetails.executable_path"
                class="mini-btn"
                :class="{ copied: copiedKey === 'path' }"
                @click="copyText(processDetails.executable_path, 'path')"
                :title="
                  copiedKey === 'path' ? t('modal.copied') : t('modal.copy')
                "
              >
                <Check v-if="copiedKey === 'path'" :size="13" />
                <Copy v-else :size="13" />
              </button>
              <button
                v-if="
                  props.processDetails &&
                  props.processDetails.executable_path &&
                  props.processDetails.executable_path !== ''
                "
                class="mini-btn"
                @click="openContainingFolder"
                :title="t('modal.openFolder')"
              >
                <FolderOpen :size="13" />
              </button>
            </span>
          </div>
          <div class="code-block">{{ processDetails.executable_path || "-" }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  Check,
  Clock,
  Copy,
  Cpu,
  FolderOpen,
  GitFork,
  MemoryStick,
  RefreshCw,
  X,
} from "lucide-vue-next";
import type { ProcessDetails } from "@/types/connection";

// 定义组件属性
interface Props {
  showProcessDetails: boolean;
  processDetails: ProcessDetails | null;
  processIcon: string | null; // Base64 encoded icon data
  processGone?: boolean; // 进程是否已退出（轮询时发现）
  refreshing?: boolean; // 手动刷新进行中
}

const props = defineProps<Props>();

// 定义事件发射器
interface Emits {
  (e: "update:showProcessDetails", value: boolean): void;
  (e: "refresh"): void;
}

const emit = defineEmits<Emits>();

// 使用国际化
const { t } = useI18n();

// 复制反馈：记录刚复制完成的字段 key，短暂显示对勾
const copiedKey = ref<string | null>(null);
let copiedTimer: number | null = null;

const copyText = async (text: string, key: string) => {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // 剪贴板 API 不可用时退化为 execCommand
    const textarea = document.createElement("textarea");
    textarea.value = text;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand("copy");
    document.body.removeChild(textarea);
  }
  copiedKey.value = key;
  if (copiedTimer !== null) {
    window.clearTimeout(copiedTimer);
  }
  copiedTimer = window.setTimeout(() => {
    copiedKey.value = null;
  }, 1500);
};

// 打开文件所在目录
const openContainingFolder = async () => {
  if (props.processDetails && props.processDetails.executable_path) {
    try {
      await invoke("open_folder", {
        path: props.processDetails.executable_path,
      });
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  }
};

// 格式化内存使用量显示
const formatMemoryUsage = (memoryInBytes: number): string => {
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
/* 弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(9, 14, 22, 0.45);
  backdrop-filter: blur(2px);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10001;
}

.modal-content {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  width: 500px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  color: var(--text-1);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
}

.process-icon-large {
  width: 36px;
  height: 36px;
  object-fit: contain;
  flex-shrink: 0;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.process-name {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pid-chip {
  width: fit-content;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-2);
  background: var(--bg-subtle);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 1px 8px;
}

/* 头部操作按钮（刷新 / 关闭） */
.header-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-3);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.icon-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-1);
}

.icon-btn.spinning {
  color: var(--accent);
  animation: pv-spin 0.8s linear infinite;
}

/* 进程已退出提示条 */
.gone-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 14px;
  border-radius: var(--radius-sm);
  background: var(--warning-weak);
  border: 1px solid var(--warning);
  color: var(--warning);
  font-size: 12.5px;
}

.process-details {
  padding: 16px 20px 20px;
  overflow-y: auto;
  flex-grow: 1;
}

/* ===== 概览指标卡片 ===== */
.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 16px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg-subtle);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  min-width: 0;
}

.stat-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: var(--accent-weak);
  color: var(--accent);
  flex-shrink: 0;
}

.stat-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.stat-label {
  font-size: 11px;
  color: var(--text-3);
}

.stat-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-variant-numeric: tabular-nums;
}

/* ===== 命令行 / 路径区块 ===== */
.detail-section {
  margin-bottom: 14px;
}

.detail-section:last-child {
  margin-bottom: 0;
}

.detail-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.detail-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-3);
}

.label-actions {
  display: inline-flex;
  gap: 4px;
}

.mini-btn {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-2);
  cursor: pointer;
  transition: border-color 0.15s ease, color 0.15s ease,
    background-color 0.15s ease;
}

.mini-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background-color: var(--accent-weak);
}

.mini-btn.copied {
  border-color: var(--success);
  color: var(--success);
  background-color: var(--success-weak);
}

.code-block {
  background: var(--bg-inset);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-1);
  white-space: pre-wrap;
  word-break: break-all;
  overflow-wrap: anywhere;
  max-height: 96px;
  overflow-y: auto;
}
</style>
