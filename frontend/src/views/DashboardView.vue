<template>
  <section>
    <div class="panel dashboard-header">
      <div class="dashboard-header-main">
        <div>
          <h2 class="dashboard-title">首页总览</h2>
          <p class="dashboard-subtitle">统计范围：{{ range.start_date }} 至 {{ range.end_date }}</p>
        </div>
        <el-date-picker
          v-model="dateRange"
          type="daterange"
          unlink-panels
          range-separator="至"
          start-placeholder="开始日期"
          end-placeholder="结束日期"
          value-format="YYYY-MM-DD"
        />
      </div>
      <el-button type="primary" @click="applyRange">刷新</el-button>
    </div>

    <div v-loading="loading" class="metric-grid dashboard-metrics">
      <div class="panel metric-card">
        <div class="metric-label">现金余额</div>
        <div class="metric-value">{{ summary?.cash_balance ?? '--' }}</div>
      </div>
      <div class="panel metric-card">
        <div class="metric-label">家庭总资产</div>
        <div class="metric-value">{{ summary?.total_assets ?? '--' }}</div>
      </div>
      <div class="panel metric-card">
        <div class="metric-label">理财总市值</div>
        <div class="metric-value">{{ summary?.wealth_amount ?? '--' }}</div>
      </div>
      <div class="panel metric-card">
        <div class="metric-label">股票总资产</div>
        <div class="metric-value">{{ summary?.stock_amount ?? '--' }}</div>
      </div>
    </div>

    <div v-loading="chartLoading" class="panel chart-panel">
      <div class="chart-head">
        <h3 class="chart-title">现金余额趋势</h3>
        <el-switch
          v-model="useProvidentFundForMortgage"
          active-text="公积金冲抵房贷"
          @change="handleMortgageSettingChange"
        />
      </div>
      <div ref="chartRef" class="chart-body"></div>
    </div>

    <div class="grid-two">
      <div class="panel section-panel">
        <h3>发薪日前准备</h3>
        <div v-if="salaryPrepCard" class="summary-block">
          <div class="summary-hero">
            <div>
              <div class="summary-label">距下次发薪</div>
              <div class="summary-value">{{ salaryPrepCard.daysUntilSalary }}</div>
            </div>
            <div>
              <div class="summary-label">目标发薪日</div>
              <div class="summary-value summary-value-sm">{{ salaryPrepCard.salaryTargetDate }}</div>
            </div>
          </div>
          <div class="summary-grid">
            <div class="summary-stat">
              <span>发薪日前应还</span>
              <strong>{{ salaryPrepCard.dueBeforeSalary }}</strong>
            </div>
            <div class="summary-stat">
              <span>窗口期应还</span>
              <strong>{{ salaryPrepCard.windowDue }}</strong>
            </div>
            <div class="summary-stat">
              <span>信用卡笔数</span>
              <strong>{{ salaryPrepCard.creditCount }}</strong>
            </div>
            <div class="summary-stat">
              <span>周期债务笔数</span>
              <strong>{{ salaryPrepCard.cycleCount }}</strong>
            </div>
          </div>
          <div class="summary-meta">
            统计窗口：{{ salaryPrepCard.windowStart }} 至 {{ salaryPrepCard.windowEnd }}
          </div>
        </div>
        <div v-else class="empty-tip">暂无数据</div>
      </div>

      <div class="panel section-panel">
        <h3>投资持仓摘要</h3>
        <div v-if="positionSummaryCard" class="summary-block">
          <div class="summary-hero">
            <div>
              <div class="summary-label">总投资额</div>
              <div class="summary-value">{{ positionSummaryCard.totalInvestment }}</div>
            </div>
            <div>
              <div class="summary-label">总收益</div>
              <div class="summary-value summary-value-sm">{{ positionSummaryCard.totalProfit }}</div>
            </div>
          </div>
          <div class="summary-grid">
            <div class="summary-stat">
              <span>持仓收益</span>
              <strong>{{ positionSummaryCard.holdingProfit }}</strong>
            </div>
            <div class="summary-stat">
              <span>平均收益率</span>
              <strong>{{ positionSummaryCard.avgProfitRate }}</strong>
            </div>
            <div class="summary-stat">
              <span>个人理财市值</span>
              <strong>{{ positionSummaryCard.personalWealth }}</strong>
            </div>
            <div class="summary-stat">
              <span>个人股票总资产</span>
              <strong>{{ positionSummaryCard.personalStock }}</strong>
            </div>
          </div>
          <div class="summary-meta">
            理财平均年化：{{ positionSummaryCard.avgAnnualRate }}，股票账户可用现金：{{ positionSummaryCard.stockIdleCash }}
          </div>
          <div class="summary-actions">
            <el-button text type="primary" @click="openHomepageInvestmentDrawer('stock')">查看股票持仓</el-button>
            <el-button text type="primary" @click="openHomepageInvestmentDrawer('wealth')">查看理财持仓</el-button>
            <el-button text type="primary" @click="openHomepageInvestmentDrawer('transactions')">股票流水</el-button>
            <el-button text type="primary" @click="openHomepageInvestmentDrawer('top20')">TOP20 建议</el-button>
          </div>
        </div>
        <div v-else class="empty-tip">暂无数据</div>
      </div>


      <div class="panel section-panel">
        <h3>信用卡待还</h3>
        <div v-if="isMobileView" class="mobile-cards-mini">
          <div v-for="row in creditCardRows" :key="row.id" class="mobile-card-mini panel">
            <div class="mobile-card-mini-head">
              <strong>{{ row.name }}</strong>
              <span>{{ row.due_date }}</span>
            </div>
            <div class="mobile-card-mini-content">
              <span>金额</span>
              <strong>{{ row.amount }}</strong>
            </div>
            <el-button class="mobile-card-mini-action" size="small" type="primary" @click="openCreditCardBillRow(row)">对账</el-button>
          </div>
          <div v-if="!creditCardRows.length" class="empty-tip">暂无信用卡待还项</div>
        </div>
        <el-table v-else-if="creditCardRows.length" :data="creditCardRows" size="small" stripe>
          <el-table-column prop="name" label="名称" min-width="160" />
          <el-table-column prop="due_date" label="到期日" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column label="操作" width="110">
            <template #default="{ row }">
              <el-button text type="primary" @click="openCreditCardBillRow(row)">对账</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="empty-tip">暂无信用卡待还项</div>
      </div>

      <div class="panel section-panel">
        <h3>周期债务待还</h3>
        <div v-if="isMobileView" class="mobile-cards-mini">
          <div v-for="row in cycleDebtRows" :key="row.id" class="mobile-card-mini panel">
            <div class="mobile-card-mini-head">
              <strong>{{ row.name }}</strong>
              <span>{{ row.due_date }}</span>
            </div>
            <div class="mobile-card-mini-content">
              <span>金额</span>
              <strong>{{ row.amount }}</strong>
            </div>
            <div class="mobile-card-mini-content">
              <span>已还</span>
              <span>{{ row.paid_period_count }}/{{ row.required_period_count }}</span>
            </div>
            <el-button class="mobile-card-mini-action" size="small" type="primary" @click="openCycleDebtRow(row)">查看明细</el-button>
          </div>
          <div v-if="!cycleDebtRows.length" class="empty-tip">暂无周期债务待还项</div>
        </div>
        <el-table v-else-if="cycleDebtRows.length" :data="cycleDebtRows" size="small" stripe>
          <el-table-column prop="name" label="名称" min-width="160" />
          <el-table-column prop="due_date" label="到期日" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column prop="required_period_count" label="总期数" width="90" />
          <el-table-column prop="paid_period_count" label="已还期数" width="90" />
          <el-table-column label="操作" width="110">
            <template #default="{ row }">
              <el-button text type="primary" @click="openCycleDebtRow(row)">查看明细</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="empty-tip">暂无周期债务待还项</div>
      </div>

    </div>

    <el-drawer v-model="investmentDrawerVisible" :title="investmentDrawerTitle" size="80%">
      <el-table
        v-if="investmentDrawerMode === 'stock' || investmentDrawerMode === 'wealth'"
        :data="investmentDrawerInvestments"
        stripe
      >
        <el-table-column prop="id" label="ID" width="90" />
        <el-table-column prop="name" label="名称" min-width="200" />
        <el-table-column prop="code" label="代码" width="130" />
        <el-table-column prop="market_value" label="市值" width="120" />
        <el-table-column label="收益金额" width="120">
          <template #default="{ row }">
            <span :class="profitColorClass(toNumber(row.total_profit))">{{ row.total_profit }}</span>
          </template>
        </el-table-column>
        <el-table-column label="收益率" width="120">
          <template #default="{ row }">
            <span :class="profitColorClass(toNumber(row.total_profit_rate))">
              {{ `${(toNumber(row.total_profit_rate) * 100).toFixed(2)}%` }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="240" fixed="right">
          <template #default="{ row }">
            <div class="action-inline">
              <el-button text type="primary" @click="openInvestmentActionDialog('open_position', row)">建仓</el-button>
              <el-button text type="success" @click="openInvestmentActionDialog('add_position', row)">加仓</el-button>
              <el-button text type="danger" @click="openInvestmentActionDialog('reduce_position', row)">减仓</el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <template v-else-if="investmentDrawerMode === 'transactions'">
        <div class="drawer-toolbar">
          <el-input
            v-model="investmentDrawerTransactionsKeyword"
            placeholder="全文搜索（动作/备注/投资名称/来源账单ID）"
            style="width: 320px"
            @keyup.enter="searchInvestmentDrawerTransactions"
          />
          <el-button @click="searchInvestmentDrawerTransactions">查询</el-button>
        </div>
        <el-table :data="investmentDrawerTransactions.list" stripe>
          <el-table-column prop="id" label="ID" width="90" />
          <el-table-column label="投资" min-width="220">
            <template #default="{ row }">
              {{ row.investment_name || '--' }}（{{ row.investment_code || '--' }}）
            </template>
          </el-table-column>
          <el-table-column prop="source_bill_id" label="来源账单ID" width="120" />
          <el-table-column prop="transaction_date" label="日期" width="120" />
          <el-table-column prop="action" label="动作" width="120" />
          <el-table-column prop="shares" label="份额" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column prop="remark" label="备注" min-width="240" show-overflow-tooltip />
        </el-table>
        <div class="pagination-wrap">
          <el-pagination
            :current-page="investmentDrawerTransactionsPage"
            :page-size="investmentDrawerTransactionsPageSize"
            :page-sizes="[20, 50, 100]"
            :total="investmentDrawerTransactions.total"
            layout="total, sizes, prev, pager, next"
            @current-change="onInvestmentDrawerTransactionsPageChange"
            @size-change="onInvestmentDrawerTransactionsPageSizeChange"
          />
        </div>
      </template>

      <template v-else>
        <div class="drawer-toolbar">
          <el-select
            v-model="investmentDrawerTopType"
            style="width: 160px"
            @change="loadInvestmentDrawerTopRecommendations"
          >
            <el-option label="全部" value="" />
            <el-option label="股票" value="stock" />
            <el-option label="理财" value="wealth" />
          </el-select>
          <el-input
            v-model="investmentDrawerTopKeyword"
            placeholder="全文搜索（名称/代码/ID/原因）"
            style="width: 320px"
            @keyup.enter="applyInvestmentDrawerTopKeyword"
          />
          <el-button @click="applyInvestmentDrawerTopKeyword">查询</el-button>
        </div>
        <el-table :data="investmentDrawerPagedTopRecommendations" stripe>
          <el-table-column prop="investment_id" label="ID" width="90" />
          <el-table-column label="类型" width="110">
            <template #default="{ row }">
              {{ row.investment_type === 'stock' ? '股票' : '理财' }}
            </template>
          </el-table-column>
          <el-table-column prop="name" label="名称" min-width="220" />
          <el-table-column prop="code" label="代码" width="130" />
          <el-table-column prop="score" label="评分" width="90" />
          <el-table-column prop="suggestion" label="建议" width="120" />
          <el-table-column prop="reason" label="原因" min-width="280" show-overflow-tooltip />
        </el-table>
        <div class="pagination-wrap">
          <el-pagination
            :current-page="investmentDrawerTopPage"
            :page-size="investmentDrawerTopPageSize"
            :page-sizes="[20, 50, 100]"
            :total="investmentDrawerFilteredTopRecommendations.length"
            layout="total, sizes, prev, pager, next"
            @current-change="onInvestmentDrawerTopPageChange"
            @size-change="onInvestmentDrawerTopPageSizeChange"
          />
        </div>
      </template>
    </el-drawer>

    <el-dialog v-model="investmentActionDialogVisible" :title="investmentActionDialogTitle" width="560px">
      <el-form label-width="110px">
        <el-form-item label="日期">
          <el-date-picker
            v-model="investmentActionForm.account_date"
            type="date"
            value-format="YYYY-MM-DD"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="金额">
          <el-input v-model="investmentActionForm.amount" placeholder="0.00" />
        </el-form-item>
        <el-form-item label="份额">
          <el-input v-model="investmentActionForm.share_amount" placeholder="0.000000" />
        </el-form-item>
        <el-form-item v-if="investmentActionForm.action === 'open_position'" label="产品名称">
          <el-input v-model="investmentActionForm.product_name" />
        </el-form-item>
        <el-form-item v-if="investmentActionForm.action === 'open_position'" label="产品代码">
          <el-input v-model="investmentActionForm.product_code" />
        </el-form-item>
        <el-form-item v-if="investmentActionForm.action === 'open_position'" label="机构名称">
          <el-input v-model="investmentActionForm.organization_name" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="investmentActionDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="investmentActionSubmitting" @click="submitInvestmentAction">
          提交
        </el-button>
      </template>
    </el-dialog>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import { requireCurrentUserId } from '@/api/client'
import {
  fetchCashTrend,
  fetchDashboardSummary,
  type CashTrendPoint,
  type DashboardPendingItem,
  type DashboardSummary
} from '@/api/dashboard'
import {
  fetchConfigItems,
  fetchCreditCards,
  updateConfigItem,
  type ConfigItem,
  type CreditCard
} from '@/api/config'
import { fetchInvestments, fetchTopInvestments, type Investment, type InvestmentRecommendation } from '@/api/investments'
import {
  fetchInvestmentTransactions,
  type InvestmentTransaction
} from '@/api/investment-transactions'
import { createBill } from '@/api/bills'

type DashboardTableRow = {
  id?: number
  credit_card_id?: number
  name: string
  due_date: string
  amount: string
  type?: string
  required_period_count?: number | string
  paid_period_count?: number | string
}

const chartRef = ref<HTMLDivElement | null>(null)
const isMobileView = ref(typeof window !== 'undefined' ? window.innerWidth <= 900 : false)
let chart: echarts.ECharts | null = null
const router = useRouter()

function syncViewportMode() {
  isMobileView.value = window.innerWidth <= 900
}

function formatLocalDate(date: Date): string {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

function normalizeDebtRows(
  value: DashboardPendingItem[] | undefined,
  includeType = false
): DashboardTableRow[] {
  return (value ?? []).map((item) => ({
    id: item.id,
    credit_card_id: item.credit_card_id,
    name: item.name?.trim() || '待处理项',
    due_date: item.due_date || '--',
    amount: item.amount || '--',
    required_period_count: item.required_period_count ?? '--',
    paid_period_count: item.paid_period_count ?? '--',
    ...(includeType ? { type: item.type || '--' } : {})
  }))
}

const currentUserId = requireCurrentUserId()
const today = new Date()
const defaultStart = new Date(today.getFullYear(), today.getMonth(), 1)
const defaultEnd = new Date(today.getFullYear(), today.getMonth() + 1, 0)
const dateRange = ref<[string, string]>([
  formatLocalDate(defaultStart),
  formatLocalDate(defaultEnd)
])
const range = ref({
  user_id: currentUserId,
  start_date: dateRange.value[0],
  end_date: dateRange.value[1]
})
const summary = ref<DashboardSummary | null>(null)
const trend = ref<CashTrendPoint[]>([])
const loading = ref(false)
const chartLoading = ref(false)
const useProvidentFundForMortgage = ref(false)
const mortgageSettingId = ref<number | null>(null)
const investmentDrawerVisible = ref(false)
const investmentDrawerMode = ref<'stock' | 'wealth' | 'transactions' | 'top20'>('stock')
const investmentDrawerInvestments = ref<Investment[]>([])
const investmentDrawerTransactions = ref<{ list: InvestmentTransaction[]; total: number }>({
  list: [],
  total: 0
})
const investmentDrawerTopRecommendations = ref<InvestmentRecommendation[]>([])
const investmentDrawerTransactionsKeyword = ref('')
const investmentDrawerTransactionsPage = ref(1)
const investmentDrawerTransactionsPageSize = ref(50)
const investmentDrawerTopType = ref('')
const investmentDrawerTopKeyword = ref('')
const investmentDrawerTopKeywordApplied = ref('')
const investmentDrawerTopPage = ref(1)
const investmentDrawerTopPageSize = ref(20)
const investmentActionDialogVisible = ref(false)
const investmentActionSubmitting = ref(false)
const accountCategoryMap = ref<Record<string, number>>({})
const creditCardConfigMap = ref<Record<number, { billingDay: number; repaymentDay: number }>>({})
const investmentActionForm = ref({
  action: 'open_position' as 'open_position' | 'add_position' | 'reduce_position',
  account_date: formatLocalDate(new Date()),
  amount: '',
  share_amount: '',
  product_name: '',
  product_code: '',
  organization_name: '',
  related_investment_id: undefined as number | undefined,
  investment_type: 'stock' as 'stock' | 'wealth'
})

const investmentDrawerTitle = computed(() => {
  if (investmentDrawerMode.value === 'stock') return '股票持仓'
  if (investmentDrawerMode.value === 'wealth') return '理财持仓'
  if (investmentDrawerMode.value === 'transactions') return '股票流水'
  return 'TOP20 建议'
})

const investmentDrawerFilteredTopRecommendations = computed(() => {
  const keyword = investmentDrawerTopKeywordApplied.value.trim().toLowerCase()
  if (!keyword) return investmentDrawerTopRecommendations.value
  return investmentDrawerTopRecommendations.value.filter((item) =>
    `${item.investment_id} ${item.name} ${item.code} ${item.reason}`.toLowerCase().includes(keyword)
  )
})

const investmentDrawerPagedTopRecommendations = computed(() => {
  const start = (investmentDrawerTopPage.value - 1) * investmentDrawerTopPageSize.value
  return investmentDrawerFilteredTopRecommendations.value.slice(
    start,
    start + investmentDrawerTopPageSize.value
  )
})

const investmentActionDialogTitle = computed(() => {
  const labelMap: Record<string, string> = {
    open_position: '建仓',
    add_position: '加仓',
    reduce_position: '减仓'
  }
  return `投资操作 - ${labelMap[investmentActionForm.value.action] || '操作'}`
})

const repayTrendRows = computed(() =>
  (summary.value?.repay_trend.items ?? []).map((item) => ({
    date: item.date || '--',
    credit_names: item.credit_names || '--',
    cycle_names: item.cycle_names || '--',
    credit_due: item.credit_due || '--',
    cycle_due: item.cycle_due || '--',
    total_due: item.total_due || '--',
    before_salary: item.before_salary
  }))
)

const pendingItems = computed(() => normalizeDebtRows(summary.value?.pending_all, true))
const creditCardRows = computed(() => normalizeDebtRows(summary.value?.credit_cards))
const cycleDebtRows = computed(() => normalizeDebtRows(summary.value?.cycle_debts))

const salaryPrepCard = computed(() => {
  const raw = summary.value?.salary_prep
  if (!raw) return null
  const hasWindowAmounts = Boolean(raw.credit_due_in_window || raw.cycle_due_in_window)
  return {
    daysUntilSalary: `${raw.days_until_salary} 天`,
    salaryTargetDate: raw.salary_target_date || '--',
    dueBeforeSalary: raw.due_before_salary || '--',
    windowStart: raw.window_start || '--',
    windowEnd: raw.window_end || '--',
    windowDue: hasWindowAmounts
      ? `${raw.credit_due_in_window || '--'} / ${raw.cycle_due_in_window || '--'}`
      : '--',
    creditCount: String(raw.credit_count),
    cycleCount: String(raw.cycle_count)
  }
})

const positionSummaryCard = computed(() => {
  const raw = summary.value?.position_summary
  if (!raw) return null
  return {
    totalInvestment: raw.total_investment || '--',
    totalProfit: raw.total_profit || '--',
    holdingProfit: raw.holding_profit || '--',
    avgProfitRate: raw.avg_profit_rate || '--',
    avgAnnualRate: raw.avg_annual_rate_wealth || '--',
    personalWealth: raw.personal_wealth || '--',
    personalStock: raw.personal_stock || '--',
    stockIdleCash: raw.stock_idle_cash || '--'
  }
})

const repaySummaryCard = computed(() => {
  const raw = summary.value?.repay_trend
  if (!raw) return null
  return {
    beforeSalaryTotal: raw.before_salary_total || '--',
    windowTotal: raw.window_total || '--',
    windowStart: raw.window_start || '--',
    windowEnd: raw.window_end || '--'
  }
})

function openBills(query: Record<string, string | number | undefined>) {
  void router.push({
    name: 'bills',
    query: Object.fromEntries(Object.entries(query).filter(([, value]) => value !== undefined && value !== ''))
  })
}

function openCreditCardBills() {
  if (!summary.value) return
  openBills({
    from: 'dashboard',
    context: 'credit-card-pending',
    payment_method: 'credit_card',
    start_date: summary.value.start_date,
    end_date: summary.value.end_date
  })
}

function openCreditCardBillRow(row: DashboardTableRow) {
  if (!summary.value) return
  const statementWindow = resolveCreditCardStatementWindow(row)
  openBills({
    from: 'dashboard',
    context: 'credit-card-row',
    payment_method: 'credit_card',
    credit_card_id: row.credit_card_id,
    start_date: statementWindow?.startDate || summary.value.start_date,
    end_date: statementWindow?.endDate || summary.value.end_date
  })
}

function openPendingBills() {
  if (!repaySummaryCard.value) return
  openBills({
    from: 'dashboard',
    context: 'pending-window',
    start_date: repaySummaryCard.value.windowStart,
    end_date: repaySummaryCard.value.windowEnd
  })
}

function openCycleDebts(keyword?: string) {
  void router.push({
    name: 'debts',
    query: {
      tab: 'non_credit_card',
      from: 'dashboard',
      context: keyword ? 'cycle-debt-row' : 'cycle-debt-summary',
      ...(keyword ? { keyword } : {})
    }
  })
}

function openCycleDebtRow(row: DashboardTableRow) {
  openCycleDebts(row.name)
}

async function openHomepageInvestmentDrawer(mode: 'stock' | 'wealth' | 'transactions' | 'top20') {
  investmentDrawerMode.value = mode
  investmentDrawerVisible.value = true

  try {
    if (mode === 'stock' || mode === 'wealth') {
      const data = await fetchInvestments({
        user_id: currentUserId,
        page: 1,
        page_size: 100,
        investment_type: mode,
        show_sold: false
      })
      investmentDrawerInvestments.value = data.list
      return
    }

    if (mode === 'transactions') {
      investmentDrawerTransactionsKeyword.value = ''
      investmentDrawerTransactionsPage.value = 1
      await loadInvestmentDrawerTransactions()
      return
    }

    investmentDrawerTopType.value = ''
    investmentDrawerTopKeyword.value = ''
    investmentDrawerTopKeywordApplied.value = ''
    investmentDrawerTopPage.value = 1
    await loadInvestmentDrawerTopRecommendations()
  } catch {
    ElMessage.error('加载投资抽屉数据失败')
  }
}

async function loadInvestmentDrawerTransactions() {
  investmentDrawerTransactions.value = await fetchInvestmentTransactions(
    investmentDrawerTransactionsKeyword.value,
    investmentDrawerTransactionsPage.value,
    investmentDrawerTransactionsPageSize.value
  )
}

function searchInvestmentDrawerTransactions() {
  investmentDrawerTransactionsPage.value = 1
  void loadInvestmentDrawerTransactions()
}

function onInvestmentDrawerTransactionsPageChange(page: number) {
  investmentDrawerTransactionsPage.value = page
  void loadInvestmentDrawerTransactions()
}

function onInvestmentDrawerTransactionsPageSizeChange(size: number) {
  investmentDrawerTransactionsPageSize.value = size
  investmentDrawerTransactionsPage.value = 1
  void loadInvestmentDrawerTransactions()
}

async function loadInvestmentDrawerTopRecommendations() {
  investmentDrawerTopRecommendations.value = await fetchTopInvestments(
    investmentDrawerTopType.value || undefined,
    currentUserId
  )
  investmentDrawerTopPage.value = 1
}

function applyInvestmentDrawerTopKeyword() {
  investmentDrawerTopKeywordApplied.value = investmentDrawerTopKeyword.value
  investmentDrawerTopPage.value = 1
}

function onInvestmentDrawerTopPageChange(page: number) {
  investmentDrawerTopPage.value = page
}

function onInvestmentDrawerTopPageSizeChange(size: number) {
  investmentDrawerTopPageSize.value = size
  investmentDrawerTopPage.value = 1
}

function toNumber(value: string | number | null | undefined) {
  const parsed = Number.parseFloat(String(value ?? '0'))
  return Number.isFinite(parsed) ? parsed : 0
}

function profitColorClass(value: number) {
  if (value > 0) return 'profit-positive'
  if (value < 0) return 'profit-negative'
  return 'profit-neutral'
}

function openPendingRow(row: DashboardTableRow) {
  if (row.type === '信用卡') {
    openCreditCardBills()
    return
  }
  openCycleDebtRow(row)
}

function renderChart() {
  if (!chartRef.value) return
  if (!chart) chart = echarts.init(chartRef.value)

  const debtEventsByDate = new Map<
    string,
    Array<{ debtType: string; debtName: string; dueDate: string; amount: string }>
  >()

  const markPointData = trend.value.flatMap((point) => {
    const y = Number.parseFloat(point.cash_balance || '0')
    const marks = point.mark_points || []
    return marks.map((mark) => {
      const events = debtEventsByDate.get(point.date) || []
      events.push({
        debtType: mark.debt_type,
        debtName: mark.debt_name,
        dueDate: mark.due_date,
        amount: mark.amount
      })
      debtEventsByDate.set(point.date, events)
      return {
        coord: [point.date, y],
        value: Number.parseFloat(mark.amount || '0'),
        itemStyle: { color: mark.debt_type === '信用卡' ? '#ef4444' : '#f59e0b' },
        label: { show: false }
      }
    })
  })

  const formatAmount = (value: number) =>
    value.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })

  chart.setOption(
    {
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'line' },
        formatter: (params: unknown) => {
          const rows = Array.isArray(params) ? params : [params]
          if (!rows.length) return ''
          const firstRow = rows[0] as { axisValue?: string }
          const date = String(firstRow.axisValue || '')
          const cashPoint = rows.find(
            (row) => typeof row === 'object' && row !== null && (row as { seriesType?: string }).seriesType === 'line'
          ) as { value?: number } | undefined
          const cash = Number(cashPoint?.value || 0)
          const lines = [
            `<div style="font-weight:600;margin-bottom:4px;">${date}</div>`,
            `现金余额: ${formatAmount(cash)}`
          ]
          const debtRows = debtEventsByDate.get(date) || []
          if (debtRows.length) {
            lines.push('<div style="margin-top:6px;font-weight:600;">到期扣减</div>')
            debtRows.forEach((item) => {
              const color = item.debtType === '信用卡' ? '#ef4444' : '#f59e0b'
              lines.push(
                `<div><span style="display:inline-block;width:7px;height:7px;border-radius:50%;background:${color};margin-right:6px;"></span>${item.debtType}:${item.debtName} (${item.dueDate}) -${item.amount}</div>`
              )
            })
          }
          return lines.join('<br/>')
        }
      },
      xAxis: { type: 'category', boundaryGap: false, data: trend.value.map((item) => item.date) },
      yAxis: { type: 'value' },
      series: [
        {
          type: 'line',
          smooth: true,
          showSymbol: false,
          data: trend.value.map((item) => Number.parseFloat(item.cash_balance || '0')),
          markPoint: {
            symbol: 'circle',
            symbolSize: 10,
            data: markPointData
          }
        }
      ]
    },
    true
  )

  chart.resize()
}

