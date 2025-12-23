<script setup lang="ts">
import { ref, computed } from 'vue';
import { useWorkflowStore } from '@/stores';
import { Plus, Delete, Edit } from '@element-plus/icons-vue';
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
    string: '#67c23a',
    number: '#409eff',
    boolean: '#e6a23c',
    list: '#909399',
    dict: '#f56c6c',
  };
  return colors[type] || '#909399';
}
</script>

<template>
  <div class="variable-panel">
    <div class="panel-header">
      <span>变量</span>
      <el-button type="primary" :icon="Plus" size="small" circle @click="showAddDialog = true" />
    </div>

    <div v-if="variables.length === 0" class="panel-empty">
      <el-empty description="暂无变量" :image-size="60" />
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
          <el-button :icon="Edit" size="small" circle @click="startEdit(variable)" />
          <el-button :icon="Delete" size="small" circle type="danger" @click="deleteVariable(variable.id)" />
        </div>
      </div>
    </div>

    <!-- Add Variable Dialog -->
    <el-dialog v-model="showAddDialog" title="添加变量" width="400px">
      <el-form label-position="top">
        <el-form-item label="变量名">
          <el-input v-model="newVariable.name" placeholder="输入变量名称" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="newVariable.type" style="width: 100%">
            <el-option label="字符串" value="string" />
            <el-option label="数字" value="number" />
            <el-option label="布尔值" value="boolean" />
            <el-option label="列表" value="list" />
            <el-option label="字典" value="dict" />
          </el-select>
        </el-form-item>
        <el-form-item label="初始值">
          <el-input v-model="newVariable.value" placeholder="输入初始值" />
        </el-form-item>
        <el-form-item label="作用域">
          <el-radio-group v-model="newVariable.scope">
            <el-radio value="global">全局</el-radio>
            <el-radio value="local">局部</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="addVariable">添加</el-button>
      </template>
    </el-dialog>

    <!-- Edit Variable Dialog -->
    <el-dialog v-model="showEditDialog" title="编辑变量" width="400px">
      <el-form v-if="editingVariable" label-position="top">
        <el-form-item label="变量名">
          <el-input v-model="editingVariable.name" placeholder="输入变量名称" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="editingVariable.type" style="width: 100%">
            <el-option label="字符串" value="string" />
            <el-option label="数字" value="number" />
            <el-option label="布尔值" value="boolean" />
            <el-option label="列表" value="list" />
            <el-option label="字典" value="dict" />
          </el-select>
        </el-form-item>
        <el-form-item label="值">
          <el-input v-model="editingVariable.value" placeholder="输入值" />
        </el-form-item>
        <el-form-item label="作用域">
          <el-radio-group v-model="editingVariable.scope">
            <el-radio value="global">全局</el-radio>
            <el-radio value="local">局部</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showEditDialog = false">取消</el-button>
        <el-button type="primary" @click="saveEdit">保存</el-button>
      </template>
    </el-dialog>
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
  border-bottom: 1px solid var(--el-border-color-light);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.panel-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.variable-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.variable-item {
  padding: 8px 12px;
  margin-bottom: 8px;
  background: var(--el-fill-color-light);
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
  color: var(--el-text-color-primary);
}

.variable-type {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  color: #fff;
}

.variable-value {
  font-size: 12px;
  color: var(--el-text-color-secondary);
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
</style>
