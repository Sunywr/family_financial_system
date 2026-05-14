<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">任务日志</h2>
        <p style="margin: 8px 0 0; color: #64748b;">展示最近任务执行记录和结果。</p>
      </div>
      <el-tag>{{ runs.total }} runs</el-tag>
    </div>

    <el-table :data="runs.list" stripe>
      <el-table-column prop="job_id" label="任务ID" width="90" />
      <el-table-column prop="status" label="状态" width="120" />
      <el-table-column prop="trigger_type" label="触发方式" width="120" />
      <el-table-column prop="started_at" label="开始时间" min-width="180" />
      <el-table-column prop="duration_ms" label="耗时ms" width="100" />
      <el-table-column prop="message" label="消息" min-width="220" />
      <el-table-column prop="error_message" label="错误" min-width="220" />
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="runs.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadRuns"
        @size-change="loadRuns"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchJobRuns, type JobRun } from '@/api/jobs'

const runs = ref<{ list: JobRun[]; total: number }>({ list: [], total: 0 })
const currentPage = ref(1)
const pageSize = ref(20)

async function loadRuns() {
  const data = await fetchJobRuns(currentPage.value, pageSize.value)
  runs.value = { list: data.list, total: data.total }
}

onMounted(async () => {
  try {
    await loadRuns()
  } catch {
    ElMessage.error('任务日志页面初始化失败')
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