async function loadDashboard() {
  loading.value = true
  chartLoading.value = true
  try {
    const [summaryData, trendData] = await Promise.all([
      fetchDashboardSummary(range.value),
      fetchCashTrend(range.value)
    ])
    summary.value = summaryData
    loading.value = false
    trend.value = trendData.filter((item) => item.date >= range.value.start_date)
    await nextTick()
    renderChart()
  } finally {
    loading.value = false
    chartLoading.value = false
  }
}

async function loadMortgageSetting() {
  const data = await fetchConfigItems('system_setting', 1, 100)
  const setting = data.list.find((item) => item.name === 'use_provident_fund_for_mortgage')
  mortgageSettingId.value = setting?.id ?? null
  useProvidentFundForMortgage.value = Boolean(setting?.enabled)
}

async function loadAccountCategoryMap() {
  const data = await fetchConfigItems('account_category', 1, 200)
  const map: Record<string, number> = {}
  data.list.forEach((item) => {
    map[item.name] = item.id
  })
  accountCategoryMap.value = map
}

function parseDateYmd(raw: string): Date | null {
  const parts = raw.split('-').map((part) => Number(part))
  if (parts.length !== 3 || parts.some((part) => !Number.isInteger(part))) return null
  const [year, month, day] = parts
  if (year <= 0 || month < 1 || month > 12 || day < 1 || day > 31) return null
  const date = new Date(year, month - 1, day)
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) return null
  return date
}

