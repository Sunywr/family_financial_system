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
      <h3 class="chart-title">现金余额趋势</h3>
      <div ref="chartRef" class="chart-body"></div>
    </div>

    <div class="grid-two">
      <div class="panel section-panel">
        <h3>发薪日前准备</h3>
        <div v-if="salaryItems.length" class="kv-list">
          <div v-for="item in salaryItems" :key="item.key" class="kv-row">
            <span>{{ item.key }}</span>
            <strong>{{ item.value }}</strong>
          </div>
        </div>
        <div v-else class="empty-tip">暂无数据</div>
      </div>

      <div class="panel section-panel">
        <h3>投资持仓摘要</h3>
        <div v-if="positionItems.length" class="kv-list">
          <div v-for="item in positionItems" :key="item.key" class="kv-row">
            <span>{{ item.key }}</span>
            <strong>{{ item.value }}</strong>
          </div>
        </div>
        <div v-else class="empty-tip">暂无数据</div>
      </div>

      <div class="panel section-panel">
        <h3>还款趋势摘要</h3>
        <div v-if="repaySummaryItems.length" class="kv-list">
          <div v-for="item in repaySummaryItems" :key="item.key" class="kv-row">
            <span>{{ item.key }}</span>
            <strong>{{ item.value }}</strong>
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
            <span>{{ row.credit_names || '--' }}</span>
            <span>{{ row.cycle_names || '--' }}</span>
            <span>{{ row.credit_due }}</span>
            <span>{{ row.cycle_due }}</span>
            <strong>{{ row.total_due }}</strong>
            <span>{{ row.before_salary ? '是' : '否' }}</span>
          </div>
        </div>
        <div v-else class="empty-tip">暂无还款日历</div>
      </div>

      <div class="panel section-panel">
        <h3>全部待处理（信用卡+周期债务）</h3>
        <div v-if="pendingItems.length" class="pending-list">
          <div v-for="(item, idx) in pendingItems" :key="idx" class="pending-row">
            <span>{{ item.name || '待处理项' }}</span>
            <span>{{ item.due_date || '--' }}</span>
            <strong>{{ item.amount || '--' }}</strong>
          </div>
        </div>
        <div v-else class="empty-tip">暂无待处理项</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import { requireCurrentUserId } from '@/api/client'
import { fetchCashTrend, fetchDashboardSummary, type CashTrendPoint, type DashboardSummary } from '@/api/dashboard'

const chartRef = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

function formatLocalDate(date: Date): string {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
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

const salaryItems = computed(() => toKvList(summary.value?.salary_prep, salaryPrepLabels))
const positionItems = computed(() => toKvList(summary.value?.position_summary, positionSummaryLabels))
const repaySummaryItems = computed(() => {
  const raw = summary.value?.repay_trend
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return []
  const rest = { ...(raw as Record<string, unknown>) }
  delete rest.items
  return toKvList(rest, repaySummaryLabels)
})
const repayTrendRows = computed(() => {
  const raw = summary.value?.repay_trend
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return []
  const rows = (raw as Record<string, unknown>).items
  if (!Array.isArray(rows)) return []
  return rows.map((item) => {
    const row = (item ?? {}) as Record<string, unknown>
    return {
      date: String(row.date ?? '--'),
      credit_names: String(row.credit_names ?? '--'),
      cycle_names: String(row.cycle_names ?? '--'),
      credit_due: String(row.credit_due ?? '--'),
      cycle_due: String(row.cycle_due ?? '--'),
      total_due: String(row.total_due ?? '--'),
      before_salary: Boolean(row.before_salary)
    }
  })
})
const pendingItems = computed(() => {
  const raw = summary.value?.pending_all
  if (!Array.isArray(raw)) return []
  return raw.map((item) => {
    const row = (item ?? {}) as Record<string, unknown>
    const name = String(row.name ?? '').trim()
    return {
      name: name || '待处理项',
      due_date: String(row.due_date ?? '--'),
      amount: String(row.amount ?? '--')
    }
  })
})

const salaryPrepLabels: Record<string, string> = {
  window_start: '统计起点',
  window_end: '统计终点',
  salary_day: '发薪日',
  salary_target_date: '目标发薪日',
  days_until_salary: '距发薪日(天)',
  pending_count: '待处理总数',
  credit_count: '信用卡笔数',
  cycle_count: '周期债务笔数',
  credit_due_before_salary: '发薪日前信用卡应还',
  cycle_due_before_salary: '发薪日前周期债务应还',
  due_before_salary: '发薪日前应还合计',
  credit_due_in_window: '窗口内信用卡应还',
  cycle_due_in_window: '窗口内周期债务应还',
  source: '数据来源'
}

const positionSummaryLabels: Record<string, string> = {
  personal_wealth: '个人理财市值',
  personal_stock: '个人股票总资产',
  total_investment: '总投资额',
  holding_profit: '持仓收益',
  total_profit: '总收益',
  avg_profit_rate: '平均收益率',
  avg_annual_rate_wealth: '平均年化收益率(仅理财)',
  family_wealth: '家庭理财市值',
  family_stock: '家庭股票总资产',
  stock_idle_cash: '股票账户可用现金',
  source: '数据来源'
}

const repaySummaryLabels: Record<string, string> = {
  source: '数据来源',
  window_start: '统计起点',
  window_end: '统计终点',
  salary_target_date: '目标发薪日',
  before_salary_total: '发薪日前应还合计',
  window_total: '窗口期应还合计'
}

function toKvList(
  value: unknown,
  labels: Record<string, string> = {}
): Array<{ key: string; value: string }> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return []
  return Object.entries(value as Record<string, unknown>)
    .filter(([_, v]) => v !== null && v !== undefined && `${v}` !== '')
    .map(([k, v]) => ({ key: labels[k] || k, value: typeof v === 'string' ? v : JSON.stringify(v) }))
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
        formatter: (params: any) => {
          const rows = Array.isArray(params) ? params : [params]
          if (!rows.length) return ''
          const date = String(rows[0].axisValue || '')
          const cashPoint = rows.find((row) => row.seriesType === 'line')
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
.chart-title {
  margin: 0 0 12px;
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
.kv-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.kv-row {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  color: #334155;
}
.pending-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pending-row {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 12px;
  color: #334155;
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
  .repay-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
