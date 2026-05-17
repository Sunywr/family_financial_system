<template>
  <section class="panel page">
    <div class="head">
      <div>
        <h2>投资仪表盘</h2>
        <p class="subhead">统一查看股票与理财持仓，并在当前页直接展开股票流水和 TOP20 建议。</p>
      </div>
      <div class="actions">
        <el-switch
          v-if="isAdmin"
          v-model="onlyMine"
          active-text="仅看我的"
          inactive-text="查看全部"
          @change="search"
        />
        <el-switch v-model="showSold" active-text="显示已卖出" @change="loadData" />
        <el-input
          v-model="keyword"
          placeholder="全文搜索（名称 / 代码 / ID）"
          style="width: 280px"
          @keyup.enter="search"
        />
        <el-button @click="search">查询</el-button>
        <el-button @click="openTransactionsDrawer">股票流水</el-button>
        <el-button type="primary" plain @click="openTopDrawer">TOP20</el-button>
      </div>
    </div>

    <div v-if="dashboardContextLabel" class="context-banner">
      <el-alert :title="`当前筛选来自首页：${dashboardContextLabel}`" type="info" :closable="false" show-icon />
      <div class="context-actions">
        <el-button text type="primary" @click="goDashboard">返回首页</el-button>
      </div>
    </div>

    <el-tabs v-model="tab" @tab-change="handleTabChange">
      <el-tab-pane label="股票" name="stock" />
      <el-tab-pane label="理财" name="wealth" />
    </el-tabs>

    <div v-if="tab === 'stock'" class="summary-row">
      <el-tag type="info">股票总市值：{{ stockMarketValue }}</el-tag>
      <el-tag type="warning">股票账户闲置资金：{{ stockIdleCash }}</el-tag>
      <el-tag type="success">总收益金额：{{ stockTotalProfit }}</el-tag>
    </div>

    <el-table v-loading="loading" :data="investments.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column label="名称" min-width="220">
        <template #default="{ row }">
          {{ displayInvestmentName(row) }}
        </template>
      </el-table-column>
      <el-table-column prop="code" label="代码" width="130" />
      <el-table-column v-if="tab === 'stock'" label="类型" width="120">
        <template #default="{ row }">
          <el-tag>{{ stockBoardLabel(row.code) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'wealth'" label="类型" width="120">
        <template #default="{ row }">
          <el-tag type="success">{{ wealthKindLabel(row) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="120">
        <template #default="{ row }">
          <el-tag :type="row.status === 'holding' ? 'success' : 'info'">
            {{ row.status === 'holding' ? '持有中' : '已卖出' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="market_value" label="市值" width="120" />
      <el-table-column label="收益金额" width="120">
        <template #default="{ row }">
          <span :class="profitColorClass(toNumber(row.total_profit))">{{ row.total_profit }}</span>
        </template>
      </el-table-column>
      <el-table-column label="收益率" width="120">
        <template #default="{ row }">
          <span :class="profitColorClass(toNumber(row.total_profit_rate))">{{ formatRate(row) }}</span>
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'stock'" label="仓位" width="120">
        <template #default="{ row }">
          {{ formatPosition(row) }}
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'wealth'" label="定投" width="100">
        <template #default="{ row }">
          {{ wealthSIPLabel(row) }}
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'wealth'" label="周期" width="140">
        <template #default="{ row }">
          {{ wealthPeriodLabel(row) }}
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'wealth'" label="理财名称" min-width="260" show-overflow-tooltip>
        <template #default="{ row }">
          {{ wealthRemarkName(row) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="investments.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadData"
        @size-change="loadData"
      />
    </div>

    <el-dialog v-model="dialogVisible" title="编辑投资" width="520px" @closed="resetForm">
      <el-form label-width="110px">
        <el-form-item label="名称">
          <el-input :model-value="editingInvestmentName" disabled />
        </el-form-item>
        <el-form-item label="当前价格">
          <el-input v-model="form.current_price" />
        </el-form-item>
        <el-form-item label="当前市值">
          <el-input v-model="form.market_value" />
        </el-form-item>
        <el-form-item label="持有份额">
          <el-input v-model="form.total_shares" />
        </el-form-item>
        <el-form-item label="总成本">
          <el-input v-model="form.total_cost" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitEdit">保存</el-button>
      </template>
    </el-dialog>

    <el-drawer v-model="transactionsDrawerVisible" title="股票流水" size="78%">
      <div class="drawer-toolbar">
        <el-input
          v-model="transactionsKeyword"
          placeholder="全文搜索（动作 / 备注 / 投资 / 来源账单 ID）"
          style="width: 320px"
          @keyup.enter="searchTransactions"
        />
        <el-button @click="searchTransactions">查询</el-button>
      </div>
      <el-table :data="transactions.list" stripe>
        <el-table-column prop="id" label="ID" width="90" />
        <el-table-column label="投资" min-width="220">
          <template #default="{ row }">
            {{ formatTransactionInvestment(row) }}
          </template>
        </el-table-column>
        <el-table-column prop="source_bill_id" label="来源账单ID" width="120" />
        <el-table-column prop="transaction_date" label="日期" width="120" />
        <el-table-column label="动作" width="120">
          <template #default="{ row }">
            <el-tag>{{ transactionActionMap[row.action] || row.action }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="shares" label="份额" width="120" />
        <el-table-column prop="amount" label="金额" width="120" />
        <el-table-column prop="unit_price" label="单价" width="120" />
        <el-table-column prop="realized_profit" label="已实现收益" width="140" />
        <el-table-column label="备注" min-width="260" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.remark || '手续费已并入操作记录' }}
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-wrap">
        <el-pagination
          v-model:current-page="transactionsPage"
          v-model:page-size="transactionsPageSize"
          :page-sizes="[20, 50, 100]"
          :total="transactions.total"
          layout="total, sizes, prev, pager, next"
          @current-change="loadTransactions"
          @size-change="loadTransactions"
        />
      </div>
    </el-drawer>

    <el-drawer v-model="topDrawerVisible" title="TOP20 建议" size="78%">
      <div class="drawer-toolbar">
        <el-switch
          v-if="isAdmin"
          v-model="topOnlyMine"
          active-text="仅看我的"
          inactive-text="查看全部"
          @change="loadTopRecommendations"
        />
        <el-select v-model="topInvestmentType" style="width: 160px" @change="loadTopRecommendations">
          <el-option label="全部" value="" />
          <el-option label="股票" value="stock" />
          <el-option label="理财" value="wealth" />
        </el-select>
        <el-input
          v-model="topKeyword"
          placeholder="全文搜索（名称 / 代码 / ID / 原因）"
          style="width: 320px"
          @keyup.enter="applyTopKeyword"
        />
        <el-button @click="applyTopKeyword">查询</el-button>
      </div>
      <el-table :data="pagedTopRecommendations" stripe>
        <el-table-column prop="investment_id" label="ID" width="90" />
        <el-table-column label="类型" width="120">
          <template #default="{ row }">
            <el-tag>{{ row.investment_type === 'stock' ? '股票' : '理财' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="名称" min-width="220">
          <template #default="{ row }">
            {{ displayRecommendationName(row) }}
          </template>
        </el-table-column>
        <el-table-column prop="code" label="代码" width="130" />
        <el-table-column prop="score" label="评分" width="90" />
        <el-table-column prop="suggestion" label="建议" width="120" />
        <el-table-column label="收益率" width="120">
          <template #default="{ row }">
            <span :class="profitColorClass(toNumber(row.total_profit_rate))">
              {{ `${(toNumber(row.total_profit_rate) * 100).toFixed(2)}%` }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因" min-width="320" show-overflow-tooltip />
      </el-table>
      <div class="pagination-wrap">
        <el-pagination
          v-model:current-page="topCurrentPage"
          v-model:page-size="topPageSize"
          :page-sizes="[20, 50, 100]"
          :total="filteredTopRecommendations.length"
          layout="total, sizes, prev, pager, next"
        />
      </div>
    </el-drawer>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { fetchDashboardSummary } from '@/api/dashboard'
import { getStoredUser, requireCurrentUserId } from '@/api/client'
import {
  fetchInvestments,
  fetchTopInvestments,
  type Investment,
  type InvestmentRecommendation,
  updateInvestment
} from '@/api/investments'
import {
  fetchInvestmentTransactions,
  type InvestmentTransaction
} from '@/api/investment-transactions'

const route = useRoute()
const router = useRouter()

const tab = ref<'stock' | 'wealth'>('stock')
const keyword = ref('')
const showSold = ref(false)
const onlyMine = ref(true)
const currentPage = ref(1)
const pageSize = ref(50)
const currentUserId = requireCurrentUserId()
const currentUser = getStoredUser()
const isAdmin = computed(() => currentUser?.username === 'admin')
const today = new Date().toISOString().slice(0, 10)

const dashboardContextMap: Record<string, string> = {
  'stock-summary': '股票持仓摘要',
  'wealth-summary': '理财持仓摘要',
  'stock-transactions': '股票流水',
  'top20-recommendations': 'TOP20 建议'
}
const dashboardContextLabel = computed(() => {
  if (route.query.from !== 'dashboard') return ''
  const key = typeof route.query.context === 'string' ? route.query.context : ''
  return dashboardContextMap[key] || '首页筛选'
})

const investments = ref<{ list: Investment[]; total: number }>({ list: [], total: 0 })
const loading = ref(false)
const stockIdleCash = ref('0.00')

const dialogVisible = ref(false)
const editingId = ref<number | null>(null)
const editingInvestmentName = ref('')
const form = ref({
  current_price: '',
  market_value: '',
  total_shares: '',
  total_cost: ''
})

const transactionsDrawerVisible = ref(false)
const transactionsKeyword = ref('')
const transactionsPage = ref(1)
const transactionsPageSize = ref(50)
const transactions = ref<{ list: InvestmentTransaction[]; total: number }>({ list: [], total: 0 })

const topDrawerVisible = ref(false)
const topOnlyMine = ref(true)
const topInvestmentType = ref('')
const topKeyword = ref('')
const topKeywordApplied = ref('')
const topRecommendations = ref<InvestmentRecommendation[]>([])
const topCurrentPage = ref(1)
const topPageSize = ref(20)

const transactionActionMap: Record<string, string> = {
  open_position: '建仓',
  add_position: '加仓',
  reduce_position: '减仓',
  dividend: '分红'
}

const stockMarketValue = computed(() =>
  investments.value.list
    .reduce((acc, item) => acc + Number.parseFloat(item.market_value || '0'), 0)
    .toFixed(2)
)
const stockTotalProfit = computed(() =>
  investments.value.list
    .reduce((acc, item) => acc + Number.parseFloat(item.total_profit || '0'), 0)
    .toFixed(2)
)
const filteredTopRecommendations = computed(() => {
  const keywordValue = topKeywordApplied.value.trim().toLowerCase()
  if (!keywordValue) return topRecommendations.value
  return topRecommendations.value.filter((row) =>
    `${row.investment_id} ${displayRecommendationName(row)} ${row.code} ${row.reason}`
      .toLowerCase()
      .includes(keywordValue)
  )
})
const pagedTopRecommendations = computed(() => {
  const start = (topCurrentPage.value - 1) * topPageSize.value
  return filteredTopRecommendations.value.slice(start, start + topPageSize.value)
})

function stockBoardLabel(code: string) {
  if (code.startsWith('6')) return '沪A'
  if (code.startsWith('3')) return '创业板'
  return '深A'
}

function isGenericStockName(name: string | null | undefined, organizationName: string | null | undefined) {
  const normalizedName = (name || '').trim()
  const normalizedOrganizationName = (organizationName || '').trim()
  return !!normalizedName && normalizedName === normalizedOrganizationName
}

function displayInvestmentName(item: Investment) {
  if (item.investment_type === 'stock' && isGenericStockName(item.name, item.organization_name)) {
    return `股票 ${item.code}`
  }
  return item.name || item.organization_name || item.code
}

function displayRecommendationName(item: InvestmentRecommendation) {
  if (item.investment_type === 'stock' && isGenericStockName(item.name, item.organization_name)) {
    return `股票 ${item.code}`
  }
  return item.name || item.organization_name || item.code
}

function wealthKindLabel(item: Investment) {
  if (item.code.startsWith('0') || item.code.startsWith('1') || item.code.startsWith('2')) {
    return '基金'
  }
  if ((item.name || '').includes('定投')) {
    return '定投'
  }
  return '固收'
}

function formatRate(item: Investment) {
  const value = toNumber(item.total_profit_rate) * 100
  return `${value.toFixed(2)}%`
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

function formatPosition(item: Investment) {
  const marketValue = Number.parseFloat(item.market_value || '0')
  const total = Number.parseFloat(stockMarketValue.value || '0')
  if (total <= 0) return '0.00%'
  return `${((marketValue / total) * 100).toFixed(2)}%`
}

function normalizedRemark(item: Investment) {
  return (item.latest_remark || '').trim()
}

type WealthRemarkMeta = {
  sip: boolean
  period?: string
  name?: string
}

function normalizeToken(raw: string) {
  return raw.replace(/\s+/g, ' ').trim()
}

function splitRemarkTokens(remark: string) {
  return remark
    .replace(/[\r\n]+/g, '|')
    .split(/[|，,；;]/)
    .map(normalizeToken)
    .filter(Boolean)
}

function normalizePeriodText(raw: string) {
  return raw.replace(/\s+/g, '').replace(/每日/g, '每1日')
}

function extractPeriodFromText(text: string) {
  const normalized = text.replace(/\s+/g, '')
  const patterns = [
    /每\d+[天日周月季年]/,
    /每[天日周月季年]/,
    /\d+[天日周月季年]一次/,
    /周[一二三四五六日天]/,
    /月初|月中|月末/,
    /双周/
  ]
  for (const pattern of patterns) {
    const matched = normalized.match(pattern)
    if (matched) {
      return normalizePeriodText(matched[0])
    }
  }
  return undefined
}

function parseSipValue(raw: string) {
  const normalized = raw.replace(/\s+/g, '')
  if (!normalized) return undefined
  if (/(是|开启|开通|已开|true|yes|y|1)/i.test(normalized)) return true
  if (/(否|未开|关闭|false|no|n|0)/i.test(normalized)) return false
  if (normalized.includes('定投')) return true
  return undefined
}

function parseWealthRemarkMeta(item: Investment): WealthRemarkMeta {
  const remark = normalizedRemark(item)
  const fallbackName = (item.name || item.organization_name || '').trim()
  const meta: WealthRemarkMeta = {
    sip: (item.name || '').includes('定投'),
    name: fallbackName || undefined
  }

  if (!remark) {
    return meta
  }

  const tokens = splitRemarkTokens(remark)
  const keyValuePattern = /^([^:=：]{1,16})[:=：]\s*(.+)$/
  for (const token of tokens) {
    const keyValue = token.match(keyValuePattern)
    if (keyValue) {
      const key = keyValue[1].trim().toLowerCase()
      const value = keyValue[2].trim()
      if (!value) continue

      if (/(定投|sip|自动投|自动扣款)/.test(key)) {
        const sip = parseSipValue(value)
        if (sip !== undefined) meta.sip = sip
        if (sip === undefined && value.includes('定投')) meta.sip = true
        continue
      }

      if (/(周期|频率|投频|扣款)/.test(key)) {
        meta.period = extractPeriodFromText(value) || normalizePeriodText(value)
        continue
      }

      if (/(名称|备注|标的|产品|理财)/.test(key) && !meta.name) {
        meta.name = value
      }
      continue
    }

    if (token.includes('定投')) {
      meta.sip = true
    }

    if (!meta.period) {
      const period = extractPeriodFromText(token)
      if (period) meta.period = period
    }

    if ((!meta.name || meta.name === fallbackName) && token.length >= 2) {
      if (!/(周期|每\d|每[天日周月季年]|周[一二三四五六日天]|月初|月中|月末)/.test(token)) {
        meta.name = token
      }
    }
  }

  if (!meta.period) {
    meta.period = extractPeriodFromText(remark)
  }

  return meta
}

function wealthSIPLabel(item: Investment) {
  return parseWealthRemarkMeta(item).sip ? '是' : '否'
}

function wealthPeriodLabel(item: Investment) {
  return parseWealthRemarkMeta(item).period || '--'
}

function wealthRemarkName(item: Investment) {
  return parseWealthRemarkMeta(item).name || '--'
}

function formatTransactionInvestment(row: InvestmentTransaction) {
  if (row.investment_name && row.investment_code) {
    return `${row.investment_name} (${row.investment_code})`
  }
  if (row.investment_name) return row.investment_name
  if (row.investment_code) return row.investment_code
  return `投资#${row.investment_id}`
}

function resetForm() {
  editingId.value = null
  editingInvestmentName.value = ''
  form.value = {
    current_price: '',
    market_value: '',
    total_shares: '',
    total_cost: ''
  }
}

function openEdit(item: Investment) {
  editingId.value = item.id
  editingInvestmentName.value = displayInvestmentName(item)
  form.value = {
    current_price: item.current_price || '',
    market_value: item.market_value || '',
    total_shares: item.total_shares || '',
    total_cost: item.total_cost || ''
  }
  dialogVisible.value = true
}

async function submitEdit() {
  if (!editingId.value) return
  try {
    await updateInvestment(editingId.value, {
      current_price: form.value.current_price.trim(),
      market_value: form.value.market_value.trim(),
      total_shares: form.value.total_shares.trim(),
      total_cost: form.value.total_cost.trim()
    })
    dialogVisible.value = false
    await loadData()
    ElMessage.success('投资数据已更新')
  } catch (error: any) {
    ElMessage.error(error?.response?.data?.message || '投资更新失败')
  }
}

async function loadData() {
  loading.value = true
  try {
    const data = await fetchInvestments({
      user_id: !isAdmin.value || onlyMine.value ? currentUserId : undefined,
      page: currentPage.value,
      page_size: pageSize.value,
      investment_type: tab.value,
      show_sold: showSold.value,
      keyword: keyword.value || undefined
    })
    investments.value = data

    const summary = await fetchDashboardSummary({
      user_id: currentUserId,
      start_date: `${today.slice(0, 8)}01`,
      end_date: today
    })
    stockIdleCash.value = summary.stock_idle_cash
  } finally {
    loading.value = false
  }
}

function search() {
  currentPage.value = 1
  void loadData()
}

function handleTabChange() {
  currentPage.value = 1
  void loadData()
}

function goDashboard() {
  void router.push({ name: 'dashboard' })
}

async function loadTransactions() {
  transactions.value = await fetchInvestmentTransactions(
    transactionsKeyword.value,
    transactionsPage.value,
    transactionsPageSize.value
  )
}

function searchTransactions() {
  transactionsPage.value = 1
  void loadTransactions()
}

async function openTransactionsDrawer() {
  transactionsDrawerVisible.value = true
  transactionsPage.value = 1
  await loadTransactions()
}

async function loadTopRecommendations() {
  topRecommendations.value = await fetchTopInvestments(
    topInvestmentType.value || undefined,
    !isAdmin.value || topOnlyMine.value ? currentUserId : undefined
  )
  topCurrentPage.value = 1
}

function applyTopKeyword() {
  topKeywordApplied.value = topKeyword.value
  topCurrentPage.value = 1
}

async function openTopDrawer() {
  if (!topInvestmentType.value) {
    topInvestmentType.value = tab.value
  }
  topDrawerVisible.value = true
  await loadTopRecommendations()
}

onMounted(async () => {
  try {
    if (route.query.tab === 'stock' || route.query.tab === 'wealth') {
      tab.value = route.query.tab
    }
    if (typeof route.query.keyword === 'string') {
      keyword.value = route.query.keyword
    }
    if (typeof route.query.show_sold === 'string') {
      showSold.value = route.query.show_sold === 'true'
    }
    if (typeof route.query.only_mine === 'string') {
      onlyMine.value = route.query.only_mine !== 'false'
    }
    await loadData()
    const openDrawer = route.query.open_drawer
    if (openDrawer === 'transactions') {
      await openTransactionsDrawer()
    } else if (openDrawer === 'top20') {
      await openTopDrawer()
    }
  } catch {
    ElMessage.error('投资页初始化失败')
  }
})
</script>

<style scoped>
.page {
  position: relative;
  padding: 24px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(246, 249, 252, 0.74)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.05), rgba(255, 190, 102, 0.06));
}

.page::before,
.page::after {
  content: "";
  position: absolute;
  border-radius: 50%;
  pointer-events: none;
  filter: blur(8px);
}

.page::before {
  top: 22px;
  right: 3%;
  width: 190px;
  height: 190px;
  background: radial-gradient(circle, rgba(76, 132, 255, 0.12) 0%, rgba(76, 132, 255, 0) 72%);
  animation: investmentsHalo 15s ease-in-out infinite alternate;
}

.page::after {
  top: 180px;
  left: -42px;
  width: 150px;
  height: 150px;
  background: radial-gradient(circle, rgba(255, 190, 102, 0.12) 0%, rgba(255, 190, 102, 0) 72%);
  animation: investmentsHalo 18s ease-in-out infinite alternate-reverse;
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 16px;
  padding: 18px 20px;
  border-radius: 20px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.84), rgba(247, 250, 253, 0.72)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.08), rgba(255, 190, 102, 0.1));
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.54);
}

.head h2 {
  margin: 0;
  font-size: 28px;
  letter-spacing: -0.04em;
  background: linear-gradient(135deg, #173454 0%, #2d5587 46%, #8b6b3f 100%);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.subhead {
  margin: 8px 0 0;
  color: #64748b;
  font-size: 13px;
  max-width: 620px;
}

.actions,
.drawer-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 10px 12px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.38);
  border: 1px solid rgba(148, 163, 184, 0.16);
  backdrop-filter: blur(12px);
}

.summary-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  padding: 12px 14px;
  border-radius: 18px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.82), rgba(246, 249, 252, 0.72)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.06), rgba(255, 190, 102, 0.07));
  border: 1px solid rgba(148, 163, 184, 0.16);
}

