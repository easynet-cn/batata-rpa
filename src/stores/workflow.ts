import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Workflow, WorkflowNode, WorkflowEdge, Variable } from '@/types';

export const useWorkflowStore = defineStore('workflow', () => {
  const workflows = ref<Workflow[]>([]);
  const currentWorkflow = ref<Workflow | null>(null);
  const selectedNodeId = ref<string | null>(null);

  const currentNodes = computed({
    get: () => currentWorkflow.value?.nodes ?? [],
    set: (nodes) => {
      if (currentWorkflow.value) {
        currentWorkflow.value.nodes = nodes;
        currentWorkflow.value.updatedAt = new Date().toISOString();
      }
    },
  });

  const currentEdges = computed({
    get: () => currentWorkflow.value?.edges ?? [],
    set: (edges) => {
      if (currentWorkflow.value) {
        currentWorkflow.value.edges = edges;
        currentWorkflow.value.updatedAt = new Date().toISOString();
      }
    },
  });
  const selectedNode = computed(() =>
    currentNodes.value.find((n) => n.id === selectedNodeId.value)
  );

  function createWorkflow(name: string): Workflow {
    const now = new Date().toISOString();
    const workflow: Workflow = {
      id: crypto.randomUUID(),
      name,
      nodes: [
        {
          id: 'start-1',
          type: 'start',
          position: { x: 250, y: 50 },
          data: {},
          label: '开始',
        },
      ],
      edges: [],
      variables: [],
      createdAt: now,
      updatedAt: now,
    };
    workflows.value.push(workflow);
    currentWorkflow.value = workflow;
    return workflow;
  }

  function openWorkflow(id: string) {
    const workflow = workflows.value.find((w) => w.id === id);
    if (workflow) {
      currentWorkflow.value = workflow;
    }
  }

  function addNode(node: WorkflowNode) {
    if (currentWorkflow.value) {
      // 使用新数组确保 Vue 响应式更新
      currentWorkflow.value.nodes = [...currentWorkflow.value.nodes, node];
      currentWorkflow.value.updatedAt = new Date().toISOString();
    }
  }

  function updateNode(id: string, data: Partial<WorkflowNode>) {
    if (currentWorkflow.value) {
      const index = currentWorkflow.value.nodes.findIndex((n) => n.id === id);
      if (index !== -1) {
        currentWorkflow.value.nodes[index] = {
          ...currentWorkflow.value.nodes[index],
          ...data,
        };
        currentWorkflow.value.updatedAt = new Date().toISOString();
      }
    }
  }

  function removeNode(id: string) {
    if (currentWorkflow.value) {
      currentWorkflow.value.nodes = currentWorkflow.value.nodes.filter(
        (n) => n.id !== id
      );
      currentWorkflow.value.edges = currentWorkflow.value.edges.filter(
        (e) => e.source !== id && e.target !== id
      );
      currentWorkflow.value.updatedAt = new Date().toISOString();
      if (selectedNodeId.value === id) {
        selectedNodeId.value = null;
      }
    }
  }

  function addEdge(edge: WorkflowEdge) {
    if (currentWorkflow.value) {
      const exists = currentWorkflow.value.edges.some(
        (e) => e.source === edge.source && e.target === edge.target
      );
      if (!exists) {
        currentWorkflow.value.edges.push(edge);
        currentWorkflow.value.updatedAt = new Date().toISOString();
      }
    }
  }

  function removeEdge(id: string) {
    if (currentWorkflow.value) {
      currentWorkflow.value.edges = currentWorkflow.value.edges.filter(
        (e) => e.id !== id
      );
      currentWorkflow.value.updatedAt = new Date().toISOString();
    }
  }

  function addVariable(variable: Variable) {
    if (currentWorkflow.value) {
      currentWorkflow.value.variables.push(variable);
      currentWorkflow.value.updatedAt = new Date().toISOString();
    }
  }

  function updateVariable(id: string, data: Partial<Variable>) {
    if (currentWorkflow.value) {
      const index = currentWorkflow.value.variables.findIndex((v) => v.id === id);
      if (index !== -1) {
        currentWorkflow.value.variables[index] = {
          ...currentWorkflow.value.variables[index],
          ...data,
        };
        currentWorkflow.value.updatedAt = new Date().toISOString();
      }
    }
  }

  function removeVariable(id: string) {
    if (currentWorkflow.value) {
      currentWorkflow.value.variables = currentWorkflow.value.variables.filter(
        (v) => v.id !== id
      );
      currentWorkflow.value.updatedAt = new Date().toISOString();
    }
  }

  function selectNode(id: string | null) {
    selectedNodeId.value = id;
  }

  function saveToJson(): string {
    if (currentWorkflow.value) {
      return JSON.stringify(currentWorkflow.value, null, 2);
    }
    return '';
  }

  function loadFromJson(json: string) {
    try {
      const workflow = JSON.parse(json) as Workflow;
      const existing = workflows.value.findIndex((w) => w.id === workflow.id);
      if (existing !== -1) {
        workflows.value[existing] = workflow;
      } else {
        workflows.value.push(workflow);
      }
      currentWorkflow.value = workflow;
    } catch (e) {
      console.error('Failed to parse workflow JSON:', e);
    }
  }

  return {
    workflows,
    currentWorkflow,
    selectedNodeId,
    currentNodes,
    currentEdges,
    selectedNode,
    createWorkflow,
    openWorkflow,
    addNode,
    updateNode,
    removeNode,
    addEdge,
    removeEdge,
    addVariable,
    updateVariable,
    removeVariable,
    selectNode,
    saveToJson,
    loadFromJson,
  };
});
