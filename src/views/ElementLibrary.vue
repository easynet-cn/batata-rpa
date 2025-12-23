<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useElementStore, type ElementLibraryInfo } from '@/stores/element';
import { Crosshair, Trash2, Eye, FolderOpen, Plus, X, FileText } from 'lucide-vue-next';
import ElementPicker from '@/components/element/ElementPicker.vue';
import type { UIElement } from '@/types';

const elementStore = useElementStore();
const showCreateDialog = ref(false);
const showPickerDialog = ref(false);
const showElementDetail = ref(false);
const showConfirmDialog = ref(false);
const confirmDialogData = ref<{ title: string; message: string; onConfirm: () => void } | null>(null);
const newLibraryName = ref('');

// Toast system
const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'warning' }[]>([]);
let toastId = 0;

function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
  const id = ++toastId;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }, 3000);
}

function showConfirm(title: string, message: string, onConfirm: () => void) {
  confirmDialogData.value = { title, message, onConfirm };
  showConfirmDialog.value = true;
}

function handleConfirm() {
  if (confirmDialogData.value) {
    confirmDialogData.value.onConfirm();
  }
  showConfirmDialog.value = false;
  confirmDialogData.value = null;
}

onMounted(async () => {
  await elementStore.fetchLibraries();
});

async function createLibrary() {
  if (newLibraryName.value.trim()) {
    const library = elementStore.createLibrary(newLibraryName.value.trim());
    await elementStore.saveLibrary(library);
    await elementStore.fetchLibraries();
    newLibraryName.value = '';
    showCreateDialog.value = false;
    showToast('元素库创建成功', 'success');
  }
}

async function openLibrary(info: ElementLibraryInfo) {
  await elementStore.loadLibrary(info.id);
}

async function deleteLibraryConfirm(info: ElementLibraryInfo) {
  showConfirm(
    '确认删除',
    `确定要删除元素库 "${info.name}" 吗？此操作不可恢复。`,
    async () => {
      await elementStore.deleteLibrary(info.id);
      showToast('元素库已删除', 'success');
    }
  );
}

function startCapture() {
  if (!elementStore.currentLibrary) {
    showToast('请先选择或创建一个元素库', 'warning');
    return;
  }
  showPickerDialog.value = true;
}

async function handleElementCaptured(element: UIElement) {
  elementStore.addElement(element);
  await elementStore.saveLibrary();
  showToast('元素已添加到库', 'success');
}

function viewElement(element: UIElement) {
  elementStore.selectElement(element);
  showElementDetail.value = true;
}

async function removeElement(id: string) {
  showConfirm(
    '确认删除',
    '确定要删除此元素吗？',
    async () => {
      elementStore.removeElement(id);
      await elementStore.saveLibrary();
      showToast('元素已删除', 'success');
    }
  );
}

async function highlightElement(element: UIElement) {
  await elementStore.highlightElement(element);
}
</script>

