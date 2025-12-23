<script setup lang="ts">
import { computed, watch, ref } from 'vue';
import { useWorkflowStore, useElementStore } from '@/stores';
import { NODE_CONFIGS } from '@/types';
import { Trash2, Crosshair, X } from 'lucide-vue-next';

const workflowStore = useWorkflowStore();
const elementStore = useElementStore();

const selectedNode = computed(() => workflowStore.selectedNode);
const nodeConfig = computed(() => {
  if (!selectedNode.value) return null;
  return NODE_CONFIGS[selectedNode.value.type as keyof typeof NODE_CONFIGS];
});

const formData = ref<Record<string, unknown>>({});

watch(
  selectedNode,
  (node) => {
    if (node) {
      formData.value = { ...node.data };
    }
  },
  { immediate: true }
);

function updateNodeData(key: string, value: unknown) {
  if (selectedNode.value) {
    workflowStore.updateNode(selectedNode.value.id, {
      data: { ...selectedNode.value.data, [key]: value },
    });
  }
}

function updateLabel(value: string) {
  if (selectedNode.value) {
    workflowStore.updateNode(selectedNode.value.id, { label: value });
  }
}

function deleteNode() {
  if (selectedNode.value) {
    workflowStore.removeNode(selectedNode.value.id);
  }
}

function selectElement() {
  elementStore.startCapture();
}

function clearSelection() {
  workflowStore.selectNode(null);
}
</script>

