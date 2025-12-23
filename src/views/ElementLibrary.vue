<script setup lang="ts">
import { ref } from 'vue';
import { useElementStore } from '@/stores';
import { Aim, Delete, View } from '@element-plus/icons-vue';

const elementStore = useElementStore();
const showCreateDialog = ref(false);
const newLibraryName = ref('');

function createLibrary() {
  if (newLibraryName.value.trim()) {
    elementStore.createLibrary(newLibraryName.value.trim());
    newLibraryName.value = '';
    showCreateDialog.value = false;
  }
}

async function startCapture() {
  elementStore.startCapture();
  // TODO: Invoke Rust element capture
}
</script>

<template>
  <div class="element-library">
    <div class="library-header">
      <h3>元素库</h3>
      <el-button-group>
        <el-button type="primary" @click="startCapture" :loading="elementStore.isCapturing">
          <el-icon><Aim /></el-icon>
          捕获元素
        </el-button>
        <el-button @click="showCreateDialog = true">新建库</el-button>
      </el-button-group>
    </div>

    <div class="library-content">
      <el-empty v-if="!elementStore.currentLibrary" description="暂无元素库，请先创建">
        <el-button type="primary" @click="showCreateDialog = true">创建元素库</el-button>
      </el-empty>

      <template v-else>
        <div class="library-info">
          <h4>{{ elementStore.currentLibrary.name }}</h4>
          <span>{{ elementStore.currentLibrary.elements.length }} 个元素</span>
        </div>

        <el-table :data="elementStore.currentLibrary.elements" style="width: 100%">
          <el-table-column prop="name" label="名称" />
          <el-table-column prop="controlType" label="控件类型" width="120" />
          <el-table-column prop="windowTitle" label="窗口" />
          <el-table-column label="操作" width="120">
            <template #default="{ row }">
              <el-button-group size="small">
                <el-button :icon="View" @click="elementStore.selectElement(row)" />
                <el-button :icon="Delete" type="danger" @click="elementStore.removeElement(row.id)" />
              </el-button-group>
            </template>
          </el-table-column>
        </el-table>
      </template>
    </div>

    <el-dialog v-model="showCreateDialog" title="新建元素库" width="400">
      <el-input v-model="newLibraryName" placeholder="请输入元素库名称" />
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="createLibrary">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.element-library {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
}

.library-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.library-content {
  flex: 1;
  overflow: auto;
}

.library-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.library-info h4 {
  margin: 0;
}

.library-info span {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}
</style>
