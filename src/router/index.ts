import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      redirect: '/designer',
    },
    {
      path: '/designer',
      name: 'designer',
      component: () => import('@/views/Designer.vue'),
      meta: { title: '流程设计器' },
    },
    {
      path: '/elements',
      name: 'elements',
      component: () => import('@/views/ElementLibrary.vue'),
      meta: { title: '元素库' },
    },
    {
      path: '/runner',
      name: 'runner',
      component: () => import('@/views/Runner.vue'),
      meta: { title: '执行监控' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
      meta: { title: '设置' },
    },
  ],
});

export default router;