<template>
  <div class="h-full flex flex-col bg-white">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200">
      <span class="font-medium text-gray-700">属性配置</span>
      <button
        v-if="selectedNode"
        class="p-1 rounded hover:bg-gray-100 transition-colors"
        @click="clearSelection"
      >
        <X :size="16" class="text-gray-400" />
      </button>
    </div>

    <!-- Empty State -->
    <div v-if="!selectedNode" class="flex-1 flex items-center justify-center">
      <div class="text-center text-gray-400">
        <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <p class="text-sm">请选择一个节点</p>
      </div>
    </div>

    <!-- Properties -->
    <div v-else class="flex-1 overflow-y-auto p-4">
      <!-- Node Info -->
      <div class="flex items-center justify-between mb-4">
        <div
          class="px-3 py-1 rounded text-white text-sm font-medium"
          :style="{ backgroundColor: nodeConfig?.color }"
        >
          {{ nodeConfig?.label }}
        </div>
        <button
          class="p-1.5 rounded-full text-red-500 hover:bg-red-50 transition-colors"
          @click="deleteNode"
        >
          <Trash2 :size="16" />
        </button>
      </div>

      <!-- Form -->
      <div class="space-y-4">
        <!-- Common: Node Label -->
        <div class="form-item">
          <label class="form-label">节点名称</label>
          <input
            type="text"
            class="input input-sm"
            :value="selectedNode.label"
            @input="updateLabel(($event.target as HTMLInputElement).value)"
          />
        </div>

        <!-- Click/Input/GetText: Element Selector -->
        <template v-if="['click', 'input', 'getText'].includes(selectedNode.type)">
          <div class="form-item">
            <label class="form-label">目标元素</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input input-sm flex-1"
                :value="selectedNode.data.elementName || '未选择'"
                readonly
                placeholder="点击选择元素"
              />
              <button class="btn btn-sm btn-default" @click="selectElement">
                <Crosshair :size="14" />
                选取
              </button>
            </div>
          </div>
        </template>

        <!-- Input: Text -->
        <template v-if="selectedNode.type === 'input'">
          <div class="form-item">
            <label class="form-label">输入内容</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.text || ''"
              placeholder="输入要填写的内容"
              @input="updateNodeData('text', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">输入方式</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.inputMethod || 'type'"
              @change="updateNodeData('inputMethod', ($event.target as HTMLSelectElement).value)"
            >
              <option value="type">模拟键入</option>
              <option value="set">直接设置</option>
            </select>
          </div>
        </template>

        <!-- Click: Type -->
        <template v-if="selectedNode.type === 'click'">
          <div class="form-item">
            <label class="form-label">点击类型</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.clickType || 'single'"
              @change="updateNodeData('clickType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="single">单击</option>
              <option value="double">双击</option>
              <option value="right">右键</option>
            </select>
          </div>
        </template>

        <!-- Delay -->
        <template v-if="selectedNode.type === 'delay'">
          <div class="form-item">
            <label class="form-label">延时(毫秒)</label>
            <input
              type="number"
              class="input input-sm"
              :value="selectedNode.data.delay || 1000"
              min="0"
              max="60000"
              step="100"
              @input="updateNodeData('delay', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
        </template>

        <!-- Log -->
        <template v-if="selectedNode.type === 'log'">
          <div class="form-item">
            <label class="form-label">日志内容</label>
            <textarea
              class="textarea text-sm"
              rows="3"
              :value="String(selectedNode.data.message || '')"
              placeholder="输入日志内容，支持变量 ${varName}"
              @input="updateNodeData('message', ($event.target as HTMLTextAreaElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">日志级别</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.level || 'info'"
              @change="updateNodeData('level', ($event.target as HTMLSelectElement).value)"
            >
              <option value="info">Info</option>
              <option value="warn">Warn</option>
              <option value="error">Error</option>
            </select>
          </div>
        </template>

        <!-- GetText -->
        <template v-if="selectedNode.type === 'getText'">
          <div class="form-item">
            <label class="form-label">保存到变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.variableName || ''"
              placeholder="变量名称"
              @input="updateNodeData('variableName', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- Condition -->
        <template v-if="selectedNode.type === 'condition'">
          <div class="form-item">
            <label class="form-label">条件表达式</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.expression || ''"
              placeholder="例如: ${count} > 10"
              @input="updateNodeData('expression', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">左操作数</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.leftOperand || ''"
              placeholder="变量名或值"
              @input="updateNodeData('leftOperand', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">比较运算符</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.operator || '=='"
              @change="updateNodeData('operator', ($event.target as HTMLSelectElement).value)"
            >
              <option value="==">等于 (==)</option>
              <option value="!=">不等于 (!=)</option>
              <option value=">">大于 (>)</option>
              <option value="<">小于 (<)</option>
              <option value=">=">大于等于 (>=)</option>
              <option value="<=">小于等于 (<=)</option>
              <option value="contains">包含</option>
              <option value="isEmpty">为空</option>
              <option value="isNotEmpty">不为空</option>
            </select>
          </div>
          <div v-if="!['isEmpty', 'isNotEmpty'].includes(String(selectedNode.data.operator))" class="form-item">
            <label class="form-label">右操作数</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.rightOperand || ''"
              placeholder="变量名或值"
              @input="updateNodeData('rightOperand', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- Loop -->
        <template v-if="selectedNode.type === 'loop'">
          <div class="form-item">
            <label class="form-label">循环类型</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.loopType || 'count'"
              @change="updateNodeData('loopType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="count">固定次数</option>
              <option value="while">条件循环</option>
            </select>
          </div>
          <div v-if="selectedNode.data.loopType === 'count' || !selectedNode.data.loopType" class="form-item">
            <label class="form-label">循环次数</label>
            <input
              type="number"
              class="input input-sm"
              :value="selectedNode.data.count || 1"
              min="1"
              max="10000"
              @input="updateNodeData('count', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
          <div v-if="selectedNode.data.loopType === 'while'" class="form-item">
            <label class="form-label">循环条件</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.condition || ''"
              placeholder="例如: ${index} < 10"
              @input="updateNodeData('condition', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">计数器变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.indexVariable || 'index'"
              placeholder="循环计数器变量名"
              @input="updateNodeData('indexVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- ForEach -->
        <template v-if="selectedNode.type === 'forEach'">
          <div class="form-item">
            <label class="form-label">遍历列表</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.listVariable || ''"
              placeholder="列表变量名"
              @input="updateNodeData('listVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">元素变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.itemVariable || 'item'"
              placeholder="当前元素变量名"
              @input="updateNodeData('itemVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">索引变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.indexVariable || 'index'"
              placeholder="当前索引变量名"
              @input="updateNodeData('indexVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- SetVariable -->
        <template v-if="selectedNode.type === 'setVariable'">
          <div class="form-item">
            <label class="form-label">变量名</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.variableName || ''"
              placeholder="变量名称"
              @input="updateNodeData('variableName', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">变量值</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.value || ''"
              placeholder="变量值，支持 ${varName} 引用"
              @input="updateNodeData('value', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">值类型</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.valueType || 'string'"
              @change="updateNodeData('valueType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="string">字符串</option>
              <option value="number">数字</option>
              <option value="boolean">布尔值</option>
              <option value="json">JSON</option>
            </select>
          </div>
        </template>

        <!-- OpenBrowser -->
        <template v-if="selectedNode.type === 'openBrowser'">
          <div class="form-item">
            <label class="form-label">浏览器类型</label>
            <select
              class="select select-sm"
              :value="selectedNode.data.browserType || 'chrome'"
              @change="updateNodeData('browserType', ($event.target as HTMLSelectElement).value)"
            >
              <option value="chrome">Chrome</option>
              <option value="edge">Edge</option>
              <option value="firefox">Firefox</option>
            </select>
          </div>
          <div class="form-item">
            <label class="flex items-center gap-2">
              <input
                type="checkbox"
                class="checkbox"
                :checked="selectedNode.data.headless === true"
                @change="updateNodeData('headless', ($event.target as HTMLInputElement).checked)"
              />
              <span class="text-sm text-gray-700">无头模式</span>
            </label>
          </div>
          <div class="form-item">
            <label class="form-label">初始URL</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.initialUrl || ''"
              placeholder="https://example.com"
              @input="updateNodeData('initialUrl', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- Navigate -->
        <template v-if="selectedNode.type === 'navigate'">
          <div class="form-item">
            <label class="form-label">目标URL</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.url || ''"
              placeholder="https://example.com"
              @input="updateNodeData('url', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- WebClick/WebInput/WebGetText -->
        <template v-if="['webClick', 'webInput', 'webGetText'].includes(selectedNode.type)">
          <div class="form-item">
            <label class="form-label">CSS选择器</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.selector || ''"
              placeholder="例如: #button, .submit-btn"
              @input="updateNodeData('selector', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <template v-if="selectedNode.type === 'webInput'">
          <div class="form-item">
            <label class="form-label">输入内容</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.text || ''"
              placeholder="输入要填写的内容"
              @input="updateNodeData('text', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <template v-if="selectedNode.type === 'webGetText'">
          <div class="form-item">
            <label class="form-label">保存到变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.variableName || 'result'"
              placeholder="变量名"
              @input="updateNodeData('variableName', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- ReadExcel/WriteExcel -->
        <template v-if="['readExcel', 'writeExcel'].includes(selectedNode.type)">
          <div class="form-item">
            <label class="form-label">文件路径</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.filePath || ''"
              placeholder="Excel文件路径 (.xlsx)"
              @input="updateNodeData('filePath', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">工作表名称</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.sheetName || 'Sheet1'"
              placeholder="工作表名称"
              @input="updateNodeData('sheetName', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <template v-if="selectedNode.type === 'readExcel'">
          <div class="form-item">
            <label class="form-label">保存到变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.variableName || 'excelData'"
              placeholder="变量名"
              @input="updateNodeData('variableName', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <template v-if="selectedNode.type === 'writeExcel'">
          <div class="form-item">
            <label class="form-label">数据来源变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.dataVariable || ''"
              placeholder="变量名"
              @input="updateNodeData('dataVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- ExecuteCommand -->
        <template v-if="selectedNode.type === 'executeCommand'">
          <div class="form-item">
            <label class="form-label">命令</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.command || ''"
              placeholder="如: ls, echo"
              @input="updateNodeData('command', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">参数</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.args || ''"
              placeholder="命令参数，空格分隔"
              @input="updateNodeData('args', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">保存输出到变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.outputVariable || ''"
              placeholder="变量名"
              @input="updateNodeData('outputVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- ListDirectory -->
        <template v-if="selectedNode.type === 'listDirectory'">
          <div class="form-item">
            <label class="form-label">目录路径</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.path || ''"
              placeholder="如: ~/Documents"
              @input="updateNodeData('path', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="flex items-center gap-2">
              <input
                type="checkbox"
                class="checkbox"
                :checked="selectedNode.data.recursive === true"
                @change="updateNodeData('recursive', ($event.target as HTMLInputElement).checked)"
              />
              <span class="text-sm text-gray-700">递归遍历子目录</span>
            </label>
          </div>
          <div class="form-item">
            <label class="flex items-center gap-2">
              <input
                type="checkbox"
                class="checkbox"
                :checked="selectedNode.data.includeHidden === true"
                @change="updateNodeData('includeHidden', ($event.target as HTMLInputElement).checked)"
              />
              <span class="text-sm text-gray-700">包含隐藏文件</span>
            </label>
          </div>
          <div class="form-item">
            <label class="form-label">保存到变量</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.outputVariable || ''"
              placeholder="变量名"
              @input="updateNodeData('outputVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- OpenApp -->
        <template v-if="selectedNode.type === 'openApp'">
          <div class="form-item">
            <label class="form-label">应用名称</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.appName || ''"
              placeholder="如: Finder, Safari"
              @input="updateNodeData('appName', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>

        <!-- TryCatch -->
        <template v-if="selectedNode.type === 'tryCatch'">
          <div class="form-item">
            <label class="form-label">错误变量名</label>
            <input
              type="text"
              class="input input-sm"
              :value="selectedNode.data.errorVariable || 'error'"
              placeholder="存储错误信息的变量名"
              @input="updateNodeData('errorVariable', ($event.target as HTMLInputElement).value)"
            />
          </div>
          <div class="form-item">
            <label class="form-label">最大重试次数</label>
            <input
              type="number"
              class="input input-sm"
              :value="selectedNode.data.maxRetries || 0"
              min="0"
              max="10"
              @input="updateNodeData('maxRetries', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
