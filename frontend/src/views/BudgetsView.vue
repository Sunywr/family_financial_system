<template>
  <section class="panel page">
    <div class="head">
      <div>
        <h2>预算</h2>
        <p>阶段七已接入预算列表和按月自动生成接口。</p>
      </div>
      <div class="actions">
        <el-tag>{{ budgets.total }} items</el-tag>
        <el-button type="primary" @click="generate">生成本月预算</el-button>
      </div>
    </div>

    <el-table v-loading="loading" :data="budgets.list" stripe>
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
const loading = ref(false)

async function loadBudgets() {
  loading.value = true
  try {
    const data = await fetchBudgets(currentPage.value, pageSize.value)
    budgets.value = { list: data.list, total: data.total }
  } finally {
    loading.value = false
  }
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
.page {
  position: relative;
  padding: 24px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(246, 249, 252, 0.74)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.05), rgba(255, 190, 102, 0.06));
}

.page::before {
  content: "";
  position: absolute;
  top: 24px;
  right: 4%;
  width: 170px;
  height: 170px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 190, 102, 0.12) 0%, rgba(255, 190, 102, 0) 72%);
  filter: blur(8px);
  pointer-events: none;
  animation: budgetsHalo 16s ease-in-out infinite alternate;
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
  gap: 16px;
  flex-wrap: wrap;
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

.head p {
  margin: 8px 0 0;
  color: #64748b;
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
  padding: 10px 12px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.38);
  border: 1px solid rgba(148, 163, 184, 0.16);
  backdrop-filter: blur(12px);
}

.page :deep(.el-table) {
  border-radius: 18px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.6);
}

.page :deep(.el-table th.el-table__cell) {
  background: linear-gradient(180deg, rgba(244, 248, 252, 0.92), rgba(236, 242, 248, 0.9));
}

.pagination-wrap {
  margin-top: 18px;
  display: flex;
  justify-content: flex-end;
  padding: 12px 14px 0;
}

@keyframes budgetsHalo {
  from {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.46;
  }
  to {
    transform: translate3d(14px, -10px, 0) scale(1.12);
    opacity: 0.88;
  }
}
</style>
