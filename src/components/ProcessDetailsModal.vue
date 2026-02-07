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
            :src="processIcon ? 'data:image/png;base64,' + processIcon : '/src/assets/exe.svg'"
            :alt="processDetails?.name || 'Process Icon'"
            class="process-icon-large"
          />
          <h3>{{ t("modal.processDetails") }}</h3>
        </div>
        <button class="close-button" @click="emit('update:showProcessDetails', false)">
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
          <span class="path-container">
            <span>{{ processDetails.executable_path }}</span>
            <button 
              v-if="props.processDetails && props.processDetails.executable_path && props.processDetails.executable_path !== ''"
              class="open-folder-btn" 
              @click="openContainingFolder"
              :title="t('modal.openFolder')"
            >
              📁
            </button>
          </span>
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
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";

// 定义进程详情类型
export interface ProcessDetails {
  pid: number;
  name: string;
  command_line: string;
  executable_path: string;
  memory_usage: number;
  cpu_usage: number;
  parent_pid: number;
  start_time: number;
}

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
      await invoke('open_folder', { path: props.processDetails.executable_path });
    } catch (error) {
      console.error('Failed to open folder:', error);
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

.header-with-icon {
  display: flex;
  align-items: center;
  gap: 10px; /* 图标与文字之间的间距 */
}

.process-icon-large {
  width: 32px;
  height: 32px;
  object-fit: contain;
  flex-shrink: 0; /* 防止图标被压缩 */
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

.path-container {
  display: flex;
  align-items: center;
  gap: 5px;
}

.open-folder-btn {
  background: none !important;
  border: none !important;
  border-radius: 4px;
  cursor: pointer;
  padding: 2px 6px;
  font-size: 16px;
  color: #3b82f6; /* 蓝色文字 */
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0; /* 防止按钮被压缩 */
  box-shadow: none !important; /* 确保没有阴影 */
  outline: none !important; /* 确保没有轮廓 */
}

.open-folder-btn:hover {
  color: #2563eb; /* 更深的蓝色 */
  background: transparent !important; /* 保持透明背景 */
  border: none !important; /* 确保边框为无 */
  box-shadow: none !important; /* 确保没有阴影 */
}

.dark .open-folder-btn {
  color: #60a5fa; /* 暗色主题下的蓝色 */
}

.dark .open-folder-btn:hover {
  color: #3b82f6; /* 暗色主题下的更深蓝色 */
  background: transparent !important; /* 保持透明背景 */
  border: none !important; /* 确保边框为无 */
  box-shadow: none !important; /* 确保没有阴影 */
}

/* 暗色主题下的弹窗样式 */
.dark .modal-content {
  background: #1f2937;
  color: #e5e7eb;
}

.dark .modal-header {
  border-bottom: 1px solid #374151;
  color: #f3f4f6;
}

.dark .modal-header h3 {
  color: #f3f4f6; /* 确保标题文字可见 */
}

.dark .close-button {
  background: #ef4444; /* 红色背景 */
}

.dark .process-details strong {
  color: #f3f4f6;
}

.dark .process-details span {
  color: #e5e7eb;
}
</style>