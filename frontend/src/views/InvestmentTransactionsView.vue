<template>
  <section class="panel page">
    <div class="head">
      <h2>股票流水</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（动作/备注/ID）" style="width: 280px;" />
        <el-button @click="search">查询</el-button>
      </div>
    </div>
    <el-table :data="transactions.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column label="投资" min-width="180">
        <template #default="{ row }">
          {{ formatInvestment(row) }}
        </template>
      </el-table-column>
      <el-table-column prop="source_bill_id" label="来源账单ID" width="120" />
      <el-table-column prop="transaction_date" label="日期" width="120" />
      <el-table-column label="动作" width="120">
        <template #default="{ row }">
          <el-tag>{{ actionMap[row.action] || row.action }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="shares" label="份额" width="120" />
      <el-table-column prop="amount" label="金额" width="120" />
      <el-table-column prop="unit_price" label="单价" width="120" />
      <el-table-column prop="realized_profit" label="已实现收益" width="140" />
      <el-table-column label="备注" min-width="240">
        <template #default="{ row }">
          {{ row.remark || '' }}（手续费并入操作记录）
        </template>
      </el-table-column>
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="transactions.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadData"
        @size-change="loadData"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import {
  fetchInvestmentTransactions,
  type InvestmentTransaction
} from '@/api/investment-transactions'

const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(50)
const transactions = ref<{ list: InvestmentTransaction[]; total: number }>({ list: [], total: 0 })
const actionMap: Record<string, string> = {
  open_position: '建仓',
  add_position: '加仓',
  reduce_position: '减仓',
  dividend: '分红'
}

function formatInvestment(row: InvestmentTransaction) {
  if (row.investment_name && row.investment_code) {
    return `${row.investment_name} (${row.investment_code})`
  }
  if (row.investment_name) return row.investment_name
  if (row.investment_code) return row.investment_code
  return `投资#${row.investment_id}`
}

async function loadData() {
  transactions.value = await fetchInvestmentTransactions(keyword.value, currentPage.value, pageSize.value)
}

function search() {
  currentPage.value = 1
  loadData()
}

onMounted(async () => {
  try {
    await loadData()
  } catch {
    ElMessage.error('股票流水页面初始化失败')
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
</style>
