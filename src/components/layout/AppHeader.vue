<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { PenLine, Database, Play, Settings } from 'lucide-vue-next';

const router = useRouter();
const route = useRoute();

const navItems = [
  { path: '/designer', label: '流程设计', icon: PenLine },
  { path: '/elements', label: '元素库', icon: Database },
  { path: '/runner', label: '执行监控', icon: Play },
  { path: '/settings', label: '设置', icon: Settings },
];
</script>

<template>
  <header class="app-header">
    <div class="app-logo">
      <img src="/tauri.svg" alt="Logo" />
      <span>Batata RPA</span>
    </div>

    <nav class="app-nav">
      <div
        v-for="item in navItems"
        :key="item.path"
        class="nav-item"
        :class="{ 'nav-item-active': route.path === item.path }"
        @click="router.push(item.path)"
      >
        <component :is="item.icon" :size="18" />
        <span>{{ item.label }}</span>
      </div>
    </nav>
  </header>
</template>
