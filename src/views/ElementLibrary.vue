<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useElementStore, type ElementLibraryInfo } from '@/stores/element';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Aim, Delete, View, FolderOpened, Plus } from '@element-plus/icons-vue';
import ElementPicker from '@/components/element/ElementPicker.vue';
import type { UIElement } from '@/types';

const elementStore = useElementStore();
const showCreateDialog = ref(false);
const showPickerDialog = ref(false);
const showElementDetail = ref(false);
const newLibraryName = ref('');

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
    ElMessage.success('元素库创建成功');
  }
}

async function openLibrary(info: ElementLibraryInfo) {
  await elementStore.loadLibrary(info.id);
}

async function deleteLibraryConfirm(info: ElementLibraryInfo) {
  try {
    await ElMessageBox.confirm(
      `确定要删除元素库 "${info.name}" 吗？此操作不可恢复。`,
      '确认删除',
      { type: 'warning' }
    );
    await elementStore.deleteLibrary(info.id);
    ElMessage.success('元素库已删除');
  } catch {
    // Cancelled
  }
}

function startCapture() {
  if (!elementStore.currentLibrary) {
    ElMessage.warning('请先选择或创建一个元素库');
    return;
  }
  showPickerDialog.value = true;
}

async function handleElementCaptured(element: UIElement) {
  elementStore.addElement(element);
  await elementStore.saveLibrary();
  ElMessage.success('元素已添加到库');
}

function viewElement(element: UIElement) {
  elementStore.selectElement(element);
  showElementDetail.value = true;
}

async function removeElement(id: string) {
  try {
    await ElMessageBox.confirm(
      '确定要删除此元素吗？',
      '确认删除',
      { type: 'warning' }
    );
    elementStore.removeElement(id);
    await elementStore.saveLibrary();
    ElMessage.success('元素已删除');
  } catch {
    // Cancelled
  }
}

async function highlightElement(element: UIElement) {
  await elementStore.highlightElement(element);
}
</script>

