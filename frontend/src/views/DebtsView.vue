<template>
  <section class="panel page">
    <div class="head">
      <h2>债务</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（分类/备注/ID）" style="width: 280px;" />
        <el-button @click="search">查询</el-button>
      </div>
    </div>

    <div v-if="dashboardContextLabel" class="context-banner">
      <el-alert :title="`当前筛选来自首页：${dashboardContextLabel}`" type="info" :closable="false" show-icon />
      <div class="context-actions">
        <el-button text type="primary" @click="goDashboard">返回首页</el-button>
      </div>
    </div>

    <el-table :data="debts.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column prop="start_date" label="开始日期" width="120" />
      <el-table-column prop="end_date" label="结束日期" width="120">
        <template #default="{ row }">
          {{ row.end_date || '--' }}
        </template>
      </el-table-column>
      <el-table-column prop="repay_deadline" label="还款截止日期" width="140">
        <template #default="{ row }">
          {{ row.repay_deadline || '--' }}
        </template>
      </el-table-column>
      <el-table-column prop="category_name" label="分类" width="120" />
      <el-table-column prop="amount" label="金额" width="120" />
      <el-table-column label="单位" width="120">
        <template #default="{ row }">
          <el-tag>{{ periodMap[row.period_unit] || row.period_unit }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="120">
        <template #default="{ row }">
          <el-tag :type="row.status === 'pending' ? 'warning' : 'success'">
            {{ statusMap[row.status] || row.status }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="remark" label="备注" min-width="220" />
    </el-table>

    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="debts.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadData"
        @size-change="loadData"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { fetchDebts, type Debt } from '@/api/debts'

const route = useRoute()
const router = useRouter()
const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const debts = ref<{ list: Debt[]; total: number }>({ list: [], total: 0 })
const periodMap: Record<string, string> = { day: '天', week: '周', month: '月', year: '年' }
const statusMap: Record<string, string> = { pending: '待还', settled: '已还', cancelled: '已取消' }
const dashboardContextMap: Record<string, string> = {
  'cycle-debt-summary': '周期债务摘要',
  'cycle-debt-row': '周期债务明细'
}
const dashboardContextLabel =
  route.query.from === 'dashboard' && typeof route.query.context === 'string'
    ? dashboardContextMap[route.query.context] || '首页钻取'
    : ''

async function loadData() {
  debts.value = await fetchDebts(keyword.value, currentPage.value, pageSize.value)
}

function search() {
  currentPage.value = 1
  loadData()
}

function goDashboard() {
  void router.push({ name: 'dashboard' })
}

onMounted(async () => {
  try {
    if (typeof route.query.keyword === 'string') {
      keyword.value = route.query.keyword
    }
    await loadData()
  } catch {
    ElMessage.error('债务页面初始化失败')
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
}

.pagination-wrap {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.context-banner {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.context-actions {
  display: flex;
  justify-content: flex-end;
}
</style>
