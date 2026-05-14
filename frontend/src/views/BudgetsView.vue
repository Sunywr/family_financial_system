<template>
  <section class="panel" style="padding: 24px;">
    <div
      style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; gap: 16px;"
    >
      <div>
        <h2 style="margin: 0;">预算</h2>
        <p style="margin: 8px 0 0; color: #64748b;">阶段七已接入预算列表和按月自动生成接口。</p>
      </div>
      <el-button type="primary" @click="generate">生成本月预算</el-button>
    </div>

    <el-table :data="budgets.list" stripe>
      <el-table-column prop="category_name" label="分类" min-width="160" />
      <el-table-column prop="planned_amount" label="预算" width="120" />
      <el-table-column prop="actual_amount" label="实际" width="120" />
      <el-table-column label="状态" width="120">
        <template #default="{ row }">
          <el-tag :type="Number(row.actual_amount) > Number(row.planned_amount) ? 'danger' : 'success'">
            {{ Number(row.actual_amount) > Number(row.planned_amount) ? '超支' : '正常' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="手调" width="100">
        <template #default="{ row }">
          <el-tag :type="row.manual_adjusted ? 'warning' : 'info'">
            {{ row.manual_adjusted ? '是' : '否' }}
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
        :total="budgets.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadBudgets"
        @size-change="loadBudgets"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { requireCurrentUserId } from '@/api/client'
import { fetchBudgets, generateBudgets, type Budget } from '@/api/budgets'

const budgets = ref<{ list: Budget[]; total: number }>({ list: [], total: 0 })
const currentPage = ref(1)
const pageSize = ref(50)

async function loadBudgets() {
  const data = await fetchBudgets(currentPage.value, pageSize.value)
  budgets.value = { list: data.list, total: data.total }
}

async function generate() {
  try {
    const month = new Date().toISOString().slice(0, 7) + '-01'
    const result = await generateBudgets(requireCurrentUserId(), month)
    ElMessage.success(`已生成 ${result.generated} 条预算`)
    await loadBudgets()
  } catch {
    ElMessage.error('预算生成失败')
  }
}

onMounted(async () => {
  try {
    await loadBudgets()
  } catch {
    ElMessage.error('预算页面初始化失败')
  }
})
</script>

<style scoped>
.pagination-wrap {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
