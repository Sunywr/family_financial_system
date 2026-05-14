<template>
  <section class="panel page">
    <div class="head">
      <h2>投资仪表盘</h2>
      <div class="actions">
        <el-switch
          v-if="isAdmin"
          v-model="onlyMine"
          active-text="仅看我购买"
          inactive-text="查看全部"
          @change="search"
        />
        <el-switch v-model="showSold" active-text="展示已出售" @change="loadData" />
        <el-input v-model="keyword" placeholder="全文检索（名称/代码/ID）" style="width: 280px;" />
        <el-button @click="search">查询</el-button>
      </div>
    </div>

    <el-tabs v-model="tab" @tab-change="search">
      <el-tab-pane label="股票" name="stock" />
      <el-tab-pane label="理财" name="wealth" />
    </el-tabs>

    <div v-if="tab === 'stock'" class="summary-row">
      <el-tag type="info">市值：{{ stockMarketValue }}</el-tag>
      <el-tag type="warning">股票账户闲置资金：{{ stockIdleCash }}</el-tag>
      <el-tag type="success">总收益金额：{{ stockTotalProfit }}</el-tag>
    </div>

    <el-table :data="investments.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column label="名称" min-width="180">
        <template #default="{ row }">
          {{ displayInvestmentName(row) }}
        </template>
      </el-table-column>
      <el-table-column prop="code" label="代码" width="120" />
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
            {{ row.status === 'holding' ? '持有中' : '已出售' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="market_value" label="市值" width="120" />
      <el-table-column prop="total_profit" label="收益金额" width="120" />
      <el-table-column label="收益率" width="120">
        <template #default="{ row }">
          {{ formatRate(row) }}
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'stock'" label="仓位" width="120">
        <template #default="{ row }">
          {{ formatPosition(row) }}
        </template>
      </el-table-column>
      <el-table-column v-if="tab === 'stock'" label="备注" min-width="220">
        <template #default="{ row }">
          定投周期：--；坝基金额：--
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
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchDashboardSummary } from '@/api/dashboard'
import { fetchInvestments, type Investment } from '@/api/investments'
import { getStoredUser, requireCurrentUserId } from '@/api/client'

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

const investments = ref<{ list: Investment[]; total: number }>({ list: [], total: 0 })
const stockIdleCash = ref('0.00')

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

function stockBoardLabel(code: string) {
  if (code.startsWith('6')) return '沪A'
  if (code.startsWith('3')) return '创业'
  return '深A'
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
  const val = Number.parseFloat(item.total_profit_rate || '0') * 100
  if (tab.value === 'stock') {
    return `${val.toFixed(2)}%`
  }
  return `${val.toFixed(2)}%`
}

function formatPosition(item: Investment) {
  const marketValue = Number.parseFloat(item.market_value || '0')
  const total = Number.parseFloat(stockMarketValue.value || '0')
  if (total <= 0) return '0.00%'
  return `${((marketValue / total) * 100).toFixed(2)}%`
}

function displayInvestmentName(item: Investment) {
  if (tab.value === 'stock' && item.name === item.organization_name) {
    return `股票 ${item.code}`
  }
  return item.name || item.organization_name || item.code
}

async function loadData() {
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
    start_date: today.slice(0, 8) + '01',
    end_date: today
  })
  stockIdleCash.value = summary.stock_idle_cash
}

function search() {
  currentPage.value = 1
  loadData()
}

onMounted(async () => {
  try {
    await loadData()
  } catch {
    ElMessage.error('投资页初始化失败')
  }
})
</script>

<style scoped>
.page {
  padding: 24px;
}
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  gap: 16px;
}
.head h2 {
  margin: 0;
}
.actions {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}
.summary-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
}
.pagination-wrap {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