function buildSafeDate(year: number, monthIndex: number, day: number): Date {
  const maxDay = new Date(year, monthIndex + 1, 0).getDate()
  const safeDay = Math.min(Math.max(day, 1), maxDay)
  return new Date(year, monthIndex, safeDay)
}

function previousMonth(year: number, monthIndex: number): [number, number] {
  if (monthIndex === 0) return [year - 1, 11]
  return [year, monthIndex - 1]
}

function resolveCreditCardStatementWindow(row: DashboardTableRow): { startDate: string; endDate: string } | null {
  if (!row.credit_card_id) return null
  if (!row.due_date || row.due_date === '--') return null
  const dueDate = parseDateYmd(row.due_date)
  if (!dueDate) return null

  const config = creditCardConfigMap.value[row.credit_card_id]
  if (!config) return null

  let endYear = dueDate.getFullYear()
  let endMonthIndex = dueDate.getMonth()
  if (config.billingDay >= config.repaymentDay) {
    const [prevYear, prevMonthIndex] = previousMonth(endYear, endMonthIndex)
    endYear = prevYear
    endMonthIndex = prevMonthIndex
  }

  const endDate = buildSafeDate(endYear, endMonthIndex, config.billingDay)
  const [startRefYear, startRefMonthIndex] = previousMonth(endDate.getFullYear(), endDate.getMonth())
  const startDate = buildSafeDate(startRefYear, startRefMonthIndex, config.billingDay)
  startDate.setDate(startDate.getDate() + 1)

  return {
    startDate: formatLocalDate(startDate),
    endDate: formatLocalDate(endDate)
  }
}

