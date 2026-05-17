<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">情报</h2>
        <p style="margin: 8px 0 0; color: #64748b;">占位扩展模块，当前支持人工记录和状态管理。</p>
      </div>
      <el-button type="primary" @click="openCreate">新增情报</el-button>
    </div>

    <el-table v-loading="loading" :data="intel.list" stripe>
      <el-table-column prop="title" label="标题" min-width="220" />
      <el-table-column prop="source" label="来源" width="140" />
      <el-table-column prop="item_date" label="日期" width="120" />
      <el-table-column prop="status" label="状态" width="100" />
      <el-table-column prop="summary" label="摘要" min-width="220" show-overflow-tooltip />
      <el-table-column label="操作" width="170">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" type="danger" @click="removeIntel(row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="intel.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadIntel"
        @size-change="loadIntel"
      />
    </div>

    <el-dialog v-model="dialogVisible" :title="editingId ? '编辑情报' : '新增情报'" width="640px">
      <el-form label-width="90px">
        <el-form-item label="用户 ID">
          <el-input-number v-model="form.user_id" :min="1" :disabled="Boolean(editingId)" />
        </el-form-item>
        <el-form-item label="标题">
          <el-input v-model="form.title" />
        </el-form-item>
        <el-form-item label="来源">
          <el-input v-model="form.source" />
        </el-form-item>
        <el-form-item label="日期">
          <el-date-picker v-model="form.item_date" type="date" value-format="YYYY-MM-DD" style="width: 100%;" />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="form.status" style="width: 100%;">
            <el-option label="草稿" value="draft" />
            <el-option label="启用" value="active" />
            <el-option label="归档" value="archived" />
          </el-select>
        </el-form-item>
        <el-form-item label="标签">
          <el-input v-model="form.tags" placeholder="逗号分隔" />
        </el-form-item>
        <el-form-item label="摘要">
          <el-input v-model="form.summary" type="textarea" :rows="2" />
        </el-form-item>
        <el-form-item label="内容">
          <el-input v-model="form.content" type="textarea" :rows="5" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitForm">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { requireCurrentUserId } from '@/api/client'
import { createIntel, deleteIntel, fetchIntel, updateIntel, type IntelItem } from '@/api/intel'

const intel = ref<{ list: IntelItem[]; total: number }>({ list: [], total: 0 })
const currentPage = ref(1)
const pageSize = ref(50)
const loading = ref(false)
const dialogVisible = ref(false)
const editingId = ref<number | null>(null)
const currentUserId = requireCurrentUserId()
const form = ref({
  user_id: currentUserId,
  title: '',
  source: '',
  item_date: new Date().toISOString().slice(0, 10),
  status: 'draft',
  tags: '',
  summary: '',
  content: ''
})

function resetForm() {
  form.value = {
    user_id: currentUserId,
    title: '',
    source: '',
    item_date: new Date().toISOString().slice(0, 10),
    status: 'draft',
    tags: '',
    summary: '',
    content: ''
  }
}

async function loadIntel() {
  loading.value = true
  try {
    const data = await fetchIntel(currentPage.value, pageSize.value)
    intel.value = { list: data.list, total: data.total }
  } finally {
    loading.value = false
  }
}

function openCreate() {
  editingId.value = null
  resetForm()
  dialogVisible.value = true
}

function openEdit(row: IntelItem) {
  editingId.value = row.id
  form.value = {
    user_id: row.user_id,
    title: row.title,
    source: row.source ?? '',
    item_date: row.item_date,
    status: row.status,
    tags: row.tags ?? '',
    summary: row.summary ?? '',
    content: row.content ?? ''
  }
  dialogVisible.value = true
}

async function submitForm() {
  try {
    const payload = {
      ...form.value,
      source: form.value.source || null,
      tags: form.value.tags || null,
      summary: form.value.summary || null,
      content: form.value.content || null
    }
    if (editingId.value) {
      await updateIntel(editingId.value, {
        title: payload.title,
        source: payload.source,
        item_date: payload.item_date,
        status: payload.status,
        tags: payload.tags,
        summary: payload.summary,
        content: payload.content
      })
    } else {
      await createIntel(payload)
    }
    dialogVisible.value = false
    await loadIntel()
    ElMessage.success('情报已保存')
  } catch {
    ElMessage.error('情报保存失败')
  }
}

async function removeIntel(id: number) {
  try {
    await deleteIntel(id)
    await loadIntel()
    ElMessage.success('情报已删除')
  } catch {
    ElMessage.error('删除失败')
  }
}

onMounted(async () => {
  try {
    await loadIntel()
  } catch {
    ElMessage.error('情报页面初始化失败')
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
