<template>
  <div v-if="show" class="message-box-overlay" @click="close">
    <div class="message-box" @click.stop>
      <div class="message-header">
        <span class="message-title">{{ title }}</span>
        <button class="close-btn" @click="close">X</button>
      </div>
      <div class="message-content">
        <div v-if="icon" class="message-icon">
          <img :src="icon" alt="icon" />
        </div>
        <div class="message-text">{{ message }}</div>
      </div>
      <div class="message-actions">
        <button class="confirm-btn" @click="confirm">{{ confirmText }}</button>
        <button v-if="showCancel" class="cancel-btn" @click="cancel">{{ cancelText }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  show: boolean;
  type?: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  showCancel?: boolean;
  icon?: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  title: '',
  confirmText: '确定',
  cancelText: '取消',
  showCancel: false,
  icon: undefined,
});

// 定义事件
interface Emits {
  (e: 'confirm'): void;
  (e: 'cancel'): void;
  (e: 'close'): void;
}

const emit = defineEmits<Emits>();

// 根据类型设置标题
const defaultTitle = computed(() => {
  switch (props.type) {
    case 'success':
      return '成功';
    case 'warning':
      return '警告';
    case 'error':
      return '错误';
    default:
      return '提示';
  }
});

// 确认操作
const confirm = () => {
  emit('confirm');
  emit('close');
};

// 取消操作
const cancel = () => {
  emit('cancel');
  emit('close');
};

// 关闭弹窗
const close = () => {
  emit('close');
};
</script>

<style scoped>
.message-box-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10000;
}

.message-box {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 300px;
  max-width: 500px;
  overflow: hidden;
}

.dark .message-box {
  background-color: #2d3748;
  color: #e2e8f0;
}

.message-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #e2e8f0;
  background-color: #f8fafc;
}

.dark .message-header {
  border-bottom: 1px solid #4a5568;
  background-color: #1a202c;
}

.message-title {
  font-size: 16px;
  font-weight: 600;
  color: #1e293b;
}

.dark .message-title {
  color: #e2e8f0;
}

.close-btn {
  background: #fef2f2; /* 浅红色背景 */
  border: 1px solid #ddd6fe; /* 红色系边框 */
  font-size: 16px; /* 调整字体大小 */
  cursor: pointer;
  color: #dc2626; /* 红色文字 */
  padding: 0;
  width: 24px;
  height: 24px;
  border-radius: 50%; /* 圆形 */
  font-weight: bold;
  box-sizing: border-box;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1; /* 确保行高为1以实现垂直居中 */
}

.close-btn:hover {
  background-color: #fecaca; /* 悬停时的浅红色背景 */
  color: #dc2626; /* 悬停时的深红色 */
  border-color: #fca5a5; /* 悬停时的红色边框 */
}

/* 暗色主题下的关闭按钮样式 */
.dark .close-btn {
  background: #3f3f46; /* 暗灰色背景 */
  border: 1px solid #52525b; /* 暗灰边框 */
  color: #f43f5e; /* 红色文字 */
}

.dark .close-btn:hover {
  background-color: #52525b; /* 悬停时的暗灰色 */
  color: #fb7185; /* 悬停时的浅红色 */
  border-color: #7f1d1d; /* 暗红边框 */
}

.message-content {
  display: flex;
  padding: 20px;
  align-items: flex-start;
}

.message-icon {
  margin-right: 12px;
  width: 24px;
  height: 24px;
}

.message-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.message-text {
  flex: 1;
  color: #334155;
  line-height: 1.5;
}

.dark .message-text {
  color: #cbd5e0;
}

.message-actions {
  display: flex;
  justify-content: flex-end;
  padding: 16px;
  gap: 8px;
  border-top: 1px solid #e2e8f0;
  background-color: #f8fafc;
}

.dark .message-actions {
  border-top: 1px solid #4a5568;
  background-color: #1a202c;
}

.confirm-btn, .cancel-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.confirm-btn {
  background-color: #3b82f6;
  color: white;
}

.confirm-btn:hover {
  background-color: #2563eb;
}

.cancel-btn {
  background-color: #e2e8f0;
  color: #475569;
}

.cancel-btn:hover {
  background-color: #cbd5e0;
}

.dark .confirm-btn {
  background-color: #4c6ef5;
}

.dark .confirm-btn:hover {
  background-color: #3b5bdb;
}

.dark .cancel-btn {
  background-color: #4a5568;
  color: #e2e8f0;
}

.dark .cancel-btn:hover {
  background-color: #718096;
}
</style>