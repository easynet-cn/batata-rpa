<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import {
  Edit,
  Collection,
  VideoPlay,
  Setting,
} from '@element-plus/icons-vue';

const router = useRouter();
const route = useRoute();

const navItems = [
  { path: '/designer', label: '流程设计', icon: Edit },
  { path: '/elements', label: '元素库', icon: Collection },
  { path: '/runner', label: '执行监控', icon: VideoPlay },
  { path: '/settings', label: '设置', icon: Setting },
];
</script>

<template>
  <header class="app-header">
    <div class="app-logo">
      <img src="/tauri.svg" alt="Logo" />
      <span>Batata RPA</span>
    </div>

    <el-menu
      :default-active="route.path"
      mode="horizontal"
      class="app-nav"
      :ellipsis="false"
      @select="(path: string) => router.push(path)"
    >
      <el-menu-item v-for="item in navItems" :key="item.path" :index="item.path">
        <el-icon><component :is="item.icon" /></el-icon>
        <span>{{ item.label }}</span>
      </el-menu-item>
    </el-menu>
  </header>
</template>

<style scoped>
.app-nav {
  border-bottom: none !important;
  flex: 1;
}

.app-nav .el-menu-item {
  height: 50px;
  line-height: 50px;
}
</style>
