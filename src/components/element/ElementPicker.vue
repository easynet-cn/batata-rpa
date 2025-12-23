<template>
  <el-dialog
    v-model="visible"
    title="元素捕获"
    width="600px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="element-picker">
      <!-- Capture mode instructions -->
      <div v-if="!capturedElement" class="capture-instructions">
        <div class="instruction-icon">
          <el-icon :size="48"><Aim /></el-icon>
        </div>
        <h3>移动鼠标到目标元素</h3>
        <p>将鼠标移动到您想要捕获的UI元素上，然后按下 <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>C</kbd> 进行捕获</p>
        <p class="hint">或者点击下方按钮手动输入坐标捕获</p>

        <el-divider>手动捕获</el-divider>

        <div class="manual-capture">
          <el-form :inline="true" size="small">
            <el-form-item label="X坐标">
              <el-input-number v-model="manualX" :min="0" :max="9999" />
            </el-form-item>
            <el-form-item label="Y坐标">
              <el-input-number v-model="manualY" :min="0" :max="9999" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="captureAtPosition" :loading="isCapturing">
                捕获
              </el-button>
            </el-form-item>
          </el-form>
        </div>

        <div class="mouse-position" v-if="showMousePosition">
          <span>当前鼠标位置: ({{ mouseX }}, {{ mouseY }})</span>
        </div>
      </div>

      <!-- Captured element preview -->
      <div v-else class="capture-result">
        <div class="element-preview">
          <div class="preview-header">
            <el-tag type="success">捕获成功</el-tag>
            <el-button text size="small" @click="recapture">
              <el-icon><RefreshRight /></el-icon>
              重新捕获
            </el-button>
          </div>

          <el-descriptions :column="2" border size="small">
            <el-descriptions-item label="元素名称">
              {{ capturedElement.name }}
            </el-descriptions-item>
            <el-descriptions-item label="控件类型">
              {{ capturedElement.controlType }}
            </el-descriptions-item>
            <el-descriptions-item label="自动化ID">
              {{ capturedElement.automationId || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="类名">
              {{ capturedElement.className || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="进程">
              {{ capturedElement.processName || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="窗口标题">
              {{ capturedElement.windowTitle || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="位置" :span="2">
              ({{ capturedElement.bounds.x }}, {{ capturedElement.bounds.y }}) -
              {{ capturedElement.bounds.width }} x {{ capturedElement.bounds.height }}
            </el-descriptions-item>
          </el-descriptions>

          <!-- Element name input -->
          <div class="element-name-input">
            <el-input
              v-model="elementName"
              placeholder="为元素命名（可选）"
              clearable
            >
              <template #prepend>元素名称</template>
            </el-input>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button
        v-if="capturedElement"
        type="primary"
        @click="confirmCapture"
      >
        添加到元素库
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { Aim, RefreshRight } from '@element-plus/icons-vue';
import type { UIElement } from '@/types';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'captured', element: UIElement): void;
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const capturedElement = ref<UIElement | null>(null);
const elementName = ref('');
const isCapturing = ref(false);
const manualX = ref(0);
const manualY = ref(0);
const mouseX = ref(0);
const mouseY = ref(0);
const showMousePosition = ref(false);

let mouseInterval: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  // Start tracking mouse position
  showMousePosition.value = true;
  mouseInterval = setInterval(updateMousePosition, 100);

  // Listen for capture hotkey
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  if (mouseInterval) {
    clearInterval(mouseInterval);
  }
  document.removeEventListener('keydown', handleKeydown);
});

function updateMousePosition() {
  // Note: This only works within the app window
  // For global mouse position, we'd need a Tauri command
}

async function handleKeydown(e: KeyboardEvent) {
  // Ctrl + Shift + C to capture
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'c') {
    e.preventDefault();
    await captureAtCurrentPosition();
  }
}

async function captureAtCurrentPosition() {
  // Use manual coordinates for now
  await captureAtPosition();
}

async function captureAtPosition() {
  if (isCapturing.value) return;

  isCapturing.value = true;
  try {
    const element = await invoke<UIElement>('capture_element', {
      x: manualX.value,
      y: manualY.value,
    });

    capturedElement.value = element;
    elementName.value = element.name || '';

    ElMessage.success('元素捕获成功');
  } catch (error) {
    ElMessage.error(`捕获失败: ${error}`);
  } finally {
    isCapturing.value = false;
  }
}

function recapture() {
  capturedElement.value = null;
  elementName.value = '';
}

function confirmCapture() {
  if (capturedElement.value) {
    const element = {
      ...capturedElement.value,
      name: elementName.value || capturedElement.value.name,
    };
    emit('captured', element);
    handleClose();
  }
}

function handleClose() {
  capturedElement.value = null;
  elementName.value = '';
  visible.value = false;
}
</script>

<style scoped>
.element-picker {
  min-height: 300px;
}

.capture-instructions {
  text-align: center;
  padding: 20px;
}

.instruction-icon {
  color: var(--el-color-primary);
  margin-bottom: 16px;
}

.capture-instructions h3 {
  margin: 0 0 12px 0;
  color: var(--el-text-color-primary);
}

.capture-instructions p {
  margin: 8px 0;
  color: var(--el-text-color-secondary);
}

.capture-instructions .hint {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

.capture-instructions kbd {
  padding: 2px 6px;
  background: var(--el-fill-color);
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  font-family: monospace;
  font-size: 12px;
}

.manual-capture {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}

.mouse-position {
  margin-top: 16px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
}

.capture-result {
  padding: 16px 0;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.element-name-input {
  margin-top: 16px;
}
</style>