async function loadCreditCardConfigs() {
  try {
    const data = await fetchCreditCards(1, 500)
    const map: Record<number, { billingDay: number; repaymentDay: number }> = {}
    data.list.forEach((item: CreditCard) => {
      map[item.id] = {
        billingDay: item.billing_day,
        repaymentDay: item.repayment_day
      }
    })
    creditCardConfigMap.value = map
  } catch {
    creditCardConfigMap.value = {}
  }
}

function openInvestmentActionDialog(
  action: 'open_position' | 'add_position' | 'reduce_position',
  row: Investment
) {
  investmentActionForm.value.action = action
  investmentActionForm.value.account_date = formatLocalDate(new Date())
  investmentActionForm.value.amount = ''
  investmentActionForm.value.share_amount = ''
  investmentActionForm.value.product_name = row.name || ''
  investmentActionForm.value.product_code = row.code || ''
  investmentActionForm.value.organization_name = row.organization_name || ''
  investmentActionForm.value.related_investment_id =
    action === 'open_position' ? undefined : row.id
  investmentActionForm.value.investment_type =
    row.investment_type === 'wealth' ? 'wealth' : 'stock'
  investmentActionDialogVisible.value = true
}

async function submitInvestmentAction() {
  const form = investmentActionForm.value
  const categoryId = accountCategoryMap.value[form.investment_type]
  if (!categoryId) {
    ElMessage.error('未找到投资分类配置')
    return
  }
  if (!form.amount.trim()) {
    ElMessage.warning('请输入金额')
    return
  }
  if (!form.share_amount.trim()) {
    ElMessage.warning('请输入份额')
    return
  }
  if (form.action === 'open_position') {
    if (!form.product_name.trim() || !form.product_code.trim() || !form.organization_name.trim()) {
      ElMessage.warning('建仓需填写产品名称、产品代码、机构名称')
      return
    }
  }

  investmentActionSubmitting.value = true
  try {
    await createBill({
      user_id: currentUserId,
      account_date: form.account_date,
      category_id: categoryId,
      bill_type: form.action,
      payment_method: 'stock_account',
      is_fixed_asset: false,
      amount: form.amount,
      investment_action: form.action,
      share_amount: form.share_amount,
      related_investment_id: form.action === 'open_position' ? undefined : form.related_investment_id,
      product_name: form.action === 'open_position' ? form.product_name : undefined,
      product_code: form.action === 'open_position' ? form.product_code : undefined,
      organization_name: form.action === 'open_position' ? form.organization_name : undefined
    })
    ElMessage.success('投资操作已提交')
    investmentActionDialogVisible.value = false
    if (investmentDrawerMode.value === 'stock' || investmentDrawerMode.value === 'wealth') {
      await openHomepageInvestmentDrawer(investmentDrawerMode.value)
    }
  } catch {
    ElMessage.error('投资操作提交失败')
  } finally {
    investmentActionSubmitting.value = false
  }
}

