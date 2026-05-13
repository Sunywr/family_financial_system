<template>
  <section class="panel jobs-page">
    <div class="toolbar">
      <div>
        <h2>任务管理</h2>
        <p>支持任务配置查看、更新和手动触发执行。</p>
      </div>
      <el-tag>{{ jobs.total }} jobs</el-tag>
    </div>

    <el-table :data="jobs.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="job_name" label="任务名称" min-width="170" />
      <el-table-column prop="job_code" label="任务编码" min-width="180" />
      <el-table-column prop="cron_expr" label="Cron" min-width="180" />
      <el-table-column label="启用" width="100">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'">
            {{ row.enabled ? '是' : '否' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="210">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" type="primary" @click="runJob(row.id)">执行</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" title="编辑任务配置" width="560px">
      <el-form label-width="130px">
        <el-form-item label="任务名称">
          <el-input v-model="form.job_name" disabled />
        </el-form-item>
        <el-form-item label="Cron 表达式">
          <el-input v-model="form.cron_expr" />
        </el-form-item>
        <el-form-item label="批大小">
          <el-input-number v-model="form.batch_size" :min="1" />
        </el-form-item>
        <el-form-item label="并发数">
          <el-input-number v-model="form.concurrency" :min="1" />
        </el-form-item>
        <el-form-item label="超时(秒)">
          <el-input-number v-model="form.timeout_seconds" :min="10" />
        </el-form-item>
        <el-form-item label="重试次数">
          <el-input-number v-model="form.retry_count" :min="0" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="form.enabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submit">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchJobs, triggerJob, updateJob, type JobConfig } from '@/api/jobs'

const jobs = ref<{ list: JobConfig[]; total: number }>({ list: [], total: 0 })
const dialogVisible = ref(false)
const editingId = ref<number | null>(null)
const form = ref({
  job_name: '',
  cron_expr: '',
  enabled: true,
  batch_size: 100,
  concurrency: 1,
  timeout_seconds: 60,
  retry_count: 0
})

async function loadJobs() {
  const data = await fetchJobs()
  jobs.value = { list: data.list, total: data.total }
}

function openEdit(row: JobConfig) {
  editingId.value = row.id
  form.value = {
    job_name: row.job_name,
    cron_expr: row.cron_expr,
    enabled: row.enabled,
    batch_size: row.batch_size,
    concurrency: row.concurrency,
    timeout_seconds: row.timeout_seconds,
    retry_count: row.retry_count
  }
  dialogVisible.value = true
}

async function submit() {
  if (!editingId.value) return
  try {
    await updateJob(editingId.value, {
      cron_expr: form.value.cron_expr,
      enabled: form.value.enabled,
      batch_size: form.value.batch_size,
      concurrency: form.value.concurrency,
      timeout_seconds: form.value.timeout_seconds,
      retry_count: form.value.retry_count
    })
    dialogVisible.value = false
    await loadJobs()
    ElMessage.success('任务配置已更新')
  } catch {
    ElMessage.error('任务配置更新失败')
  }
}

async function runJob(id: number) {
  try {
    await triggerJob(id)
    ElMessage.success('任务已触发')
  } catch {
    ElMessage.error('任务触发失败')
  }
}

onMounted(async () => {
  try {
    await loadJobs()
  } catch {
    ElMessage.error('任务页面初始化失败')
  }
})
</script>

<style scoped>
.jobs-page {
  padding: 24px;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
}
.toolbar h2 {
  margin: 0;
}
.toolbar p {
  margin: 8px 0 0;
  color: #64748b;
}
</style>
