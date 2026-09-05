<template>
  <div
    v-if="showProcessDetails"
    class="modal-overlay"
    @click="emit('update:showProcessDetails', false)"
  >
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <div class="header-with-icon">
          <img
            :src="
              processIcon ? 'data:image/png;base64,' + processIcon : '/exe.svg'
            "
            :alt="processDetails?.name || 'Process Icon'"
            class="process-icon-large"
          />
          <h3>{{ t("modal.processDetails") }}</h3>
        </div>
        <button
          class="icon-close"
          @click="emit('update:showProcessDetails', false)"
          :title="t('modal.processDetails')"
        >
          <X :size="16" />
        </button>
      </div>
      <div v-if="processDetails" class="process-details">
        <p>
          <strong>{{ t("modal.pid") }}</strong>
          <span>{{ processDetails.pid }}</span>
        </p>
        <p>
          <strong>{{ t("modal.name") }}</strong>
          <span>{{ processDetails.name }}</span>
        </p>
        <p>
          <strong>{{ t("modal.commandLine") }}</strong>
          <span>{{ processDetails.command_line }}</span>
        </p>
        <p>
          <strong>{{ t("modal.executablePath") }}</strong>
          <span class="path-container">
            <span>{{ processDetails.executable_path }}</span>
            <button
              v-if="
                props.processDetails &&
                props.processDetails.executable_path &&
                props.processDetails.executable_path !== ''
              "
              class="open-folder-btn"
              @click="openContainingFolder"
              :title="t('modal.openFolder')"
            >
              <FolderOpen :size="15" />
            </button>
          </span>
        </p>
        <p>
          <strong>{{ t("modal.memoryUsage") }}</strong>
          <span>{{ formatMemoryUsage(processDetails.memory_usage) }}</span>
        </p>
        <p>
          <strong>{{ t("modal.cpuUsage") }}</strong>
          <span>{{ processDetails.cpu_usage }}%</span>
        </p>
        <p>
          <strong>{{ t("modal.parentPid") }}</strong>
          <span>{{ processDetails.parent_pid }}</span>
        </p>
        <p>
          <strong>{{ t("modal.startTime") }}</strong>
          <span>{{ formatDate(processDetails.start_time) }}</span>
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { X, FolderOpen } from "lucide-vue-next";
import type { ProcessDetails } from "@/types/connection";

// 定义组件属性
interface Props {
  showProcessDetails: boolean;
  processDetails: ProcessDetails | null;
  processIcon: string | null; // Base64 encoded icon data
}

const props = defineProps<Props>();

// 定义事件发射器
interface Emits {
  (e: "update:showProcessDetails", value: boolean): void;
}

const emit = defineEmits<Emits>();

// 使用国际化
const { t } = useI18n();

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
  justify-content: space-between;
  align-items: center;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
}

.header-with-icon {
  display: flex;
  align-items: center;
  gap: 10px;
}

.process-icon-large {
  width: 32px;
  height: 32px;
  object-fit: contain;
  flex-shrink: 0;
}

.modal-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

/* 统一的幽灵关闭按钮 */
.icon-close {
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

.icon-close:hover {
  background-color: var(--bg-hover);
  color: var(--text-1);
}

.process-details {
  padding: 16px 20px 20px;
  overflow-y: auto;
  flex-grow: 1;
}

.process-details p {
  margin: 8px 0;
  font-size: 13.5px;
  color: var(--text-2);
  line-height: 1.45;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.process-details strong {
  color: var(--text-3);
  font-size: 12px;
  font-weight: 500;
  word-break: break-all;
  overflow-wrap: break-word;
}

.process-details span {
  color: var(--text-1);
  word-break: break-all;
  overflow-wrap: break-word;
  white-space: pre-wrap;
}

.path-container {
  display: flex;
  align-items: center;
  gap: 6px;
}

.open-folder-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-2);
  cursor: pointer;
  flex-shrink: 0;
  transition: border-color 0.15s ease, color 0.15s ease,
    background-color 0.15s ease;
}

.open-folder-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background-color: var(--accent-weak);
}
</style>
