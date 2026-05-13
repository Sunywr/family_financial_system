<template>
  <section class="panel page">
    <div class="head">
      <h2>资产</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（名称/备注/ID）" style="width: 280px;" />
        <el-button @click="loadData">查询</el-button>
      </div>
    </div>
    <el-table :data="assets.list" stripe>
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
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchAssets, type Asset } from '@/api/assets'

const keyword = ref('')
const assets = ref<{ list: Asset[]; total: number }>({ list: [], total: 0 })
const statusMap: Record<string, string> = { active: '使用中', archived: '已归档' }

async function loadData() {
  assets.value = await fetchAssets(keyword.value)
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
