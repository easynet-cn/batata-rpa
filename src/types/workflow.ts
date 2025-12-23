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
  | 'writeExcel';

export interface NodeConfig {
  type: NodeType;
  label: string;
  icon: string;
  category: 'control' | 'action' | 'data';
  color: string;
}

export const NODE_CONFIGS: Record<NodeType, NodeConfig> = {
  start: {
    type: 'start',
    label: '开始',
    icon: 'CaretRight',
    category: 'control',
    color: '#67c23a',
  },
  end: {
    type: 'end',
    label: '结束',
    icon: 'Close',
    category: 'control',
    color: '#f56c6c',
  },
  click: {
    type: 'click',
    label: '点击',
    icon: 'Pointer',
    category: 'action',
    color: '#409eff',
  },
  input: {
    type: 'input',
    label: '输入',
    icon: 'Edit',
    category: 'action',
    color: '#409eff',
  },
  getText: {
    type: 'getText',
    label: '获取文本',
    icon: 'Document',
    category: 'data',
    color: '#e6a23c',
  },
  condition: {
    type: 'condition',
    label: '条件分支',
    icon: 'Switch',
    category: 'control',
    color: '#e6a23c',
  },
  loop: {
    type: 'loop',
    label: '循环',
    icon: 'Refresh',
    category: 'control',
    color: '#909399',
  },
  forEach: {
    type: 'forEach',
    label: '遍历',
    icon: 'Sort',
    category: 'control',
    color: '#909399',
  },
  delay: {
    type: 'delay',
    label: '延时',
    icon: 'Timer',
    category: 'action',
    color: '#409eff',
  },
  log: {
    type: 'log',
    label: '日志',
    icon: 'Tickets',
    category: 'data',
    color: '#e6a23c',
  },
  setVariable: {
    type: 'setVariable',
    label: '设置变量',
    icon: 'Coin',
    category: 'data',
    color: '#e6a23c',
  },
  readFile: {
    type: 'readFile',
    label: '读取文件',
    icon: 'FolderOpened',
    category: 'data',
    color: '#67c23a',
  },
  writeFile: {
    type: 'writeFile',
    label: '写入文件',
    icon: 'Folder',
    category: 'data',
    color: '#67c23a',
  },
  tryCatch: {
    type: 'tryCatch',
    label: '异常处理',
    icon: 'Warning',
    category: 'control',
    color: '#f56c6c',
  },
  subflow: {
    type: 'subflow',
    label: '子流程',
    icon: 'Connection',
    category: 'control',
    color: '#909399',
  },
  waitElement: {
    type: 'waitElement',
    label: '等待元素',
    icon: 'Clock',
    category: 'action',
    color: '#409eff',
  },
  openBrowser: {
    type: 'openBrowser',
    label: '打开浏览器',
    icon: 'Monitor',
    category: 'action',
    color: '#409eff',
  },
  navigate: {
    type: 'navigate',
    label: '导航网页',
    icon: 'Link',
    category: 'action',
    color: '#409eff',
  },
  screenshot: {
    type: 'screenshot',
    label: '截图',
    icon: 'Camera',
    category: 'action',
    color: '#409eff',
  },
  hotkey: {
    type: 'hotkey',
    label: '快捷键',
    icon: 'Key',
    category: 'action',
    color: '#409eff',
  },
  webClick: {
    type: 'webClick',
    label: '网页点击',
    icon: 'Pointer',
    category: 'action',
    color: '#67c23a',
  },
  webInput: {
    type: 'webInput',
    label: '网页输入',
    icon: 'Edit',
    category: 'action',
    color: '#67c23a',
  },
  webGetText: {
    type: 'webGetText',
    label: '网页取值',
    icon: 'Document',
    category: 'data',
    color: '#67c23a',
  },
  closeBrowser: {
    type: 'closeBrowser',
    label: '关闭浏览器',
    icon: 'Close',
    category: 'action',
    color: '#f56c6c',
  },
  executeJs: {
    type: 'executeJs',
    label: '执行JS',
    icon: 'Promotion',
    category: 'action',
    color: '#e6a23c',
  },
  readExcel: {
    type: 'readExcel',
    label: '读取Excel',
    icon: 'Document',
    category: 'data',
    color: '#67c23a',
  },
  writeExcel: {
    type: 'writeExcel',
    label: '写入Excel',
    icon: 'Document',
    category: 'data',
    color: '#67c23a',
  },
};
