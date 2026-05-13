<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">品牌红黑榜</h2>
        <p style="margin: 8px 0 0; color: #64748b;">阶段七已接入品牌评价分页接口。</p>
      </div>
      <el-tag>{{ brands.total }} brands</el-tag>
    </div>

    <el-table :data="brands.list" stripe>
      <el-table-column prop="brand_name" label="品牌" min-width="180" />
      <el-table-column prop="category_name" label="分类" width="140" />
      <el-table-column prop="score" label="评分" width="100" />
      <el-table-column prop="board_type" label="榜单" width="120" />
      <el-table-column prop="review" label="评价" min-width="220" />
      <el-table-column prop="remark" label="备注" min-width="200" />
    </el-table>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchBrands, type Brand } from '@/api/brands'

const brands = ref<{ list: Brand[]; total: number }>({ list: [], total: 0 })

onMounted(async () => {
  try {
    const data = await fetchBrands()
    brands.value = { list: data.list, total: data.total }
  } catch {
    ElMessage.error('品牌页面初始化失败')
  }
})
</script>
