<template>
  <div class="app-layout" :class="{ 'mobile-menu-open': mobileMenuOpen }">
    <button class="mobile-backdrop" type="button" aria-label="关闭菜单" @click="closeMobileMenu"></button>

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
          <el-menu-item index="/auto-invest-plans">定投计划</el-menu-item>
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
      <header class="mobile-topbar panel">
        <div class="mobile-topbar-left">
          <el-button text class="mobile-menu-btn" @click="toggleMobileMenu">
            <el-icon><Menu /></el-icon>
          </el-button>
          <div class="mobile-title-group">
            <div class="mobile-title">{{ currentSectionTitle }}</div>
            <div class="mobile-subtitle">{{ authStore.user?.display_name || '未登录' }}</div>
          </div>
        </div>
      </header>

      <main class="page-content">
        <RouterView v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <component :is="Component" :key="$route.path" />
          </Transition>
        </RouterView>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  DataAnalysis,
  Expand,
  Fold,
  House,
  Menu,
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
const mobileMenuOpen = ref(false)

const activePath = computed(() => route.path)
const defaultOpeneds = ['finance', 'ops', 'system']
const routeTitleMap: Record<string, string> = {
  '/': '首页',
  '/bills': '账单',
  '/debts': '债务',
  '/presales': '预售',
  '/assets': '资产',
  '/investments': '投资仪表盘',
  '/investment-transactions': '投资流水',
  '/investment-top': '投资建议',
  '/strategies': '策略',
  '/intel': '情报',
  '/balance-calibrations': '余额校准',
  '/budgets': '预算',
  '/brands': '品牌',
  '/jobs': '任务',
  '/job-runs': '任务日志',
  '/users': '用户',
  '/config': '配置项',
  '/credit-cards': '信用卡',
  '/migration-audit': '迁移审计'
}

const currentSectionTitle = computed(() => routeTitleMap[route.path] || '家庭财务系统')

function closeMobileMenu() {
  mobileMenuOpen.value = false
}

function toggleMobileMenu() {
  if (window.innerWidth <= 900) {
    isCollapsed.value = false
  }
  mobileMenuOpen.value = !mobileMenuOpen.value
}

function syncSidebarMode() {
  if (window.innerWidth <= 900) {
    isCollapsed.value = false
    return
  }
  isCollapsed.value = true
}

onMounted(() => {
  syncSidebarMode()
  window.addEventListener('resize', syncSidebarMode)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', syncSidebarMode)
})

watch(
  () => route.fullPath,
  () => {
    mobileMenuOpen.value = false
  }
)

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
  position: relative;
  display: flex;
  min-height: 100vh;
  gap: 20px;
  padding: 20px;
}

.app-layout::before,
.app-layout::after {
  content: "";
  position: fixed;
  pointer-events: none;
  z-index: 0;
  filter: blur(10px);
}

.app-layout::before {
  top: 8%;
  left: -120px;
  width: 360px;
  height: 360px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 187, 94, 0.18) 0%, rgba(255, 187, 94, 0) 72%);
  animation: floatHalo 16s ease-in-out infinite alternate;
}

.app-layout::after {
  right: -100px;
  bottom: 4%;
  width: 320px;
  height: 320px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(76, 132, 255, 0.16) 0%, rgba(76, 132, 255, 0) 72%);
  animation: floatHalo 18s ease-in-out infinite alternate-reverse;
}

.mobile-backdrop {
  display: none;
}

.app-sidebar {
  position: relative;
  z-index: 1;
  width: 300px;
  min-width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 20px 16px;
  overflow: hidden;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.86) 0%, rgba(243, 247, 252, 0.75) 100%),
    linear-gradient(135deg, rgba(76, 132, 255, 0.08), rgba(255, 187, 94, 0.08));
  border: 1px solid rgba(255, 255, 255, 0.52);
  box-shadow: 0 28px 54px rgba(15, 23, 42, 0.12);
}

.app-sidebar::before {
  content: "";
  position: absolute;
  inset: -10% auto auto -18%;
  width: 180px;
  height: 180px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 191, 102, 0.16) 0%, rgba(255, 191, 102, 0) 72%);
  animation: pulseGlow 10s ease-in-out infinite alternate;
}

.app-sidebar.collapsed {
  width: 96px;
  min-width: 96px;
  padding-inline: 10px;
}

.sidebar-brand {
  position: relative;
  padding: 8px 8px 18px;
}

