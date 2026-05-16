<template>
  <div class="app-layout">
    <aside class="app-sidebar panel" :class="{ collapsed: isCollapsed }">
      <div class="sidebar-brand">
        <div class="sidebar-title">FFS</div>
        <div v-if="!isCollapsed" class="sidebar-subtitle">家庭财务系统</div>
      </div>

      <el-menu
        :default-active="activePath"
        :default-openeds="defaultOpeneds"
        :collapse="isCollapsed"
        class="sidebar-menu"
        router
      >
        <el-menu-item index="/">
          <el-icon><House /></el-icon>
          <span>首页</span>
        </el-menu-item>

        <el-sub-menu index="finance">
          <template #title>
            <el-icon><Wallet /></el-icon>
            <span>财务管理</span>
          </template>
          <el-menu-item index="/bills">账单</el-menu-item>
          <el-menu-item index="/debts">债务</el-menu-item>
          <el-menu-item index="/presales">预售</el-menu-item>
          <el-menu-item index="/assets">资产</el-menu-item>
          <el-menu-item index="/investments">投资仪表盘</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="ops">
          <template #title>
            <el-icon><DataAnalysis /></el-icon>
            <span>运营配置</span>
          </template>
          <el-menu-item index="/strategies">策略</el-menu-item>
          <el-menu-item index="/intel">情报</el-menu-item>
          <el-menu-item index="/balance-calibrations">余额校准</el-menu-item>
          <el-menu-item index="/budgets">预算</el-menu-item>
          <el-menu-item index="/brands">品牌</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="system">
          <template #title>
            <el-icon><Setting /></el-icon>
            <span>系统</span>
          </template>
          <el-menu-item index="/jobs">任务</el-menu-item>
          <el-menu-item index="/job-runs">任务日志</el-menu-item>
          <el-menu-item index="/users">用户</el-menu-item>
          <el-menu-item index="/config">配置项</el-menu-item>
          <el-menu-item index="/credit-cards">信用卡</el-menu-item>
          <el-menu-item index="/migration-audit">迁移审计</el-menu-item>
        </el-sub-menu>
      </el-menu>

      <div class="sidebar-footer">
        <template v-if="!isCollapsed">
          <div class="sidebar-user-label">
            <el-icon><UserFilled /></el-icon>
            <span>当前用户</span>
          </div>
          <div class="sidebar-user-name">{{ authStore.user?.display_name || '未登录' }}</div>
          <el-button class="sidebar-logout" @click="handleLogout">
            <el-icon><SwitchButton /></el-icon>
            <span>退出登录</span>
          </el-button>
        </template>
        <el-button text class="collapse-btn" @click="isCollapsed = !isCollapsed">
          <el-icon>
            <Expand v-if="isCollapsed" />
            <Fold v-else />
          </el-icon>
          <span v-if="!isCollapsed">{{ isCollapsed ? '展开菜单' : '折叠菜单' }}</span>
        </el-button>
      </div>
    </aside>

    <div class="app-main">
      <main class="page-content">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  DataAnalysis,
  Expand,
  Fold,
  House,
  Setting,
  SwitchButton,
  UserFilled,
  Wallet
} from '@element-plus/icons-vue'
import { logout } from '@/api/auth'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const isCollapsed = ref(true)

const activePath = computed(() => route.path)
const defaultOpeneds = ['finance', 'ops', 'system']

async function handleLogout() {
  try {
    await logout()
  } catch {
    // ignore
  }
  authStore.clear()
  await router.push('/login')
  ElMessage.success('已退出登录')
}
</script>

<style scoped>
.app-layout {
  display: flex;
  min-height: 100vh;
  gap: 20px;
  padding: 20px;
}

.app-sidebar {
  width: 300px;
  min-width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 20px 16px;
  overflow: hidden;
}

.app-sidebar.collapsed {
  width: 96px;
  min-width: 96px;
  padding-inline: 10px;
}

.sidebar-brand {
  padding: 8px 8px 18px;
}

.sidebar-title {
  font-size: 28px;
  font-weight: 800;
}

.sidebar-subtitle {
  margin-top: 6px;
  color: #64748b;
  font-size: 13px;
}

.sidebar-menu {
  flex: 1;
  border-right: none;
  background: transparent;
}

.sidebar-menu :deep(.el-menu-item .el-icon),
.sidebar-menu :deep(.el-sub-menu__title .el-icon) {
  margin-right: 10px;
}

.sidebar-menu :deep(.el-menu) {
  border-right: none;
}

.app-sidebar.collapsed .sidebar-menu :deep(.el-sub-menu .el-sub-menu__title),
.app-sidebar.collapsed .sidebar-menu :deep(.el-menu-item) {
  justify-content: center;
}

.app-sidebar.collapsed .sidebar-menu :deep(.el-menu-item .el-icon),
.app-sidebar.collapsed .sidebar-menu :deep(.el-sub-menu__title .el-icon) {
  margin-right: 0;
}

.sidebar-footer {
  padding: 12px 8px 2px;
  border-top: 1px solid rgba(148, 163, 184, 0.2);
}

.collapse-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.app-sidebar.collapsed .collapse-btn {
  width: 100%;
  justify-content: center;
}

.sidebar-user-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: #64748b;
  font-size: 12px;
}

.sidebar-user-name {
  margin-top: 6px;
  font-size: 15px;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-logout {
  width: 100%;
  margin-top: 10px;
}

.app-main {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
}

@media (max-width: 1280px) {
  .app-sidebar {
    width: 280px;
    min-width: 280px;
  }
}

@media (max-width: 900px) {
  .app-layout {
    padding: 12px;
    gap: 12px;
  }

  .app-sidebar {
    width: 250px;
    min-width: 250px;
  }
}
</style>