async function handleMortgageSettingChange() {
  if (!mortgageSettingId.value) {
    ElMessage.error('未找到首页房贷设置项')
    return
  }
  try {
    const configData = await fetchConfigItems('system_setting', 1, 100)
    const current = configData.list.find((item) => item.id === mortgageSettingId.value) as ConfigItem | undefined
    if (!current) {
      throw new Error('missing setting')
    }
    await updateConfigItem(mortgageSettingId.value, {
      display_name: current.display_name,
      enabled: useProvidentFundForMortgage.value,
      sort_order: current.sort_order
    })
    await loadDashboard()
    ElMessage.success('现金趋势设置已更新')
  } catch {
    useProvidentFundForMortgage.value = !useProvidentFundForMortgage.value
    ElMessage.error('更新首页房贷设置失败')
  }
}

async function applyRange() {
  const [startDate, endDate] = dateRange.value
  range.value = { user_id: currentUserId, start_date: startDate, end_date: endDate }
  try {
    await loadDashboard()
  } catch {
    ElMessage.error('首页统计刷新失败')
  }
}

onMounted(async () => {
  try {
    await Promise.all([loadAccountCategoryMap(), loadCreditCardConfigs()])
    await loadMortgageSetting()
    await loadDashboard()
  } catch {
    ElMessage.error('首页统计加载失败')
  }
})

