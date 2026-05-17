<template>
  <section class="panel page">
    <div class="head">
      <h2>预售</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（分类/备注/ID）" style="width: 280px;" />
        <el-button @click="search">查询</el-button>
      </div>
    </div>
    <el-table v-loading="loading" :data="presales.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column prop="deposit_date" label="定金日期" width="120" />
      <el-table-column prop="final_payment_date" label="尾款日期" width="120" />
      <el-table-column prop="category_name" label="分类" width="120" />
      <el-table-column prop="deposit_amount" label="定金" width="120" />
      <el-table-column prop="final_payment_amount" label="尾款" width="120" />
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
        :total="presales.total"
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
import { fetchPresales, type Presale } from '@/api/presales'

const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const presales = ref<{ list: Presale[]; total: number }>({ list: [], total: 0 })
const statusMap: Record<string, string> = { pending: '待完成', settled: '已完成' }

async function loadData() {
  loading.value = true
  try {
    presales.value = await fetchPresales(keyword.value, currentPage.value, pageSize.value)
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
    ElMessage.error('预售页面初始化失败')
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
