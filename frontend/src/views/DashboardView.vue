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
        <div class="metric-label">股票账户总额</div>
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
        <pre>{{ pretty(summary?.salary_prep) }}</pre>
      </div>
      <div class="panel section-panel">
        <h3>投资持仓摘要</h3>
        <pre>{{ pretty(summary?.position_summary) }}</pre>
      </div>
      <div class="panel section-panel">
        <h3>还款趋势</h3>
        <pre>{{ pretty(summary?.repay_trend) }}</pre>
      </div>
      <div class="panel section-panel">
        <h3>全部待处理（信用卡+周期债务）</h3>
        <pre>{{ pretty(summary?.pending_all) }}</pre>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import { requireCurrentUserId } from '@/api/client'
import {
  fetchCashTrend,
  fetchDashboardSummary,
  type CashTrendPoint,
  type DashboardSummary
} from '@/api/dashboard'

const chartRef = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null

const currentUserId = requireCurrentUserId()
const today = new Date()
const defaultStart = new Date(today.getFullYear(), today.getMonth(), 1)
const dateRange = ref<[string, string]>([
  defaultStart.toISOString().slice(0, 10),
  today.toISOString().slice(0, 10)
])
const range = ref({
  user_id: currentUserId,
  start_date: dateRange.value[0],
  end_date: dateRange.value[1]
})
const summary = ref<DashboardSummary | null>(null)
const trend = ref<CashTrendPoint[]>([])

function pretty(value: unknown) {
  return JSON.stringify(value ?? {}, null, 2)
}

function renderChart() {
  if (!chartRef.value) return
  if (!chart) chart = echarts.init(chartRef.value)
  chart.setOption(
    {
      tooltip: { trigger: 'axis' },
      xAxis: { type: 'category', boundaryGap: false, data: trend.value.map((item) => item.date) },
      yAxis: { type: 'value' },
      series: [
        {
          type: 'line',
          smooth: true,
          showSymbol: false,
          data: trend.value.map((item) => Number.parseFloat(item.cash_balance || '0'))
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
  trend.value = trendData
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
.section-panel pre {
  white-space: pre-wrap;
  margin: 0;
  color: #334155;
}
@media (max-width: 960px) {
  .grid-two {
    grid-template-columns: 1fr;
  }
}
</style>