.pagination-wrap {
  margin-top: 18px;
  display: flex;
  justify-content: flex-end;
  padding: 12px 14px 0;
}

.context-banner {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px;
  border-radius: 18px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.82), rgba(246, 249, 252, 0.72)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.06), rgba(255, 190, 102, 0.07));
  border: 1px solid rgba(148, 163, 184, 0.16);
}

.context-actions {
  display: flex;
  justify-content: flex-end;
}

.page :deep(.el-tabs__nav-wrap::after) {
  height: 1px;
  background: rgba(148, 163, 184, 0.16);
}

.page :deep(.el-tabs__item.is-active) {
  transform: translateY(-1px);
}

.page :deep(.el-tag) {
  border-radius: 999px;
}

.page :deep(.el-table) {
  border-radius: 18px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.6);
}

.page :deep(.el-table th.el-table__cell) {
  background: linear-gradient(180deg, rgba(244, 248, 252, 0.92), rgba(236, 242, 248, 0.9));
}

.page :deep(.el-drawer__body),
.page :deep(.el-dialog__body) {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.94), rgba(247, 250, 253, 0.84)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.04), rgba(255, 190, 102, 0.05));
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

@keyframes investmentsHalo {
  from {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.46;
  }
  to {
    transform: translate3d(14px, -10px, 0) scale(1.12);
    opacity: 0.88;
  }
}

@media (max-width: 900px) {
  .head {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
