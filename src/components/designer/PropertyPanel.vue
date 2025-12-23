<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore, useElementStore } from '@/stores';
import { NODE_CONFIGS } from '@/types';
import { Delete, Aim } from '@element-plus/icons-vue';

const workflowStore = useWorkflowStore();
const elementStore = useElementStore();

const selectedNode = computed(() => workflowStore.selectedNode);
const nodeConfig = computed(() => {
  if (!selectedNode.value) return null;
  return NODE_CONFIGS[selectedNode.value.type as keyof typeof NODE_CONFIGS];
});

function updateNodeData(key: string, value: unknown) {
  if (selectedNode.value) {
    workflowStore.updateNode(selectedNode.value.id, {
      data: { ...selectedNode.value.data, [key]: value },
    });
  }
}

function deleteNode() {
  if (selectedNode.value) {
    workflowStore.removeNode(selectedNode.value.id);
  }
}

function selectElement() {
  elementStore.startCapture();
  // TODO: Invoke element capture, then update node data
}
</script>

<template>
  <div class="property-panel">
    <div class="panel-header">
      <span>属性配置</span>
    </div>

    <div v-if="!selectedNode" class="panel-empty">
      <el-empty description="请选择一个节点" :image-size="80" />
    </div>

    <div v-else class="panel-content">
      <div class="node-info">
        <div class="node-type" :style="{ backgroundColor: nodeConfig?.color }">
          {{ nodeConfig?.label }}
        </div>
        <el-button
          type="danger"
          :icon="Delete"
          size="small"
          circle
          @click="deleteNode"
        />
      </div>

      <el-form label-position="top" size="small">
        <el-form-item label="节点名称">
          <el-input
            :model-value="selectedNode.label"
            @update:model-value="workflowStore.updateNode(selectedNode!.id, { label: $event })"
          />
        </el-form-item>

        <!-- Click/Input node properties -->
        <template v-if="['click', 'input', 'getText'].includes(selectedNode.type)">
          <el-form-item label="目标元素">
            <div class="element-selector">
              <el-input
                :model-value="selectedNode.data.elementName || '未选择'"
                readonly
                placeholder="点击选择元素"
              />
              <el-button :icon="Aim" @click="selectElement">选取</el-button>
            </div>
          </el-form-item>
        </template>

        <!-- Input node specific -->
        <template v-if="selectedNode.type === 'input'">
          <el-form-item label="输入内容">
            <el-input
              :model-value="selectedNode.data.text || ''"
              @update:model-value="updateNodeData('text', $event)"
              placeholder="输入要填写的内容"
            />
          </el-form-item>
          <el-form-item label="输入方式">
            <el-select
              :model-value="selectedNode.data.inputMethod || 'type'"
              @update:model-value="updateNodeData('inputMethod', $event)"
            >
              <el-option label="模拟键入" value="type" />
              <el-option label="直接设置" value="set" />
            </el-select>
          </el-form-item>
        </template>

        <!-- Click node specific -->
        <template v-if="selectedNode.type === 'click'">
          <el-form-item label="点击类型">
            <el-select
              :model-value="selectedNode.data.clickType || 'single'"
              @update:model-value="updateNodeData('clickType', $event)"
            >
              <el-option label="单击" value="single" />
              <el-option label="双击" value="double" />
              <el-option label="右键" value="right" />
            </el-select>
          </el-form-item>
        </template>

        <!-- Delay node -->
        <template v-if="selectedNode.type === 'delay'">
          <el-form-item label="延时(毫秒)">
            <el-input-number
              :model-value="selectedNode.data.delay || 1000"
              @update:model-value="updateNodeData('delay', $event)"
              :min="0"
              :max="60000"
              :step="100"
            />
          </el-form-item>
        </template>

        <!-- Log node -->
        <template v-if="selectedNode.type === 'log'">
          <el-form-item label="日志内容">
            <el-input
              :model-value="selectedNode.data.message || ''"
              @update:model-value="updateNodeData('message', $event)"
              type="textarea"
              :rows="3"
              placeholder="输入日志内容，支持变量 ${varName}"
            />
          </el-form-item>
          <el-form-item label="日志级别">
            <el-select
              :model-value="selectedNode.data.level || 'info'"
              @update:model-value="updateNodeData('level', $event)"
            >
              <el-option label="Info" value="info" />
              <el-option label="Warn" value="warn" />
              <el-option label="Error" value="error" />
            </el-select>
          </el-form-item>
        </template>

        <!-- GetText node -->
        <template v-if="selectedNode.type === 'getText'">
          <el-form-item label="保存到变量">
            <el-input
              :model-value="selectedNode.data.variableName || ''"
              @update:model-value="updateNodeData('variableName', $event)"
              placeholder="变量名称"
            />
          </el-form-item>
        </template>

        <!-- Condition node -->
        <template v-if="selectedNode.type === 'condition'">
          <el-form-item label="条件表达式">
            <el-input
              :model-value="selectedNode.data.expression || ''"
              @update:model-value="updateNodeData('expression', $event)"
              placeholder="例如: ${count} > 10"
            />
          </el-form-item>
          <el-form-item label="左操作数">
            <el-input
              :model-value="selectedNode.data.leftOperand || ''"
              @update:model-value="updateNodeData('leftOperand', $event)"
              placeholder="变量名或值"
            />
          </el-form-item>
          <el-form-item label="比较运算符">
            <el-select
              :model-value="selectedNode.data.operator || '=='"
              @update:model-value="updateNodeData('operator', $event)"
            >
              <el-option label="等于 (==)" value="==" />
              <el-option label="不等于 (!=)" value="!=" />
              <el-option label="大于 (>)" value=">" />
              <el-option label="小于 (<)" value="<" />
              <el-option label="大于等于 (>=)" value=">=" />
              <el-option label="小于等于 (<=)" value="<=" />
              <el-option label="包含" value="contains" />
              <el-option label="为空" value="isEmpty" />
              <el-option label="不为空" value="isNotEmpty" />
            </el-select>
          </el-form-item>
          <el-form-item label="右操作数" v-if="!['isEmpty', 'isNotEmpty'].includes(String(selectedNode.data.operator))">
            <el-input
              :model-value="selectedNode.data.rightOperand || ''"
              @update:model-value="updateNodeData('rightOperand', $event)"
              placeholder="变量名或值"
            />
          </el-form-item>
        </template>

        <!-- Loop node -->
        <template v-if="selectedNode.type === 'loop'">
          <el-form-item label="循环类型">
            <el-select
              :model-value="selectedNode.data.loopType || 'count'"
              @update:model-value="updateNodeData('loopType', $event)"
            >
              <el-option label="固定次数" value="count" />
              <el-option label="条件循环" value="while" />
            </el-select>
          </el-form-item>
          <el-form-item v-if="selectedNode.data.loopType === 'count' || !selectedNode.data.loopType" label="循环次数">
            <el-input-number
              :model-value="selectedNode.data.count || 1"
              @update:model-value="updateNodeData('count', $event)"
              :min="1"
              :max="10000"
            />
          </el-form-item>
          <el-form-item v-if="selectedNode.data.loopType === 'while'" label="循环条件">
            <el-input
              :model-value="selectedNode.data.condition || ''"
              @update:model-value="updateNodeData('condition', $event)"
              placeholder="例如: ${index} < 10"
            />
          </el-form-item>
          <el-form-item label="计数器变量">
            <el-input
              :model-value="selectedNode.data.indexVariable || 'index'"
              @update:model-value="updateNodeData('indexVariable', $event)"
              placeholder="循环计数器变量名"
            />
          </el-form-item>
        </template>

        <!-- ForEach node -->
        <template v-if="selectedNode.type === 'forEach'">
          <el-form-item label="遍历列表">
            <el-input
              :model-value="selectedNode.data.listVariable || ''"
              @update:model-value="updateNodeData('listVariable', $event)"
              placeholder="列表变量名"
            />
          </el-form-item>
          <el-form-item label="元素变量">
            <el-input
              :model-value="selectedNode.data.itemVariable || 'item'"
              @update:model-value="updateNodeData('itemVariable', $event)"
              placeholder="当前元素变量名"
            />
          </el-form-item>
          <el-form-item label="索引变量">
            <el-input
              :model-value="selectedNode.data.indexVariable || 'index'"
              @update:model-value="updateNodeData('indexVariable', $event)"
              placeholder="当前索引变量名"
            />
          </el-form-item>
        </template>

        <!-- SetVariable node -->
        <template v-if="selectedNode.type === 'setVariable'">
          <el-form-item label="变量名">
            <el-input
              :model-value="selectedNode.data.variableName || ''"
              @update:model-value="updateNodeData('variableName', $event)"
              placeholder="变量名称"
            />
          </el-form-item>
          <el-form-item label="变量值">
            <el-input
              :model-value="selectedNode.data.value || ''"
              @update:model-value="updateNodeData('value', $event)"
              placeholder="变量值，支持 ${varName} 引用"
            />
          </el-form-item>
          <el-form-item label="值类型">
            <el-select
              :model-value="selectedNode.data.valueType || 'string'"
              @update:model-value="updateNodeData('valueType', $event)"
            >
              <el-option label="字符串" value="string" />
              <el-option label="数字" value="number" />
              <el-option label="布尔值" value="boolean" />
              <el-option label="JSON" value="json" />
            </el-select>
          </el-form-item>
        </template>

        <!-- ReadFile node -->
        <template v-if="selectedNode.type === 'readFile'">
          <el-form-item label="文件路径">
            <el-input
              :model-value="selectedNode.data.filePath || ''"
              @update:model-value="updateNodeData('filePath', $event)"
              placeholder="文件路径，支持变量"
            />
          </el-form-item>
          <el-form-item label="保存到变量">
            <el-input
              :model-value="selectedNode.data.variableName || 'fileContent'"
              @update:model-value="updateNodeData('variableName', $event)"
              placeholder="存储文件内容的变量名"
            />
          </el-form-item>
          <el-form-item label="编码">
            <el-select
              :model-value="selectedNode.data.encoding || 'utf-8'"
              @update:model-value="updateNodeData('encoding', $event)"
            >
              <el-option label="UTF-8" value="utf-8" />
              <el-option label="GBK" value="gbk" />
              <el-option label="ASCII" value="ascii" />
            </el-select>
          </el-form-item>
        </template>

        <!-- WriteFile node -->
        <template v-if="selectedNode.type === 'writeFile'">
          <el-form-item label="文件路径">
            <el-input
              :model-value="selectedNode.data.filePath || ''"
              @update:model-value="updateNodeData('filePath', $event)"
              placeholder="文件路径，支持变量"
            />
          </el-form-item>
          <el-form-item label="写入内容">
            <el-input
              :model-value="selectedNode.data.content || ''"
              @update:model-value="updateNodeData('content', $event)"
              type="textarea"
              :rows="3"
              placeholder="文件内容，支持变量 ${varName}"
            />
          </el-form-item>
          <el-form-item label="写入模式">
            <el-select
              :model-value="selectedNode.data.writeMode || 'overwrite'"
              @update:model-value="updateNodeData('writeMode', $event)"
            >
              <el-option label="覆盖" value="overwrite" />
              <el-option label="追加" value="append" />
            </el-select>
          </el-form-item>
        </template>

        <!-- TryCatch node -->
        <template v-if="selectedNode.type === 'tryCatch'">
          <el-form-item label="错误变量名">
            <el-input
              :model-value="selectedNode.data.errorVariable || 'error'"
              @update:model-value="updateNodeData('errorVariable', $event)"
              placeholder="存储错误信息的变量名"
            />
          </el-form-item>
          <el-form-item label="最大重试次数">
            <el-input-number
              :model-value="selectedNode.data.maxRetries || 0"
              @update:model-value="updateNodeData('maxRetries', $event)"
              :min="0"
              :max="10"
            />
          </el-form-item>
          <el-form-item label="重试间隔(毫秒)">
            <el-input-number
              :model-value="selectedNode.data.retryDelay || 1000"
              @update:model-value="updateNodeData('retryDelay', $event)"
              :min="0"
              :max="60000"
              :step="100"
            />
          </el-form-item>
        </template>

        <!-- Subflow node -->
        <template v-if="selectedNode.type === 'subflow'">
          <el-form-item label="子流程ID">
            <el-input
              :model-value="selectedNode.data.subflowId || ''"
              @update:model-value="updateNodeData('subflowId', $event)"
              placeholder="子流程标识符"
            />
          </el-form-item>
          <el-form-item label="输入参数">
            <el-input
              :model-value="selectedNode.data.inputParams || ''"
              @update:model-value="updateNodeData('inputParams', $event)"
              type="textarea"
              :rows="2"
              placeholder="JSON格式参数"
            />
          </el-form-item>
          <el-form-item label="输出变量">
            <el-input
              :model-value="selectedNode.data.outputVariable || 'result'"
              @update:model-value="updateNodeData('outputVariable', $event)"
              placeholder="存储子流程结果的变量名"
            />
          </el-form-item>
        </template>

        <!-- WaitElement node -->
        <template v-if="selectedNode.type === 'waitElement'">
          <el-form-item label="目标元素">
            <div class="element-selector">
              <el-input
                :model-value="selectedNode.data.elementName || '未选择'"
                readonly
                placeholder="点击选择元素"
              />
              <el-button :icon="Aim" @click="selectElement">选取</el-button>
            </div>
          </el-form-item>
          <el-form-item label="超时时间(毫秒)">
            <el-input-number
              :model-value="selectedNode.data.timeout || 30000"
              @update:model-value="updateNodeData('timeout', $event)"
              :min="1000"
              :max="300000"
              :step="1000"
            />
          </el-form-item>
          <el-form-item label="等待条件">
            <el-select
              :model-value="selectedNode.data.waitCondition || 'visible'"
              @update:model-value="updateNodeData('waitCondition', $event)"
            >
              <el-option label="元素可见" value="visible" />
              <el-option label="元素存在" value="exists" />
              <el-option label="元素可点击" value="clickable" />
              <el-option label="元素消失" value="hidden" />
            </el-select>
          </el-form-item>
        </template>

        <!-- OpenBrowser node -->
        <template v-if="selectedNode.type === 'openBrowser'">
          <el-form-item label="浏览器类型">
            <el-select
              :model-value="selectedNode.data.browserType || 'chrome'"
              @update:model-value="updateNodeData('browserType', $event)"
            >
              <el-option label="Chrome" value="chrome" />
              <el-option label="Edge" value="edge" />
              <el-option label="Firefox" value="firefox" />
            </el-select>
          </el-form-item>
          <el-form-item label="无头模式">
            <el-switch
              :model-value="selectedNode.data.headless || false"
              @update:model-value="updateNodeData('headless', $event)"
            />
          </el-form-item>
          <el-form-item label="初始URL">
            <el-input
              :model-value="selectedNode.data.initialUrl || ''"
              @update:model-value="updateNodeData('initialUrl', $event)"
              placeholder="https://example.com"
            />
          </el-form-item>
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="存储浏览器实例的变量名"
            />
          </el-form-item>
        </template>

        <!-- Navigate node -->
        <template v-if="selectedNode.type === 'navigate'">
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="浏览器实例变量名"
            />
          </el-form-item>
          <el-form-item label="目标URL">
            <el-input
              :model-value="selectedNode.data.url || ''"
              @update:model-value="updateNodeData('url', $event)"
              placeholder="https://example.com"
            />
          </el-form-item>
          <el-form-item label="等待加载">
            <el-select
              :model-value="selectedNode.data.waitUntil || 'load'"
              @update:model-value="updateNodeData('waitUntil', $event)"
            >
              <el-option label="页面加载完成" value="load" />
              <el-option label="DOM加载完成" value="domcontentloaded" />
              <el-option label="网络空闲" value="networkidle" />
            </el-select>
          </el-form-item>
        </template>

        <!-- Screenshot node -->
        <template v-if="selectedNode.type === 'screenshot'">
          <el-form-item label="保存路径">
            <el-input
              :model-value="selectedNode.data.filePath || ''"
              @update:model-value="updateNodeData('filePath', $event)"
              placeholder="截图保存路径"
            />
          </el-form-item>
          <el-form-item label="截图类型">
            <el-select
              :model-value="selectedNode.data.screenshotType || 'fullPage'"
              @update:model-value="updateNodeData('screenshotType', $event)"
            >
              <el-option label="整个页面" value="fullPage" />
              <el-option label="可见区域" value="viewport" />
              <el-option label="指定元素" value="element" />
            </el-select>
          </el-form-item>
          <el-form-item label="图片格式">
            <el-select
              :model-value="selectedNode.data.imageFormat || 'png'"
              @update:model-value="updateNodeData('imageFormat', $event)"
            >
              <el-option label="PNG" value="png" />
              <el-option label="JPEG" value="jpeg" />
            </el-select>
          </el-form-item>
        </template>

        <!-- Hotkey node -->
        <template v-if="selectedNode.type === 'hotkey'">
          <el-form-item label="快捷键">
            <el-input
              :model-value="selectedNode.data.keys || ''"
              @update:model-value="updateNodeData('keys', $event)"
              placeholder="例如: Ctrl+C, Alt+Tab"
            />
          </el-form-item>
          <el-form-item label="修饰键">
            <el-checkbox-group
              :model-value="selectedNode.data.modifiers || []"
              @update:model-value="updateNodeData('modifiers', $event)"
            >
              <el-checkbox value="ctrl">Ctrl</el-checkbox>
              <el-checkbox value="alt">Alt</el-checkbox>
              <el-checkbox value="shift">Shift</el-checkbox>
              <el-checkbox value="meta">Meta</el-checkbox>
            </el-checkbox-group>
          </el-form-item>
          <el-form-item label="按键">
            <el-input
              :model-value="selectedNode.data.key || ''"
              @update:model-value="updateNodeData('key', $event)"
              placeholder="例如: a, Enter, Tab"
            />
          </el-form-item>
        </template>

        <!-- WebClick node -->
        <template v-if="selectedNode.type === 'webClick'">
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="浏览器实例变量名"
            />
          </el-form-item>
          <el-form-item label="CSS选择器">
            <el-input
              :model-value="selectedNode.data.selector || ''"
              @update:model-value="updateNodeData('selector', $event)"
              placeholder="例如: #button, .submit-btn"
            />
          </el-form-item>
          <el-form-item label="点击类型">
            <el-select
              :model-value="selectedNode.data.clickType || 'single'"
              @update:model-value="updateNodeData('clickType', $event)"
            >
              <el-option label="单击" value="single" />
              <el-option label="双击" value="double" />
              <el-option label="右键" value="right" />
            </el-select>
          </el-form-item>
        </template>

        <!-- WebInput node -->
        <template v-if="selectedNode.type === 'webInput'">
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="浏览器实例变量名"
            />
          </el-form-item>
          <el-form-item label="CSS选择器">
            <el-input
              :model-value="selectedNode.data.selector || ''"
              @update:model-value="updateNodeData('selector', $event)"
              placeholder="例如: #username, input[name='email']"
            />
          </el-form-item>
          <el-form-item label="输入内容">
            <el-input
              :model-value="selectedNode.data.text || ''"
              @update:model-value="updateNodeData('text', $event)"
              placeholder="输入要填写的内容，支持变量 ${varName}"
            />
          </el-form-item>
          <el-form-item label="清空后输入">
            <el-switch
              :model-value="selectedNode.data.clearFirst || true"
              @update:model-value="updateNodeData('clearFirst', $event)"
            />
          </el-form-item>
        </template>

        <!-- WebGetText node -->
        <template v-if="selectedNode.type === 'webGetText'">
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="浏览器实例变量名"
            />
          </el-form-item>
          <el-form-item label="CSS选择器">
            <el-input
              :model-value="selectedNode.data.selector || ''"
              @update:model-value="updateNodeData('selector', $event)"
              placeholder="例如: .result, #output"
            />
          </el-form-item>
          <el-form-item label="保存到变量">
            <el-input
              :model-value="selectedNode.data.variableName || 'result'"
              @update:model-value="updateNodeData('variableName', $event)"
              placeholder="存储获取文本的变量名"
            />
          </el-form-item>
          <el-form-item label="获取内容">
            <el-select
              :model-value="selectedNode.data.textType || 'innerText'"
              @update:model-value="updateNodeData('textType', $event)"
            >
              <el-option label="文本内容" value="innerText" />
              <el-option label="HTML内容" value="innerHTML" />
              <el-option label="属性值" value="attribute" />
            </el-select>
          </el-form-item>
          <el-form-item v-if="selectedNode.data.textType === 'attribute'" label="属性名">
            <el-input
              :model-value="selectedNode.data.attributeName || ''"
              @update:model-value="updateNodeData('attributeName', $event)"
              placeholder="例如: href, src, value"
            />
          </el-form-item>
        </template>

        <!-- CloseBrowser node -->
        <template v-if="selectedNode.type === 'closeBrowser'">
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="要关闭的浏览器实例变量名"
            />
          </el-form-item>
        </template>

        <!-- ExecuteJs node -->
        <template v-if="selectedNode.type === 'executeJs'">
          <el-form-item label="浏览器变量">
            <el-input
              :model-value="selectedNode.data.browserVariable || 'browser'"
              @update:model-value="updateNodeData('browserVariable', $event)"
              placeholder="浏览器实例变量名"
            />
          </el-form-item>
          <el-form-item label="JavaScript代码">
            <el-input
              :model-value="selectedNode.data.script || ''"
              @update:model-value="updateNodeData('script', $event)"
              type="textarea"
              :rows="5"
              placeholder="return document.title;"
            />
          </el-form-item>
          <el-form-item label="保存结果到变量">
            <el-input
              :model-value="selectedNode.data.resultVariable || ''"
              @update:model-value="updateNodeData('resultVariable', $event)"
              placeholder="留空则不保存返回值"
            />
          </el-form-item>
        </template>

        <!-- ReadExcel node -->
        <template v-if="selectedNode.type === 'readExcel'">
          <el-form-item label="文件路径">
            <el-input
              :model-value="selectedNode.data.filePath || ''"
              @update:model-value="updateNodeData('filePath', $event)"
              placeholder="Excel文件路径 (.xlsx)"
            />
          </el-form-item>
          <el-form-item label="工作表名称">
            <el-input
              :model-value="selectedNode.data.sheetName || ''"
              @update:model-value="updateNodeData('sheetName', $event)"
              placeholder="留空读取第一个工作表"
            />
          </el-form-item>
          <el-form-item label="保存到变量">
            <el-input
              :model-value="selectedNode.data.variableName || 'excelData'"
              @update:model-value="updateNodeData('variableName', $event)"
              placeholder="存储Excel数据的变量名"
            />
          </el-form-item>
          <el-form-item label="读取范围">
            <el-select
              :model-value="selectedNode.data.readMode || 'all'"
              @update:model-value="updateNodeData('readMode', $event)"
            >
              <el-option label="全部数据" value="all" />
              <el-option label="指定单元格" value="cell" />
              <el-option label="指定范围" value="range" />
            </el-select>
          </el-form-item>
          <el-form-item v-if="selectedNode.data.readMode === 'cell'" label="单元格位置">
            <el-input
              :model-value="selectedNode.data.cellPosition || 'A1'"
              @update:model-value="updateNodeData('cellPosition', $event)"
              placeholder="例如: A1, B2"
            />
          </el-form-item>
        </template>

        <!-- WriteExcel node -->
        <template v-if="selectedNode.type === 'writeExcel'">
          <el-form-item label="文件路径">
            <el-input
              :model-value="selectedNode.data.filePath || ''"
              @update:model-value="updateNodeData('filePath', $event)"
              placeholder="Excel文件路径 (.xlsx)"
            />
          </el-form-item>
          <el-form-item label="工作表名称">
            <el-input
              :model-value="selectedNode.data.sheetName || 'Sheet1'"
              @update:model-value="updateNodeData('sheetName', $event)"
              placeholder="工作表名称"
            />
          </el-form-item>
          <el-form-item label="数据来源">
            <el-input
              :model-value="selectedNode.data.dataVariable || ''"
              @update:model-value="updateNodeData('dataVariable', $event)"
              placeholder="包含表格数据的变量名"
            />
          </el-form-item>
          <el-form-item label="写入模式">
            <el-select
              :model-value="selectedNode.data.writeMode || 'overwrite'"
              @update:model-value="updateNodeData('writeMode', $event)"
            >
              <el-option label="覆盖" value="overwrite" />
              <el-option label="追加" value="append" />
            </el-select>
          </el-form-item>
        </template>

      </el-form>
    </div>
  </div>
</template>

<style scoped>
.property-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.panel-header {
  padding: 12px 16px;
  font-weight: 500;
  border-bottom: 1px solid var(--el-border-color-light);
}

.panel-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.node-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.node-type {
  padding: 4px 12px;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  font-weight: 500;
}

.element-selector {
  display: flex;
  gap: 8px;
}

.element-selector .el-input {
  flex: 1;
}
</style>
