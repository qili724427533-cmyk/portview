<template>
  <div class="about-modal-overlay" v-if="showAbout">
    <div class="about-modal">
      <div class="about-header">
        <h2>{{ t("about.title") }}</h2>
        <button class="close-btn" @click="closeAbout">&times;</button>
      </div>
      <div class="about-content">
        <div class="app-logo-container">
          <img src="/icon.svg" alt="PortView Logo" class="app-logo-large" />
        </div>
        <div class="app-info">
          <p><strong>{{ t("about.appName") }}:</strong> {{ t("appTitle") }}</p>
          <p><strong>{{ t("about.version") }}:</strong> {{ appVersion }}</p>
          <p><strong>{{ t("about.description") }}:</strong> {{ t("about.descriptionText") }}</p>
          <p><strong>{{ t("about.author") }}:</strong> vcyang</p>
          <p><strong>{{ t("about.github") }}:</strong> 
            <a href="#" @click.prevent="openGitHub">{{ t("about.githubLink") }}</a>
          </p>
        </div>
      </div>
      <div class="about-footer">
        <button class="ok-btn" @click="closeAbout">{{ t("about.okButton") }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { openUrl } from '@tauri-apps/plugin-opener';

interface Props {
  showAbout: boolean;
  appVersion: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: "update:showAbout", value: boolean): void;
}

const emit = defineEmits<Emits>();

const { t } = useI18n();

const closeAbout = () => {
  emit('update:showAbout', false);
};

const openGitHub = async () => {
  // 使用系统默认浏览器打开链接
  await openUrl('https://github.com/vcyang/portview');
};
</script>

<style scoped>
.about-modal-overlay {
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

.about-modal {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  position: relative;
}

.about-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
  background-color: #f8fafc;
  border-radius: 8px 8px 0 0;
}

.about-header h2 {
  margin: 0;
  font-size: 1.25rem;
  color: #1e293b;
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: #ef4444; /* 红色 */
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.close-btn:hover {
  background-color: #fee2e2; /* 红色背景 */
  color: #dc2626; /* 深红色 */
}

.about-content {
  padding: 20px;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.app-logo-container {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 10px;
}

.app-logo-large {
  width: 96px;
  height: 96px;
  object-fit: contain;
}

.app-info p {
  margin: 0 0 10px 0;
  color: #334155;
  line-height: 1.5;
}

.about-content a {
  color: #3b82f6;
  text-decoration: none;
}

.about-content a:hover {
  text-decoration: underline;
}

.about-footer {
  padding: 16px 20px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: flex-end;
  background-color: #f8fafc;
  border-radius: 0 0 8px 8px;
}

.ok-btn {
  padding: 6px 16px;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.875rem;
}

.ok-btn:hover {
  background-color: #2563eb;
}

/* 暗色主题样式 */
.dark .about-modal {
  background-color: #1e293b;
  color: #e2e8f0;
}

.dark .about-header {
  border-bottom: 1px solid #334155;
  background-color: #334155;
}

.dark .about-header h2 {
  color: #f8fafc;
}

.dark .about-content {
  color: #cbd5e0;
}

.dark .about-content p {
  color: #e2e8f0; /* 确保段落文字在暗色主题下可见 */
}

.dark .about-content a {
  color: #60a5fa;
}

.dark .about-content a:hover {
  color: #93c5fd;
}

.dark .about-footer {
  border-top: 1px solid #334155;
  background-color: #334155;
}

.dark .ok-btn {
  background-color: #3b82f6;
}

.dark .ok-btn:hover {
  background-color: #2563eb;
}

.dark .close-btn {
  color: #ef4444; /* 红色 */
}

.dark .close-btn:hover {
  background-color: #475569;
  color: #dc2626; /* 深红色 */
}
</style>