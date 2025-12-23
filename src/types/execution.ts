export type ExecutionStatus =
  | 'idle'
  | 'running'
  | 'paused'
  | 'completed'
  | 'failed';

export interface ExecutionLog {
  id: string;
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  nodeId?: string;
  message: string;
  details?: unknown;
}

export interface ExecutionState {
  workflowId: string;
  status: ExecutionStatus;
  currentNodeId?: string;
  startTime?: string;
  endTime?: string;
  logs: ExecutionLog[];
  variables: Record<string, unknown>;
  error?: string;
}

export interface ExecutionResult {
  success: boolean;
  duration: number;
  logs: ExecutionLog[];
  error?: string;
}