<template>
  <div class="element-library">
    <!-- Left sidebar: Library list -->
    <div class="library-sidebar">
      <div class="sidebar-header">
        <h3>元素库列表</h3>
        <el-button type="primary" size="small" :icon="Plus" @click="showCreateDialog = true">
          新建
        </el-button>
      </div>

      <el-scrollbar class="library-list">
        <div
          v-for="lib in elementStore.libraryList"
          :key="lib.id"
          class="library-item"
          :class="{ active: elementStore.currentLibrary?.id === lib.id }"
          @click="openLibrary(lib)"
        >
          <div class="library-item-content">
            <el-icon><FolderOpened /></el-icon>
            <div class="library-item-info">
              <span class="library-name">{{ lib.name }}</span>
              <span class="library-count">{{ lib.element_count }} 个元素</span>
            </div>
          </div>
          <el-button
            class="library-delete-btn"
            text
            type="danger"
            size="small"
            :icon="Delete"
            @click.stop="deleteLibraryConfirm(lib)"
          />
        </div>

        <el-empty v-if="elementStore.libraryList.length === 0" description="暂无元素库" />
      </el-scrollbar>
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
          <el-button
            type="primary"
            :icon="Aim"
            @click="startCapture"
            :disabled="!elementStore.currentLibrary"
          >
            捕获元素
          </el-button>
        </div>
      </div>

      <div class="main-content">
        <el-empty
          v-if="!elementStore.currentLibrary"
          description="请从左侧选择一个元素库，或创建新的元素库"
        >
          <el-button type="primary" @click="showCreateDialog = true">创建元素库</el-button>
        </el-empty>

        <el-empty
          v-else-if="elementStore.currentLibrary.elements.length === 0"
          description="此元素库暂无元素，点击上方按钮开始捕获"
        />

        <el-table
          v-else
          :data="elementStore.currentLibrary.elements"
          style="width: 100%"
          stripe
        >
          <el-table-column prop="name" label="元素名称" min-width="150">
            <template #default="{ row }">
              <div class="element-name-cell">
                <span>{{ row.name }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column prop="controlType" label="控件类型" width="120">
            <template #default="{ row }">
              <el-tag size="small" type="info">{{ row.controlType }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="processName" label="进程" width="120" />
          <el-table-column prop="windowTitle" label="窗口标题" min-width="150" show-overflow-tooltip />
          <el-table-column label="位置" width="180">
            <template #default="{ row }">
              <span class="position-text">
                ({{ row.bounds.x }}, {{ row.bounds.y }})
                {{ row.bounds.width }}x{{ row.bounds.height }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="150" fixed="right">
            <template #default="{ row }">
              <el-button-group size="small">
                <el-tooltip content="查看详情">
                  <el-button :icon="View" @click="viewElement(row)" />
                </el-tooltip>
                <el-tooltip content="高亮显示">
                  <el-button :icon="Aim" @click="highlightElement(row)" />
                </el-tooltip>
                <el-tooltip content="删除">
                  <el-button :icon="Delete" type="danger" @click="removeElement(row.id)" />
                </el-tooltip>
              </el-button-group>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- Create library dialog -->
    <el-dialog v-model="showCreateDialog" title="新建元素库" width="400px">
      <el-form @submit.prevent="createLibrary">
        <el-form-item label="库名称">
          <el-input
            v-model="newLibraryName"
            placeholder="请输入元素库名称"
            autofocus
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="createLibrary" :disabled="!newLibraryName.trim()">
          创建
        </el-button>
      </template>
    </el-dialog>

    <!-- Element picker dialog -->
    <ElementPicker
      v-model="showPickerDialog"
      @captured="handleElementCaptured"
    />

    <!-- Element detail dialog -->
    <el-dialog
      v-model="showElementDetail"
      :title="elementStore.selectedElement?.name || '元素详情'"
      width="600px"
    >
      <template v-if="elementStore.selectedElement">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="ID">
            {{ elementStore.selectedElement.id }}
          </el-descriptions-item>
          <el-descriptions-item label="名称">
            {{ elementStore.selectedElement.name }}
          </el-descriptions-item>
          <el-descriptions-item label="控件类型">
            {{ elementStore.selectedElement.controlType }}
          </el-descriptions-item>
          <el-descriptions-item label="自动化ID">
            {{ elementStore.selectedElement.automationId || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="类名">
            {{ elementStore.selectedElement.className || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="进程">
            {{ elementStore.selectedElement.processName || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="窗口标题" :span="2">
            {{ elementStore.selectedElement.windowTitle || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="XPath" :span="2">
            {{ elementStore.selectedElement.xpath || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="CSS选择器" :span="2">
            {{ elementStore.selectedElement.cssSelector || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="位置">
            ({{ elementStore.selectedElement.bounds.x }}, {{ elementStore.selectedElement.bounds.y }})
          </el-descriptions-item>
          <el-descriptions-item label="大小">
            {{ elementStore.selectedElement.bounds.width }} x {{ elementStore.selectedElement.bounds.height }}
          </el-descriptions-item>
          <el-descriptions-item label="创建时间" :span="2">
            {{ elementStore.selectedElement.createdAt }}
          </el-descriptions-item>
        </el-descriptions>

        <!-- Attributes -->
        <div v-if="Object.keys(elementStore.selectedElement.attributes || {}).length > 0" class="attributes-section">
          <h4>扩展属性</h4>
          <el-descriptions :column="2" border size="small">
            <el-descriptions-item
              v-for="(value, key) in elementStore.selectedElement.attributes"
              :key="key"
              :label="String(key)"
            >
              {{ value }}
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.element-library {
  height: 100%;
  display: flex;
  background: var(--el-bg-color-page);
}

.library-sidebar {
  width: 280px;
  background: var(--el-bg-color);
  border-right: 1px solid var(--el-border-color-light);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.sidebar-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
}

.library-list {
  flex: 1;
  padding: 8px;
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
  background: var(--el-fill-color-light);
}

.library-item.active {
  background: var(--el-color-primary-light-9);
  border: 1px solid var(--el-color-primary-light-5);
}

.library-item-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.library-item-content .el-icon {
  font-size: 20px;
  color: var(--el-color-primary);
}

.library-item-info {
  display: flex;
  flex-direction: column;
}

.library-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.library-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.library-delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.library-item:hover .library-delete-btn {
  opacity: 1;
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
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-light);
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

.element-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.position-text {
  font-family: monospace;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.attributes-section {
  margin-top: 16px;
}

.attributes-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 500;
}
</style>
