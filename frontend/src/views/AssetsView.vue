<template>
  <section class="panel page">
    <div class="head">
      <h2>资产</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（名称/备注/ID）" style="width: 280px;" />
        <el-button @click="search">查询</el-button>
      </div>
    </div>
    <el-table v-loading="loading" :data="assets.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column prop="created_at" label="创建日期" width="170" />
      <el-table-column prop="name" label="名称" min-width="180" />
      <el-table-column prop="category_name" label="分类" width="140" />
      <el-table-column prop="amount" label="金额" width="120" />
      <el-table-column label="状态" width="120">
        <template #default="{ row }">
          <el-tag :type="row.status === 'active' ? 'success' : 'info'">
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
        :total="assets.total"
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
import { fetchAssets, type Asset } from '@/api/assets'

const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const assets = ref<{ list: Asset[]; total: number }>({ list: [], total: 0 })
const statusMap: Record<string, string> = { active: '使用中', archived: '已归档' }

async function loadData() {
  loading.value = true
  try {
    assets.value = await fetchAssets(keyword.value, currentPage.value, pageSize.value)
  } finally {
    loading.value = false
  }
}

function search() {
  currentPage.value = 1
  loadData()
}

onMounted(async () => {
  try {
    await loadData()
  } catch {
    ElMessage.error('资产页面初始化失败')
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
  top: 22px;
  right: 3%;
  width: 180px;
  height: 180px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(76, 132, 255, 0.12) 0%, rgba(76, 132, 255, 0) 72%);
  filter: blur(8px);
  pointer-events: none;
  animation: assetsHalo 16s ease-in-out infinite alternate;
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  gap: 16px;
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
.actions {
  display: flex;
  gap: 10px;
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

@keyframes assetsHalo {
  from {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.46;
  }
  to {
    transform: translate3d(-12px, 12px, 0) scale(1.1);
    opacity: 0.88;
  }
}
</style>
