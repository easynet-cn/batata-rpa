export interface Position {
  x: number;
  y: number;
}

export interface WorkflowNode {
  id: string;
  type: string;
  position: Position;
  data: Record<string, unknown>;
  label?: string;
}

export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  label?: string;
}

export interface Variable {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'list' | 'dict';
  value: unknown;
  scope: 'global' | 'local';
}

export interface Workflow {
  id: string;
  name: string;
  description?: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: Variable[];
  createdAt: string;
  updatedAt: string;
}

export type NodeType =
  | 'start'
  | 'end'
  | 'click'
  | 'input'
  | 'getText'
  | 'condition'
  | 'loop'
  | 'forEach'
  | 'delay'
  | 'log'
  | 'setVariable'
  | 'readFile'
  | 'writeFile'
  | 'tryCatch'
  | 'subflow'
  | 'waitElement'
  | 'openBrowser'
  | 'navigate'
  | 'screenshot'
  | 'hotkey'
  | 'webClick'
  | 'webInput'
  | 'webGetText'
  | 'closeBrowser'
  | 'executeJs'
  | 'readExcel'
  | 'writeExcel'
  | 'executeCommand'
  | 'listDirectory'
  | 'openApp';

export interface NodeConfig {
  type: NodeType;
  label: string;
  icon: string;
  category: 'control' | 'action' | 'data';
  color: string;
}

// Lucide icon names mapping
export const NODE_CONFIGS: Record<NodeType, NodeConfig> = {
  start: {
    type: 'start',
    label: '开始',
    icon: 'Play',
    category: 'control',
    color: '#22c55e',
  },
  end: {
    type: 'end',
    label: '结束',
    icon: 'Square',
    category: 'control',
    color: '#ef4444',
  },
  click: {
    type: 'click',
    label: '点击',
    icon: 'MousePointer',
    category: 'action',
    color: '#3b82f6',
  },
  input: {
    type: 'input',
    label: '输入',
    icon: 'PenLine',
    category: 'action',
    color: '#3b82f6',
  },
  getText: {
    type: 'getText',
    label: '获取文本',
    icon: 'FileText',
    category: 'data',
    color: '#f59e0b',
  },
  condition: {
    type: 'condition',
    label: '条件分支',
    icon: 'GitBranch',
    category: 'control',
    color: '#f59e0b',
  },
  loop: {
    type: 'loop',
    label: '循环',
    icon: 'RefreshCw',
    category: 'control',
    color: '#6b7280',
  },
  forEach: {
    type: 'forEach',
    label: '遍历',
    icon: 'List',
    category: 'control',
    color: '#6b7280',
  },
  delay: {
    type: 'delay',
    label: '延时',
    icon: 'Clock',
    category: 'action',
    color: '#3b82f6',
  },
  log: {
    type: 'log',
    label: '日志',
    icon: 'FileText',
    category: 'data',
    color: '#f59e0b',
  },
  setVariable: {
    type: 'setVariable',
    label: '设置变量',
    icon: 'Variable',
    category: 'data',
    color: '#f59e0b',
  },
  readFile: {
    type: 'readFile',
    label: '读取文件',
    icon: 'FolderOpen',
    category: 'data',
    color: '#22c55e',
  },
  writeFile: {
    type: 'writeFile',
    label: '写入文件',
    icon: 'FolderClosed',
    category: 'data',
    color: '#22c55e',
  },
  tryCatch: {
    type: 'tryCatch',
    label: '异常处理',
    icon: 'AlertTriangle',
    category: 'control',
    color: '#ef4444',
  },
  subflow: {
    type: 'subflow',
    label: '子流程',
    icon: 'Workflow',
    category: 'control',
    color: '#6b7280',
  },
  waitElement: {
    type: 'waitElement',
    label: '等待元素',
    icon: 'Clock',
    category: 'action',
    color: '#3b82f6',
  },
  openBrowser: {
    type: 'openBrowser',
    label: '打开浏览器',
    icon: 'Globe',
    category: 'action',
    color: '#3b82f6',
  },
  navigate: {
    type: 'navigate',
    label: '导航网页',
    icon: 'Link',
    category: 'action',
    color: '#3b82f6',
  },
  screenshot: {
    type: 'screenshot',
    label: '截图',
    icon: 'Camera',
    category: 'action',
    color: '#3b82f6',
  },
  hotkey: {
    type: 'hotkey',
    label: '快捷键',
    icon: 'Keyboard',
    category: 'action',
    color: '#3b82f6',
  },
  webClick: {
    type: 'webClick',
    label: '网页点击',
    icon: 'MousePointer',
    category: 'action',
    color: '#22c55e',
  },
  webInput: {
    type: 'webInput',
    label: '网页输入',
    icon: 'PenLine',
    category: 'action',
    color: '#22c55e',
  },
  webGetText: {
    type: 'webGetText',
    label: '网页取值',
    icon: 'FileText',
    category: 'data',
    color: '#22c55e',
  },
  closeBrowser: {
    type: 'closeBrowser',
    label: '关闭浏览器',
    icon: 'X',
    category: 'action',
    color: '#ef4444',
  },
  executeJs: {
    type: 'executeJs',
    label: '执行JS',
    icon: 'Code',
    category: 'action',
    color: '#f59e0b',
  },
  readExcel: {
    type: 'readExcel',
    label: '读取Excel',
    icon: 'Table',
    category: 'data',
    color: '#22c55e',
  },
  writeExcel: {
    type: 'writeExcel',
    label: '写入Excel',
    icon: 'Table',
    category: 'data',
    color: '#22c55e',
  },
  executeCommand: {
    type: 'executeCommand',
    label: '执行命令',
    icon: 'Terminal',
    category: 'action',
    color: '#6b7280',
  },
  listDirectory: {
    type: 'listDirectory',
    label: '列出目录',
    icon: 'Files',
    category: 'data',
    color: '#22c55e',
  },
  openApp: {
    type: 'openApp',
    label: '打开应用',
    icon: 'AppWindow',
    category: 'action',
    color: '#3b82f6',
  },
};
