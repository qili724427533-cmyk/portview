<template>
  <div v-if="showContextMenu" class="context-menu" :style="contextMenuStyle">
    <ul>
      <li @click="handleShowProcessDetails">
        {{ t("contextMenu.processDetails") }}
      </li>
      <li @click="handleKillProcess">
        {{ t("contextMenu.killProcess") }}
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { TcpConnection } from "./ConnectionsTable.vue"; // 导入TcpConnection类型

// 定义组件属性
interface Props {
  showContextMenu: boolean;
  contextMenuPosition: { x: number; y: number };
  selectedConnection: TcpConnection | null;
}

const props = defineProps<Props>();

// 定义事件发射器
interface Emits {
  (e: "update:showContextMenu", value: boolean): void;
  (e: "showProcessDetailsDialog", conn: TcpConnection): void;
  (e: "killProcess", conn: TcpConnection): void;
}

const emit = defineEmits<Emits>();

// 使用国际化
const { t } = useI18n();

// 计算右键菜单样式
const contextMenuStyle = computed(() => {
  return {
    top: `${props.contextMenuPosition.y}px`,
    left: `${props.contextMenuPosition.x}px`,
    position: "fixed" as const,
    zIndex: 1000,
  };
});

// 方法
const handleShowProcessDetails = () => {
  if (props.selectedConnection) {
    emit('showProcessDetailsDialog', props.selectedConnection);
  }
};

const handleKillProcess = () => {
  if (props.selectedConnection) {
    emit('killProcess', props.selectedConnection);
  }
};
</script>

<style scoped>
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

/* 暗色主题下的右键菜单样式 */
.dark .context-menu {
  background: #1f2937;
  border: 1px solid #374151;
  color: #f3f4f6;
}

.dark .context-menu li {
  color: #d1d5db;
}

.dark .context-menu li:hover {
  background-color: #374151;
}

.dark .context-menu li:active {
  background-color: #4b5563;
}

.dark .context-menu li:not(:last-child) {
  border-bottom: 1px solid #4b5563;
}
</style>