<template>
  <div class="element-library">
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

    <!-- Left sidebar: Library list -->
    <div class="library-sidebar">
      <div class="sidebar-header">
        <h3>元素库列表</h3>
        <button class="btn btn-primary btn-small" @click="showCreateDialog = true">
          <Plus :size="14" />
          新建
        </button>
      </div>

      <div class="library-list">
        <div
          v-for="lib in elementStore.libraryList"
          :key="lib.id"
          class="library-item"
          :class="{ active: elementStore.currentLibrary?.id === lib.id }"
          @click="openLibrary(lib)"
        >
          <div class="library-item-content">
            <FolderOpen :size="20" class="library-icon" />
            <div class="library-item-info">
              <span class="library-name">{{ lib.name }}</span>
              <span class="library-count">{{ lib.element_count }} 个元素</span>
            </div>
          </div>
          <button
            class="library-delete-btn"
            @click.stop="deleteLibraryConfirm(lib)"
          >
            <Trash2 :size="14" />
          </button>
        </div>

        <div v-if="elementStore.libraryList.length === 0" class="empty-state">
          <FileText :size="40" class="empty-icon" />
          <span>暂无元素库</span>
        </div>
      </div>
    </div>

    <!-- Main content: Element list -->
    <div class="library-main">
      <div class="main-header">
        <div class="header-left">
          <h3 v-if="elementStore.currentLibrary">
            {{ elementStore.currentLibrary.name }}
          </h3>
          <h3 v-else>选择一个元素库</h3>
        </div>
        <div class="header-right">
          <button
            class="btn btn-primary"
            @click="startCapture"
            :disabled="!elementStore.currentLibrary"
          >
            <Crosshair :size="14" />
            捕获元素
          </button>
        </div>
      </div>

      <div class="main-content">
        <div
          v-if="!elementStore.currentLibrary"
          class="empty-state-large"
        >
          <FileText :size="64" class="empty-icon" />
          <p>请从左侧选择一个元素库，或创建新的元素库</p>
          <button class="btn btn-primary" @click="showCreateDialog = true">创建元素库</button>
        </div>

        <div
          v-else-if="elementStore.currentLibrary.elements.length === 0"
          class="empty-state-large"
        >
          <FileText :size="64" class="empty-icon" />
          <p>此元素库暂无元素，点击上方按钮开始捕获</p>
        </div>

        <table v-else class="data-table">
          <thead>
            <tr>
              <th>元素名称</th>
              <th>控件类型</th>
              <th>进程</th>
              <th>窗口标题</th>
              <th>位置</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="element in elementStore.currentLibrary.elements" :key="element.id">
              <td>
                <span class="element-name">{{ element.name }}</span>
              </td>
              <td>
                <span class="tag tag-info">{{ element.controlType }}</span>
              </td>
              <td>{{ element.processName }}</td>
              <td class="cell-ellipsis" :title="element.windowTitle">{{ element.windowTitle }}</td>
              <td>
                <span class="position-text">
                  ({{ element.bounds.x }}, {{ element.bounds.y }})
                  {{ element.bounds.width }}x{{ element.bounds.height }}
                </span>
              </td>
              <td>
                <div class="action-buttons">
                  <button class="btn-icon" title="查看详情" @click="viewElement(element)">
                    <Eye :size="14" />
                  </button>
                  <button class="btn-icon" title="高亮显示" @click="highlightElement(element)">
                    <Crosshair :size="14" />
                  </button>
                  <button class="btn-icon btn-icon-danger" title="删除" @click="removeElement(element.id)">
                    <Trash2 :size="14" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Create library dialog -->
    <div v-if="showCreateDialog" class="dialog-overlay" @click.self="showCreateDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <span>新建元素库</span>
          <button class="btn-icon" @click="showCreateDialog = false">
            <X :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <div class="form-item">
            <label class="form-label">库名称</label>
            <input
              v-model="newLibraryName"
              class="input"
              placeholder="请输入元素库名称"
              @keyup.enter="createLibrary"
            />
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="showCreateDialog = false">取消</button>
          <button class="btn btn-primary" @click="createLibrary" :disabled="!newLibraryName.trim()">
            创建
          </button>
        </div>
      </div>
    </div>

    <!-- Confirm dialog -->
    <div v-if="showConfirmDialog" class="dialog-overlay" @click.self="showConfirmDialog = false">
      <div class="dialog dialog-small">
        <div class="dialog-header">
          <span>{{ confirmDialogData?.title }}</span>
          <button class="btn-icon" @click="showConfirmDialog = false">
            <X :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <p>{{ confirmDialogData?.message }}</p>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="showConfirmDialog = false">取消</button>
          <button class="btn btn-danger" @click="handleConfirm">确认</button>
        </div>
      </div>
    </div>

    <!-- Element picker dialog -->
    <ElementPicker
      v-model="showPickerDialog"
      @captured="handleElementCaptured"
    />

    <!-- Element detail dialog -->
    <div v-if="showElementDetail" class="dialog-overlay" @click.self="showElementDetail = false">
      <div class="dialog dialog-large">
        <div class="dialog-header">
          <span>{{ elementStore.selectedElement?.name || '元素详情' }}</span>
          <button class="btn-icon" @click="showElementDetail = false">
            <X :size="18" />
          </button>
        </div>
        <div v-if="elementStore.selectedElement" class="dialog-body">
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">ID</span>
              <span class="detail-value">{{ elementStore.selectedElement.id }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">名称</span>
              <span class="detail-value">{{ elementStore.selectedElement.name }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">控件类型</span>
              <span class="detail-value">{{ elementStore.selectedElement.controlType }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">自动化ID</span>
              <span class="detail-value">{{ elementStore.selectedElement.automationId || '-' }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">类名</span>
              <span class="detail-value">{{ elementStore.selectedElement.className || '-' }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">进程</span>
              <span class="detail-value">{{ elementStore.selectedElement.processName || '-' }}</span>
            </div>
            <div class="detail-item detail-item-full">
              <span class="detail-label">窗口标题</span>
              <span class="detail-value">{{ elementStore.selectedElement.windowTitle || '-' }}</span>
            </div>
            <div class="detail-item detail-item-full">
              <span class="detail-label">XPath</span>
              <span class="detail-value detail-value-code">{{ elementStore.selectedElement.xpath || '-' }}</span>
            </div>
            <div class="detail-item detail-item-full">
              <span class="detail-label">CSS选择器</span>
              <span class="detail-value detail-value-code">{{ elementStore.selectedElement.cssSelector || '-' }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">位置</span>
              <span class="detail-value">({{ elementStore.selectedElement.bounds.x }}, {{ elementStore.selectedElement.bounds.y }})</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">大小</span>
              <span class="detail-value">{{ elementStore.selectedElement.bounds.width }} x {{ elementStore.selectedElement.bounds.height }}</span>
            </div>
            <div class="detail-item detail-item-full">
              <span class="detail-label">创建时间</span>
              <span class="detail-value">{{ elementStore.selectedElement.createdAt }}</span>
            </div>
          </div>

          <!-- Attributes -->
          <div v-if="Object.keys(elementStore.selectedElement.attributes || {}).length > 0" class="attributes-section">
            <h4>扩展属性</h4>
            <div class="detail-grid">
              <div
                v-for="(value, key) in elementStore.selectedElement.attributes"
                :key="key"
                class="detail-item"
              >
                <span class="detail-label">{{ key }}</span>
                <span class="detail-value">{{ value }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.element-library {
  height: 100%;
  display: flex;
  background: #f9fafb;
  position: relative;
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

.toast-warning {
  background: #f59e0b;
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

.library-sidebar {
  width: 280px;
  background: #fff;
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
}

.btn-small {
  padding: 6px 12px;
  font-size: 13px;
}

.library-list {
  flex: 1;
  padding: 8px;
  overflow-y: auto;
}

.library-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.library-item:hover {
  background: #f3f4f6;
}

.library-item.active {
  background: #dbeafe;
  border: 1px solid #93c5fd;
}

.library-item-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.library-icon {
  color: #3b82f6;
}

.library-item-info {
  display: flex;
  flex-direction: column;
}

.library-name {
  font-size: 14px;
  font-weight: 500;
  color: #1f2937;
}

.library-count {
  font-size: 12px;
  color: #6b7280;
}

.library-delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
  background: transparent;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: #ef4444;
  border-radius: 4px;
}

.library-delete-btn:hover {
  background: #fef2f2;
}

.library-item:hover .library-delete-btn {
  opacity: 1;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 20px;
  color: #9ca3af;
}

.empty-state-large {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 60px;
  color: #9ca3af;
}

.empty-state-large p {
  margin: 0;
  font-size: 14px;
}

.empty-icon {
  color: #d1d5db;
}

.library-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: #fff;
  border-bottom: 1px solid var(--border-color);
}

.main-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.main-content {
  flex: 1;
  padding: 16px 24px;
  overflow: auto;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  background: #fff;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.data-table th,
.data-table td {
  padding: 12px 16px;
  text-align: left;
  border-bottom: 1px solid #f3f4f6;
}

.data-table th {
  background: #f9fafb;
  font-weight: 500;
  font-size: 13px;
  color: #6b7280;
}

.data-table td {
  font-size: 14px;
  color: #1f2937;
}

.data-table tr:hover td {
  background: #f9fafb;
}

.cell-ellipsis {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.element-name {
  font-weight: 500;
}

.tag {
  display: inline-flex;
  padding: 2px 8px;
  font-size: 12px;
  border-radius: 4px;
}

.tag-info {
  background: #e0f2fe;
  color: #0369a1;
}

.position-text {
  font-family: monospace;
  font-size: 12px;
  color: #6b7280;
}

.action-buttons {
  display: flex;
  gap: 4px;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: #6b7280;
}

.btn-icon:hover {
  background: #f3f4f6;
  color: #374151;
}

.btn-icon-danger:hover {
  background: #fef2f2;
  color: #ef4444;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  width: 400px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.dialog-small {
  width: 360px;
}

.dialog-large {
  width: 600px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
  font-size: 16px;
}

.dialog-body {
  padding: 20px;
  overflow-y: auto;
}

.dialog-body p {
  margin: 0;
  color: #374151;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}

.btn-danger {
  background: #ef4444;
  color: #fff;
}

.btn-danger:hover {
  background: #dc2626;
}

.form-item {
  margin-bottom: 16px;
}

.form-item:last-child {
  margin-bottom: 0;
}

.form-label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 500;
  color: #374151;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-item-full {
  grid-column: span 2;
}

.detail-label {
  font-size: 12px;
  color: #6b7280;
}

.detail-value {
  font-size: 14px;
  color: #1f2937;
  word-break: break-all;
}

.detail-value-code {
  font-family: monospace;
  font-size: 12px;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 4px;
}

.attributes-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}

.attributes-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 500;
}
</style>
