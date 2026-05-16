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

    <div class="metric-grid dashboard-metrics">
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

    <div class="panel chart-panel">
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
          <div class="summary-actions">
            <el-button text type="primary" @click="openCreditCardBills">查看信用卡账单</el-button>
            <el-button text type="primary" @click="openCycleDebts()">查看周期债务</el-button>
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
            <el-button text type="primary" @click="openInvestments('stock')">查看股票持仓</el-button>
            <el-button text type="primary" @click="openInvestments('wealth')">查看理财持仓</el-button>
            <el-button text type="primary" @click="openInvestmentDrawer('transactions')">股票流水</el-button>
            <el-button text type="primary" @click="openInvestmentDrawer('top20')">TOP20 建议</el-button>
          </div>
        </div>
        <div v-else class="empty-tip">暂无数据</div>
      </div>

      <div class="panel section-panel">
        <h3>还款趋势摘要</h3>
        <div v-if="repaySummaryCard" class="summary-block">
          <div class="summary-hero">
            <div>
              <div class="summary-label">发薪日前应还</div>
              <div class="summary-value">{{ repaySummaryCard.beforeSalaryTotal }}</div>
            </div>
            <div>
              <div class="summary-label">窗口期应还</div>
              <div class="summary-value summary-value-sm">{{ repaySummaryCard.windowTotal }}</div>
            </div>
          </div>
          <div class="summary-grid">
            <div class="summary-stat">
              <span>统计起点</span>
              <strong>{{ repaySummaryCard.windowStart }}</strong>
            </div>
            <div class="summary-stat">
              <span>统计终点</span>
              <strong>{{ repaySummaryCard.windowEnd }}</strong>
            </div>
            <div class="summary-stat">
              <span>目标发薪日</span>
              <strong>{{ repaySummaryCard.salaryTargetDate }}</strong>
            </div>
            <div class="summary-stat">
              <span>到期日数</span>
              <strong>{{ repaySummaryCard.dueDates }}</strong>
            </div>
          </div>
          <div class="summary-meta">
            数据按到期日聚合展示，主卡片不再显示技术来源字段。
          </div>
          <div class="summary-actions">
            <el-button text type="primary" @click="openPendingBills">查看待处理账单</el-button>
            <el-button text type="primary" @click="openCycleDebts()">查看债务列表</el-button>
          </div>
        </div>
        <div v-else class="empty-tip">暂无摘要数据</div>

        <div v-if="repayTrendRows.length" class="repay-list">
          <div class="repay-row repay-head">
            <span>日期</span>
            <span>信用卡名称</span>
            <span>周期债务名称</span>
            <span>信用卡应还</span>
            <span>周期债务应还</span>
            <span>合计</span>
            <span>发薪日前</span>
          </div>
          <div v-for="(row, idx) in repayTrendRows" :key="`${row.date}-${idx}`" class="repay-row">
            <span>{{ row.date }}</span>
            <span>{{ row.credit_names }}</span>
            <span>{{ row.cycle_names }}</span>
            <span>{{ row.credit_due }}</span>
            <span>{{ row.cycle_due }}</span>
            <strong>{{ row.total_due }}</strong>
            <span>{{ row.before_salary ? '是' : '否' }}</span>
          </div>
        </div>
        <div v-else class="empty-tip">暂无还款日历</div>
      </div>

      <div class="panel section-panel">
        <h3>信用卡待还</h3>
        <el-table v-if="creditCardRows.length" :data="creditCardRows" size="small" stripe>
          <el-table-column prop="name" label="名称" min-width="160" />
          <el-table-column prop="due_date" label="到期日" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column label="操作" width="110">
            <template #default>
              <el-button text type="primary" @click="openCreditCardBills">查看明细</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="empty-tip">暂无信用卡待还项</div>
      </div>

      <div class="panel section-panel">
        <h3>周期债务待还</h3>
        <el-table v-if="cycleDebtRows.length" :data="cycleDebtRows" size="small" stripe>
          <el-table-column prop="name" label="名称" min-width="160" />
          <el-table-column prop="due_date" label="到期日" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column label="操作" width="110">
            <template #default="{ row }">
              <el-button text type="primary" @click="openCycleDebtRow(row)">查看明细</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="empty-tip">暂无周期债务待还项</div>
      </div>

      <div class="panel section-panel">
        <h3>全部待处理（信用卡+周期债务）</h3>
        <el-table v-if="pendingItems.length" :data="pendingItems" size="small" stripe>
          <el-table-column prop="name" label="名称" min-width="180" />
          <el-table-column prop="type" label="类型" width="110" />
          <el-table-column prop="due_date" label="到期日" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column label="操作" width="110">
            <template #default="{ row }">
              <el-button text type="primary" @click="openPendingRow(row)">查看明细</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="empty-tip">暂无待处理项</div>
      </div>
    </div>
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
import { fetchConfigItems, updateConfigItem, type ConfigItem } from '@/api/config'

type DashboardTableRow = {
  name: string
  due_date: string
  amount: string
  type?: string
}

const chartRef = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null
const router = useRouter()

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
    name: item.name?.trim() || '待处理项',
    due_date: item.due_date || '--',
    amount: item.amount || '--',
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
const useProvidentFundForMortgage = ref(false)
const mortgageSettingId = ref<number | null>(null)

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
    windowEnd: raw.window_end || '--',
    salaryTargetDate: raw.salary_target_date || '--',
    dueDates: String(raw.items.length)
  }
})

function openBills(query: Record<string, string | number | undefined>) {
  void router.push({
    name: 'bills',
    query: Object.fromEntries(Object.entries(query).filter(([, value]) => value !== undefined && value !== ''))
  })
}

function openCreditCardBills() {
  if (!salaryPrepCard.value) return
  openBills({
    from: 'dashboard',
    context: 'credit-card-pending',
    payment_method: 'credit_card',
    start_date: salaryPrepCard.value.windowStart,
    end_date: salaryPrepCard.value.windowEnd
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
      from: 'dashboard',
      context: keyword ? 'cycle-debt-row' : 'cycle-debt-summary',
      ...(keyword ? { keyword } : {})
    }
  })
}

function openCycleDebtRow(row: DashboardTableRow) {
  openCycleDebts(row.name)
}

function openInvestments(tab: 'stock' | 'wealth') {
  void router.push({
    name: 'investments',
    query: {
      from: 'dashboard',
      context: tab === 'stock' ? 'stock-summary' : 'wealth-summary',
      tab,
      show_sold: 'false'
    }
  })
}

function openInvestmentDrawer(drawer: 'transactions' | 'top20') {
  void router.push({
    name: 'investments',
    query: {
      from: 'dashboard',
      context: drawer === 'transactions' ? 'stock-transactions' : 'top20-recommendations',
      open_drawer: drawer,
      tab: 'stock',
      show_sold: 'false'
    }
  })
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
  const [summaryData, trendData] = await Promise.all([
    fetchDashboardSummary(range.value),
    fetchCashTrend(range.value)
  ])
  summary.value = summaryData
  trend.value = trendData.filter((item) => item.date >= range.value.start_date)
  await nextTick()
  renderChart()
}

async function loadMortgageSetting() {
  const data = await fetchConfigItems('system_setting', 1, 100)
  const setting = data.list.find((item) => item.name === 'use_provident_fund_for_mortgage')
  mortgageSettingId.value = setting?.id ?? null
  useProvidentFundForMortgage.value = Boolean(setting?.enabled)
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
