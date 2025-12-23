<script setup lang="ts">
import { ref } from 'vue';

const settings = ref({
  executionDelay: 500,
  retryCount: 3,
  retryDelay: 1000,
  screenshotOnError: true,
  logLevel: 'info',
});

function saveSettings() {
  // TODO: Save to local storage or backend
  console.log('Settings saved:', settings.value);
}
</script>

<template>
  <div class="settings-container">
    <h3>设置</h3>

    <el-form :model="settings" label-width="140px" style="max-width: 500px">
      <el-divider content-position="left">执行设置</el-divider>

      <el-form-item label="操作间隔(ms)">
        <el-input-number v-model="settings.executionDelay" :min="0" :max="10000" :step="100" />
      </el-form-item>

      <el-form-item label="重试次数">
        <el-input-number v-model="settings.retryCount" :min="0" :max="10" />
      </el-form-item>

      <el-form-item label="重试间隔(ms)">
        <el-input-number v-model="settings.retryDelay" :min="0" :max="10000" :step="100" />
      </el-form-item>

      <el-divider content-position="left">日志设置</el-divider>

      <el-form-item label="日志级别">
        <el-select v-model="settings.logLevel">
          <el-option label="Debug" value="debug" />
          <el-option label="Info" value="info" />
          <el-option label="Warn" value="warn" />
          <el-option label="Error" value="error" />
        </el-select>
      </el-form-item>

      <el-form-item label="错误时截图">
        <el-switch v-model="settings.screenshotOnError" />
      </el-form-item>

      <el-form-item>
        <el-button type="primary" @click="saveSettings">保存设置</el-button>
      </el-form-item>
    </el-form>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 16px;
}

.settings-container h3 {
  margin-bottom: 24px;
}
</style>
