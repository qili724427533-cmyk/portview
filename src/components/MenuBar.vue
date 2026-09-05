<template>
  <!-- 主菜单容器：按功能分组，空间不足时自动换行 -->
  <div class="menu-bar">
    <div class="menu-cluster">
      <div class="menu-group">
        <span class="menu-label">{{ t("menu.protocol") }}</span>
        <div class="segmented">
          <button
            :class="['segment-btn', { active: filterProtocol === 'all' }]"
            @click="setProtocolFilter('all')"
          >
            {{ t("menu.protocolAll") }}
          </button>
          <button
            :class="['segment-btn', { active: filterProtocol === 'TCP' }]"
            @click="setProtocolFilter('TCP')"
          >
            {{ t("menu.protocolTCP") }}
          </button>
          <button
            :class="['segment-btn', { active: filterProtocol === 'UDP' }]"
            @click="setProtocolFilter('UDP')"
          >
            {{ t("menu.protocolUDP") }}
          </button>
        </div>
      </div>

      <div class="menu-group">
        <span class="menu-label">{{ t("menu.state") }}</span>
        <select
          :value="filterState"
          @change="handleStateChange"
          class="control-select state-select"
        >
          <option value="all">{{ t("menu.stateAll") }}</option>
          <option value="LISTEN">{{ t("menu.stateListen") }}</option>
          <option value="ESTABLISHED">{{ t("menu.stateEstablished") }}</option>
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
    </div>

    <div class="menu-cluster">
      <div class="menu-group">
        <span class="menu-label">{{ t("menu.searchProcess") }}</span>
        <input
          type="text"
          :value="searchProcessName"
          @input="handleSearchProcessInput"
          :placeholder="t('menu.searchPlaceholder')"
          class="control-input"
        />
      </div>

      <div class="menu-group">
        <span class="menu-label">{{ t("menu.searchLocalPort") }}</span>
        <input
          type="text"
          :value="searchLocalPort"
          @input="handleSearchLocalPortInput"
          :placeholder="t('menu.localPortPlaceholder')"
          class="control-input control-input--port"
        />
      </div>
    </div>

    <div class="menu-cluster">
      <div class="menu-group">
        <span class="menu-label">{{ t("menu.autoRefresh") }}</span>
        <button
          :class="['control-btn', 'refresh-toggle-btn', { active: isAutoRefreshEnabled }]"
          @click="toggleAutoRefresh"
        >
          <RefreshCw
            class="btn-icon"
            :class="{ spinning: isAutoRefreshEnabled }"
            :size="13"
          />
          {{ isAutoRefreshEnabled ? t("menu.refreshStop") : t("menu.refreshStart") }}
        </button>
        <select
          :value="selectedRefreshInterval"
          @change="handleRefreshIntervalChange"
          class="control-select refresh-interval-select"
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

    <div class="menu-cluster">
      <div class="menu-group">
        <span class="menu-label">{{ t("menu.language") }}</span>
        <select
          :value="$i18n.locale"
          @change="handleLanguageChange"
          class="control-select lang-select"
        >
          <option value="zh">{{ t("zh") }}</option>
          <option value="en">{{ t("en") }}</option>
        </select>
      </div>

      <div class="menu-group">
        <button
          class="icon-btn"
          @click="handleThemeToggle"
          :title="isDarkMode ? t('menu.lightTheme') : t('menu.darkTheme')"
        >
          <Sun v-if="isDarkMode" :size="15" />
          <Moon v-else :size="15" />
        </button>
        <button class="control-btn about-btn" @click="showAboutDialog">
          {{ t("menu.about") }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { RefreshCw, Sun, Moon } from "lucide-vue-next";

// 定义组件属性
interface Props {
  filterProtocol: "all" | "TCP" | "UDP";
  filterState: string;
  searchProcessName: string;
  searchLocalPort: string;
  isAutoRefreshEnabled: boolean;
  selectedRefreshInterval: number;
  refreshIntervals: number[];
  isDarkMode: boolean;
}

defineProps<Props>();

// 定义事件发射器
interface Emits {
  (e: "update:filterProtocol", value: "all" | "TCP" | "UDP"): void;
  (e: "update:filterState", value: string): void;
  (e: "update:searchProcessName", value: string): void;
  (e: "update:searchLocalPort", value: string): void;
  (e: "applyFiltersAndSearch"): void;
  (e: "toggleAutoRefresh"): void;
  (e: "changeRefreshInterval", interval: number): void;
  (e: "update:selectedRefreshInterval", value: number): void;
  (e: "changeLanguage", lang: "zh" | "en"): void;
  (e: "toggleTheme"): void;
  (e: "setProtocolFilter", protocol: "all" | "TCP" | "UDP"): void;
  (e: "showAboutDialog"): void;
}

const emit = defineEmits<Emits>();

// 使用国际化
const { t } = useI18n();

// 方法
const setProtocolFilter = (protocol: "all" | "TCP" | "UDP") => {
  emit('setProtocolFilter', protocol);
};

const toggleAutoRefresh = () => {
  emit('toggleAutoRefresh');
};

const showAboutDialog = () => {
  emit('showAboutDialog');
};

const handleStateChange = (event: Event) => {
  const target = event.target as HTMLSelectElement;
  emit('update:filterState', target.value);
};

const handleSearchProcessInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit('update:searchProcessName', target.value);
  // 触发搜索
  emit('applyFiltersAndSearch');
};

const handleSearchLocalPortInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit('update:searchLocalPort', target.value);
  // 触发搜索
  emit('applyFiltersAndSearch');
};

const handleRefreshIntervalChange = (event: Event) => {
  const target = event.target as HTMLSelectElement;
  const interval = Number(target.value);
  emit('update:selectedRefreshInterval', interval);
  emit('changeRefreshInterval', interval);
};

const handleLanguageChange = (event: Event) => {
  const target = event.target as HTMLSelectElement;
  const lang = target.value as 'zh' | 'en';
  emit('changeLanguage', lang);
};

const handleThemeToggle = () => {
  emit('toggleTheme');
};
</script>

<style scoped>
/* 主菜单容器：单行排列，窗口宽度不足时允许被裁剪（保持原有行为） */
.menu-bar {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  min-width: fit-content;
  width: 100%;
  padding: 7px 10px;
  background-color: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  z-index: 100;
}

/* 功能分组，竖线分隔 */
.menu-cluster {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 14px;
  flex-shrink: 0;
  white-space: nowrap;
}

.menu-cluster + .menu-cluster {
  border-left: 1px solid var(--border);
}

/* 第一组贴近窗口左缘，减少整体左缩进 */
.menu-cluster:first-child {
  padding-left: 4px;
}

.menu-group {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  flex-shrink: 0;
}

.menu-label {
  font-size: 12px;
  color: var(--text-3);
  white-space: nowrap;
}

/* ===== 通用控件 ===== */
.control-input,
.control-select,
.control-btn {
  height: var(--control-height);
  padding: 0 8px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background-color: var(--bg-panel);
  color: var(--text-1);
  font-size: 12px;
  font-family: inherit;
  transition: border-color 0.15s ease, background-color 0.15s ease,
    box-shadow 0.15s ease;
}

.control-input {
  width: 130px;
  color: var(--text-1);
}

.control-input::placeholder {
  color: var(--text-3);
}

.control-input--port {
  width: 90px;
}

.control-input:focus,
.control-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-weak);
}

.control-input:hover,
.control-select:hover:not(:disabled),
.control-btn:hover {
  border-color: var(--accent);
}

.control-select {
  cursor: pointer;
}

.control-select:disabled {
  background-color: var(--bg-subtle);
  color: var(--text-3);
  cursor: not-allowed;
}

.control-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  cursor: pointer;
  font-weight: 500;
  white-space: nowrap;
}

.control-btn:active {
  transform: translateY(0.5px);
}

.btn-icon {
  flex-shrink: 0;
}

.btn-icon.spinning {
  animation: pv-spin 1.2s linear infinite;
}

/* ===== 协议分段按钮 ===== */
.segmented {
  display: flex;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  overflow: hidden;
  height: var(--control-height);
}

.segment-btn {
  padding: 0 12px;
  border: none;
  background-color: var(--bg-panel);
  color: var(--text-2);
  font-size: 12px;
  font-family: inherit;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.segment-btn + .segment-btn {
  border-left: 1px solid var(--border);
}

.segment-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-1);
}

.segment-btn.active {
  background-color: var(--accent);
  color: #ffffff;
  font-weight: 600;
}

/* ===== 自动刷新激活态 ===== */
.refresh-toggle-btn.active {
  background-color: var(--success-weak);
  border-color: var(--success);
  color: var(--success);
  font-weight: 600;
}

/* ===== 图标按钮（主题切换） ===== */
.icon-btn {
  height: var(--control-height);
  width: var(--control-height);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background-color: var(--bg-panel);
  color: var(--text-2);
  cursor: pointer;
  transition: border-color 0.15s ease, color 0.15s ease,
    background-color 0.15s ease;
}

.icon-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background-color: var(--accent-weak);
}

/* ===== 下拉宽度微调 ===== */
.state-select {
  min-width: 96px;
}

.lang-select {
  min-width: 84px;
}

.refresh-interval-select {
  min-width: 68px;
}
</style>
