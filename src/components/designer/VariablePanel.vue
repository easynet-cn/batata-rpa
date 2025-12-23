<script setup lang="ts">
import { ref, computed } from 'vue';
import { useWorkflowStore } from '@/stores';
import { Plus, Trash2, Pencil, X, Package } from 'lucide-vue-next';
import type { Variable } from '@/types';

const workflowStore = useWorkflowStore();

const variables = computed(() => workflowStore.currentWorkflow?.variables || []);

const showAddDialog = ref(false);
const showEditDialog = ref(false);
const editingVariable = ref<Variable | null>(null);

const newVariable = ref<Omit<Variable, 'id'>>({
  name: '',
  type: 'string',
  value: '',
  scope: 'global',
});

function addVariable() {
  if (!newVariable.value.name) return;

  const variable: Variable = {
    id: `var-${Date.now()}`,
    ...newVariable.value,
  };

  workflowStore.addVariable(variable);
  showAddDialog.value = false;
  resetNewVariable();
}

function resetNewVariable() {
  newVariable.value = {
    name: '',
    type: 'string',
    value: '',
    scope: 'global',
  };
}

function startEdit(variable: Variable) {
  editingVariable.value = { ...variable };
  showEditDialog.value = true;
}

function saveEdit() {
  if (editingVariable.value) {
    workflowStore.updateVariable(editingVariable.value.id, editingVariable.value);
    showEditDialog.value = false;
    editingVariable.value = null;
  }
}

function deleteVariable(id: string) {
  workflowStore.removeVariable(id);
}

function getValueDisplay(variable: Variable): string {
  if (variable.value === null || variable.value === undefined) {
    return '<null>';
  }
  if (typeof variable.value === 'object') {
    return JSON.stringify(variable.value);
  }
  return String(variable.value);
}

function getTypeColor(type: string): string {
  const colors: Record<string, string> = {
    string: '#22c55e',
    number: '#3b82f6',
    boolean: '#f59e0b',
    list: '#6b7280',
    dict: '#ef4444',
  };
  return colors[type] || '#6b7280';
}
</script>

<template>
  <div class="variable-panel">
    <div class="panel-header">
      <span>变量</span>
      <button class="btn-icon btn-primary-icon" @click="showAddDialog = true" title="添加变量">
        <Plus :size="16" />
      </button>
    </div>

    <div v-if="variables.length === 0" class="panel-empty">
      <Package :size="48" class="empty-icon" />
      <span class="empty-text">暂无变量</span>
    </div>

    <div v-else class="variable-list">
      <div v-for="variable in variables" :key="variable.id" class="variable-item">
        <div class="variable-info">
          <span class="variable-name">{{ variable.name }}</span>
          <span class="variable-type" :style="{ backgroundColor: getTypeColor(variable.type) }">
            {{ variable.type }}
          </span>
        </div>
        <div class="variable-value" :title="getValueDisplay(variable)">
          {{ getValueDisplay(variable) }}
        </div>
        <div class="variable-actions">
          <button class="btn-icon" @click="startEdit(variable)" title="编辑">
            <Pencil :size="14" />
          </button>
          <button class="btn-icon btn-danger-icon" @click="deleteVariable(variable.id)" title="删除">
            <Trash2 :size="14" />
          </button>
        </div>
      </div>
    </div>

    <!-- Add Variable Dialog -->
    <div v-if="showAddDialog" class="dialog-overlay" @click.self="showAddDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <span>添加变量</span>
          <button class="btn-icon" @click="showAddDialog = false">
            <X :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <div class="form-item">
            <label class="form-label">变量名</label>
            <input v-model="newVariable.name" class="input" placeholder="输入变量名称" />
          </div>
          <div class="form-item">
            <label class="form-label">类型</label>
            <select v-model="newVariable.type" class="select">
              <option value="string">字符串</option>
              <option value="number">数字</option>
              <option value="boolean">布尔值</option>
              <option value="list">列表</option>
              <option value="dict">字典</option>
            </select>
          </div>
          <div class="form-item">
            <label class="form-label">初始值</label>
            <input v-model="newVariable.value" class="input" placeholder="输入初始值" />
          </div>
          <div class="form-item">
            <label class="form-label">作用域</label>
            <div class="radio-group">
              <label class="radio-label">
                <input type="radio" v-model="newVariable.scope" value="global" />
                全局
              </label>
              <label class="radio-label">
                <input type="radio" v-model="newVariable.scope" value="local" />
                局部
              </label>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="showAddDialog = false">取消</button>
          <button class="btn btn-primary" @click="addVariable">添加</button>
        </div>
      </div>
    </div>

    <!-- Edit Variable Dialog -->
    <div v-if="showEditDialog" class="dialog-overlay" @click.self="showEditDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <span>编辑变量</span>
          <button class="btn-icon" @click="showEditDialog = false">
            <X :size="18" />
          </button>
        </div>
        <div v-if="editingVariable" class="dialog-body">
          <div class="form-item">
            <label class="form-label">变量名</label>
            <input v-model="editingVariable.name" class="input" placeholder="输入变量名称" />
          </div>
          <div class="form-item">
            <label class="form-label">类型</label>
            <select v-model="editingVariable.type" class="select">
              <option value="string">字符串</option>
              <option value="number">数字</option>
              <option value="boolean">布尔值</option>
              <option value="list">列表</option>
              <option value="dict">字典</option>
            </select>
          </div>
          <div class="form-item">
            <label class="form-label">值</label>
            <input v-model="editingVariable.value" class="input" placeholder="输入值" />
          </div>
          <div class="form-item">
            <label class="form-label">作用域</label>
            <div class="radio-group">
              <label class="radio-label">
                <input type="radio" v-model="editingVariable.scope" value="global" />
                全局
              </label>
              <label class="radio-label">
                <input type="radio" v-model="editingVariable.scope" value="local" />
                局部
              </label>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="showEditDialog = false">取消</button>
          <button class="btn btn-primary" @click="saveEdit">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.variable-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.panel-header {
  padding: 12px 16px;
  font-weight: 500;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.btn-primary-icon {
  background: #3b82f6;
  color: #fff;
}

.btn-primary-icon:hover {
  background: #2563eb;
  color: #fff;
}

.btn-danger-icon:hover {
  background: #fef2f2;
  color: #ef4444;
}

.panel-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.empty-icon {
  color: #d1d5db;
}

.empty-text {
  color: #9ca3af;
  font-size: 14px;
}

.variable-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.variable-item {
  padding: 8px 12px;
  margin-bottom: 8px;
  background: #f9fafb;
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.variable-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.variable-name {
  font-weight: 500;
  font-size: 13px;
  color: #1f2937;
}

.variable-type {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  color: #fff;
}

.variable-value {
  font-size: 12px;
  color: #6b7280;
  font-family: monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.variable-actions {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  margin-top: 4px;
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

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: #374151;
  cursor: pointer;
}

.radio-label input {
  accent-color: #3b82f6;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}
</style>
