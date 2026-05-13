<template>
  <section class="panel page">
    <div class="head">
      <h2>TOP20</h2>
      <div class="actions">
        <el-select v-model="investmentType" style="width: 160px;" @change="loadData">
          <el-option label="全部" value="" />
          <el-option label="股票" value="stock" />
          <el-option label="理财" value="wealth" />
        </el-select>
        <el-input v-model="keyword" placeholder="全文检索（名称/代码/ID）" style="width: 260px;" />
        <el-button @click="applyKeyword">查询</el-button>
      </div>
    </div>

    <el-table :data="filteredList" stripe>
      <el-table-column prop="investment_id" label="ID" width="90" />
      <el-table-column label="类型" width="120">
        <template #default="{ row }">
          <el-tag>{{ row.investment_type === 'stock' ? '股票' : '理财' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="name" label="名称" min-width="180" />
      <el-table-column prop="code" label="代码" width="120" />
      <el-table-column prop="score" label="评分" width="90" />
      <el-table-column prop="suggestion" label="建议" width="120" />
      <el-table-column prop="total_profit_rate" label="收益率" width="120" />
      <el-table-column prop="reason" label="原因" min-width="320" show-overflow-tooltip />
    </el-table>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchTopInvestments, type InvestmentRecommendation } from '@/api/investments'

const recommendations = ref<InvestmentRecommendation[]>([])
const filteredList = computed(() => {
  const k = keywordApplied.value.trim().toLowerCase()
  if (!k) return recommendations.value
  return recommendations.value.filter((row) =>
    `${row.investment_id} ${row.name} ${row.code} ${row.reason}`.toLowerCase().includes(k)
  )
})
const investmentType = ref('')
const keyword = ref('')
const keywordApplied = ref('')

async function loadData() {
  recommendations.value = await fetchTopInvestments(investmentType.value || undefined)
}

function applyKeyword() {
  keywordApplied.value = keyword.value
}

onMounted(async () => {
  try {
    await loadData()
  } catch {
    ElMessage.error('TOP20 页面初始化失败')
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
}
</style>