.sidebar-title {
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.08em;
  background: linear-gradient(135deg, #1f3d67 0%, #355c92 52%, #987247 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
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

.sidebar-menu :deep(.el-menu-item),
.sidebar-menu :deep(.el-sub-menu__title) {
  margin-bottom: 6px;
  border: 1px solid transparent;
  border-radius: 14px;
  transition: transform 0.22s ease, background-color 0.22s ease, border-color 0.22s ease, box-shadow 0.22s ease;
}

.sidebar-menu :deep(.el-menu-item:hover),
.sidebar-menu :deep(.el-sub-menu__title:hover) {
  transform: translateX(4px);
  background: linear-gradient(135deg, rgba(76, 132, 255, 0.1), rgba(255, 190, 102, 0.12));
  border-color: rgba(148, 163, 184, 0.18);
  box-shadow: 0 12px 20px rgba(15, 23, 42, 0.06);
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  color: #1d3d6b;
  background: linear-gradient(135deg, rgba(66, 120, 226, 0.18), rgba(255, 196, 116, 0.18));
  border-color: rgba(66, 120, 226, 0.18);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.55);
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
  background: linear-gradient(180deg, rgba(255, 255, 255, 0), rgba(255, 255, 255, 0.28));
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
  position: relative;
  z-index: 1;
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.mobile-topbar {
  display: none;
}

.mobile-topbar-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.mobile-menu-btn {
  display: inline-flex;
}

.mobile-title-group {
  display: flex;
  flex-direction: column;
}

.mobile-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  line-height: 1.2;
}

.mobile-subtitle {
  margin-top: 2px;
  font-size: 12px;
  color: #64748b;
}

.page-content {
  flex: 1;
  position: relative;
}

.page-content::before {
  content: "";
  position: absolute;
  inset: 8px 6% auto auto;
  width: 180px;
  height: 180px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 203, 129, 0.12) 0%, rgba(255, 203, 129, 0) 72%);
  pointer-events: none;
  filter: blur(8px);
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
    gap: 0;
  }

  .app-sidebar {
    position: fixed;
    top: 10px;
    bottom: 10px;
    left: 10px;
    z-index: 32;
    width: min(82vw, 320px);
    min-width: min(82vw, 320px);
    border-radius: 16px;
    transform: translateX(calc(-100% - 16px));
    transition: transform 0.26s cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow: 0 18px 48px rgba(15, 23, 42, 0.22);
  }

  .app-main {
    width: 100%;
  }

  .mobile-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding: 8px 10px;
    border-radius: 14px;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.88), rgba(244, 248, 252, 0.7)),
      linear-gradient(135deg, rgba(76, 132, 255, 0.08), rgba(255, 190, 102, 0.08));
  }

  .mobile-menu-open .app-sidebar {
    transform: translateX(0);
  }

  .mobile-backdrop {
    display: block;
    position: fixed;
    inset: 0;
    z-index: 30;
    border: 0;
    padding: 0;
    margin: 0;
    background: linear-gradient(180deg, rgba(15, 23, 42, 0.16), rgba(15, 23, 42, 0.34));
    opacity: 0;
    pointer-events: none;
    backdrop-filter: blur(4px);
    transition: opacity 0.22s ease;
  }

  .mobile-menu-open .mobile-backdrop {
    opacity: 1;
    pointer-events: auto;
  }

  .app-sidebar.collapsed {
    width: min(82vw, 320px);
    min-width: min(82vw, 320px);
    padding-inline: 16px;
  }

  .app-sidebar.collapsed .sidebar-brand,
  .app-sidebar.collapsed .sidebar-footer {
    padding-inline: 8px;
  }

  .app-sidebar.collapsed .sidebar-subtitle,
  .app-sidebar.collapsed .sidebar-user-label,
  .app-sidebar.collapsed .sidebar-user-name,
  .app-sidebar.collapsed .sidebar-logout,
  .app-sidebar.collapsed .collapse-btn span,
  .app-sidebar.collapsed .sidebar-menu :deep(.el-sub-menu__title span),
  .app-sidebar.collapsed .sidebar-menu :deep(.el-menu-item span) {
    display: inline;
  }

  .app-sidebar.collapsed .sidebar-menu :deep(.el-menu-item),
  .app-sidebar.collapsed .sidebar-menu :deep(.el-sub-menu__title),
  .app-sidebar.collapsed .collapse-btn {
    justify-content: flex-start;
  }

  .app-sidebar.collapsed .sidebar-menu :deep(.el-menu-item .el-icon),
  .app-sidebar.collapsed .sidebar-menu :deep(.el-sub-menu__title .el-icon) {
    margin-right: 10px;
  }
}

@keyframes floatHalo {
  from {
    transform: translate3d(0, 0, 0) scale(1);
  }
  to {
    transform: translate3d(18px, -20px, 0) scale(1.08);
  }
}

@keyframes pulseGlow {
  from {
    opacity: 0.42;
    transform: scale(1);
  }
  to {
    opacity: 0.9;
    transform: scale(1.12);
  }
}
</style>
