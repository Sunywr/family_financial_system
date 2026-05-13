import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue')
    },
    {
      path: '/',
      component: () => import('@/layouts/AppLayout.vue'),
      children: [
        {
          path: '',
          name: 'dashboard',
          component: () => import('@/views/DashboardView.vue')
        },
        {
          path: 'bills',
          name: 'bills',
          component: () => import('@/views/BillsView.vue')
        },
        {
          path: 'debts',
          name: 'debts',
          component: () => import('@/views/DebtsView.vue')
        },
        {
          path: 'presales',
          name: 'presales',
          component: () => import('@/views/PresalesView.vue')
        },
        {
          path: 'assets',
          name: 'assets',
          component: () => import('@/views/AssetsView.vue')
        },
        {
          path: 'investments',
          name: 'investments',
          component: () => import('@/views/InvestmentsView.vue')
        },
        {
          path: 'investment-transactions',
          name: 'investmentTransactions',
          component: () => import('@/views/InvestmentTransactionsView.vue')
        },
        {
          path: 'investment-top',
          name: 'investmentTop',
          component: () => import('@/views/InvestmentTopView.vue')
        },
        {
          path: 'strategies',
          name: 'strategies',
          component: () => import('@/views/StrategyConfigsView.vue')
        },
        {
          path: 'intel',
          name: 'intel',
          component: () => import('@/views/IntelView.vue')
        },
        {
          path: 'migration-audit',
          name: 'migrationAudit',
          component: () => import('@/views/MigrationAuditView.vue')
        },
        {
          path: 'balance-calibrations',
          name: 'balanceCalibrations',
          component: () => import('@/views/BalanceCalibrationsView.vue')
        },
        {
          path: 'budgets',
          name: 'budgets',
          component: () => import('@/views/BudgetsView.vue')
        },
        {
          path: 'brands',
          name: 'brands',
          component: () => import('@/views/BrandsView.vue')
        },
        {
          path: 'jobs',
          name: 'jobs',
          component: () => import('@/views/JobsView.vue')
        },
        {
          path: 'job-runs',
          name: 'jobRuns',
          component: () => import('@/views/JobRunsView.vue')
        },
        {
          path: 'users',
          name: 'users',
          component: () => import('@/views/UsersView.vue')
        },
        {
          path: 'config',
          name: 'config',
          component: () => import('@/views/ConfigView.vue')
        },
        {
          path: 'credit-cards',
          name: 'creditCards',
          component: () => import('@/views/CreditCardsView.vue')
        }
      ]
    }
  ]
})

router.beforeEach((to) => {
  const authStore = useAuthStore()
  if (to.name === 'login') {
    return authStore.token ? '/' : true
  }
  return authStore.token ? true : '/login'
})

export default router
