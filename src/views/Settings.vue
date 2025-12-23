<script setup lang="ts">
import { ref } from 'vue';

const settings = ref({
  executionDelay: 500,
  retryCount: 3,
  retryDelay: 1000,
  screenshotOnError: true,
  logLevel: 'info',
});

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' }[]>([]);
let toastId = 0;

function showToast(message: string, type: 'success' | 'error' = 'success') {
  const id = ++toastId;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }, 3000);
}

function saveSettings() {
  // TODO: Save to local storage or backend
  console.log('Settings saved:', settings.value);
  showToast('设置已保存', 'success');
}
</script>

<template>
  <div class="settings-container">
    <!-- Toast notifications -->
    <div class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="[`toast-${toast.type}`]"
      >
        {{ toast.message }}
      </div>
    </div>

    <h3>设置</h3>

    <div class="settings-form">
      <div class="form-section">
        <div class="section-title">执行设置</div>

        <div class="form-item">
          <label class="form-label">操作间隔(ms)</label>
          <input
            v-model.number="settings.executionDelay"
            type="number"
            class="input input-number"
            :min="0"
            :max="10000"
            step="100"
          />
        </div>

        <div class="form-item">
          <label class="form-label">重试次数</label>
          <input
            v-model.number="settings.retryCount"
            type="number"
            class="input input-number"
            :min="0"
            :max="10"
          />
        </div>

        <div class="form-item">
          <label class="form-label">重试间隔(ms)</label>
          <input
            v-model.number="settings.retryDelay"
            type="number"
            class="input input-number"
            :min="0"
            :max="10000"
            step="100"
          />
        </div>
      </div>

      <div class="form-section">
        <div class="section-title">日志设置</div>

        <div class="form-item">
          <label class="form-label">日志级别</label>
          <select v-model="settings.logLevel" class="select">
            <option value="debug">Debug</option>
            <option value="info">Info</option>
            <option value="warn">Warn</option>
            <option value="error">Error</option>
          </select>
        </div>

        <div class="form-item">
          <label class="form-label">错误时截图</label>
          <label class="switch">
            <input type="checkbox" v-model="settings.screenshotOnError" />
            <span class="slider"></span>
          </label>
        </div>
      </div>

      <div class="form-item">
        <button class="btn btn-primary" @click="saveSettings">保存设置</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 16px 24px;
  max-width: 600px;
  position: relative;
}

.settings-container h3 {
  margin-bottom: 24px;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-section {
  padding-bottom: 16px;
}

.section-title {
  font-size: 14px;
  font-weight: 500;
  color: #6b7280;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.form-item {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
}

.form-label {
  width: 140px;
  flex-shrink: 0;
  font-size: 14px;
  color: #374151;
}

.input-number {
  width: 160px;
}

.switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #d1d5db;
  transition: 0.3s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.switch input:checked + .slider {
  background-color: #3b82f6;
}

.switch input:checked + .slider:before {
  transform: translateX(20px);
}

.toast-container {
  position: fixed;
  top: 80px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast {
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 14px;
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  animation: slideIn 0.3s ease;
}

.toast-success {
  background: #22c55e;
}

.toast-error {
  background: #ef4444;
}

@keyframes slideIn {
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
