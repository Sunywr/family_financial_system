<template>
  <section class="panel page">
    <div class="head">
      <h2>债务</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（分类/备注/ID）" style="width: 280px;" />
        <el-button @click="loadData">查询</el-button>
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
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchDebts, type Debt } from '@/api/debts'

const keyword = ref('')
const debts = ref<{ list: Debt[]; total: number }>({ list: [], total: 0 })
const periodMap: Record<string, string> = { day: '天', week: '周', month: '月', year: '年' }
const statusMap: Record<string, string> = { pending: '待还', settled: '已还', cancelled: '已取消' }

async function loadData() {
  debts.value = await fetchDebts(keyword.value)
}

onMounted(async () => {
  try {
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
</style>