onBeforeUnmount(() => {
  chart?.dispose()
  chart = null
})
</script>

<style scoped>
.dashboard-header {
  padding: 20px;
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dashboard-header-main {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: center;
  flex-wrap: wrap;
}

.dashboard-title {
  margin: 0;
}

.dashboard-subtitle {
  color: #64748b;
  margin: 8px 0 0;
}

.dashboard-metrics {
  margin-bottom: 16px;
}

.chart-panel {
  padding: 20px;
  margin-bottom: 16px;
}

.chart-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.chart-title {
  margin: 0;
}

.chart-body {
  height: 320px;
}

.grid-two {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.section-panel {
  padding: 16px;
}

.summary-block {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.summary-hero {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  padding: 14px 16px;
  border-radius: 16px;
  background: linear-gradient(135deg, rgba(241, 245, 249, 0.95), rgba(248, 250, 252, 0.95));
}

.summary-label {
  font-size: 12px;
  color: #64748b;
}

.summary-value {
  margin-top: 6px;
  font-size: 26px;
  font-weight: 700;
  color: #0f172a;
}

.summary-value-sm {
  font-size: 20px;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.summary-stat {
  padding: 12px 14px;
  border-radius: 14px;
  background: rgba(248, 250, 252, 0.9);
  border: 1px solid rgba(148, 163, 184, 0.16);
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: #475569;
}

.summary-stat strong {
  color: #0f172a;
}

.summary-meta {
  color: #64748b;
  font-size: 12px;
}

.summary-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.drawer-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 10px;
}

.action-inline {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  white-space: nowrap;
}

.profit-positive {
  color: #dc2626;
}

.profit-negative {
  color: #16a34a;
}

.profit-neutral {
  color: #111827;
}

.empty-tip {
  color: #94a3b8;
}

.repay-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 12px;
}

.repay-row {
  display: grid;
  grid-template-columns: 1fr 1.2fr 1.2fr 0.8fr 0.8fr 0.8fr 0.8fr;
  gap: 8px;
  color: #334155;
}

.repay-head {
  color: #64748b;
  font-size: 12px;
}

.mobile-cards-mini {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mobile-card-mini {
  padding: 12px;
  border-radius: 12px;
}

.mobile-card-mini-head {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  color: #475569;
}

.mobile-card-mini-content {
  margin-top: 8px;
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
}

.mobile-card-mini-action {
  margin-top: 10px;
  width: 100%;
}

@media (max-width: 960px) {
  .grid-two {
    grid-template-columns: 1fr;
  }

  .summary-hero,
  .summary-grid {
    grid-template-columns: 1fr;
  }

  .repay-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
