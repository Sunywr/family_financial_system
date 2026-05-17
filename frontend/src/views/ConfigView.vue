<template>
  <section class="panel config-page">
    <div class="toolbar">
      <div>
        <h2>配置项管理</h2>
        <p>支持手动创建、修改、删除非内置配置项。</p>
      </div>
      <div class="toolbar-actions">
        <el-select v-model="filterType" clearable placeholder="按类型筛选" style="width: 220px;">
          <el-option v-for="item in typeOptions" :key="item.code" :label="item.label" :value="item.code" />
        </el-select>
        <el-button type="primary" @click="openCreate">新增配置项</el-button>
      </div>
    </div>

    <el-table v-loading="loading" :data="items.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="config_type" label="类型" min-width="160" />
      <el-table-column prop="display_name" label="显示名" min-width="160" />
      <el-table-column prop="name" label="编码" min-width="160" />
      <el-table-column label="启用" width="100">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'">{{ row.enabled ? '是' : '否' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="内置" width="100">
        <template #default="{ row }">
          <el-tag :type="row.is_builtin ? 'warning' : 'info'">{{ row.is_builtin ? '是' : '否' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="sort_order" label="排序" width="100" />
      <el-table-column label="操作" width="180">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
          <el-button
            size="small"
            type="danger"
            :disabled="row.is_builtin"
            @click="remove(row)"
          >
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="items.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadItemsOnly"
        @size-change="loadItemsOnly"
      />
    </div>

    <el-dialog v-model="dialogVisible" :title="editingId ? '编辑配置项' : '新增配置项'" width="520px">
      <el-form label-width="100px">
        <el-form-item label="类型">
          <el-select
            v-model="form.config_type"
            :disabled="Boolean(editingId)"
            placeholder="请选择类型"
            style="width: 100%;"
          >
            <el-option v-for="item in typeOptions" :key="item.code" :label="item.label" :value="item.code" />
          </el-select>
        </el-form-item>
        <el-form-item label="编码">
          <el-input v-model="form.name" :disabled="Boolean(editingId)" />
        </el-form-item>
        <el-form-item label="显示名">
          <el-input v-model="form.display_name" />
        </el-form-item>
        <el-form-item label="排序">
          <el-input-number v-model="form.sort_order" :min="0" />
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
import { onMounted, ref, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  createConfigItem,
  deleteConfigItem,
  fetchConfigItems,
  fetchConfigTypes,
  updateConfigItem,
  type ConfigItem,
  type ConfigTypeOption
} from '@/api/config'

const items = ref<{ list: ConfigItem[]; total: number }>({ list: [], total: 0 })
const loading = ref(false)
const typeOptions = ref<ConfigTypeOption[]>([])
const filterType = ref('')
const currentPage = ref(1)
const pageSize = ref(50)
const dialogVisible = ref(false)
const editingId = ref<number | null>(null)
const form = ref({
  config_type: '',
  name: '',
  display_name: '',
  sort_order: 0,
  enabled: true
})

async function load() {
  loading.value = true
  try {
    const [types, data] = await Promise.all([
      fetchConfigTypes(),
      fetchConfigItems(filterType.value, currentPage.value, pageSize.value)
    ])
    typeOptions.value = types
    items.value = { list: data.list, total: data.total }
  } finally {
    loading.value = false
  }
}

async function loadItemsOnly() {
  loading.value = true
  try {
    const data = await fetchConfigItems(filterType.value, currentPage.value, pageSize.value)
    items.value = { list: data.list, total: data.total }
  } finally {
    loading.value = false
  }
}

function resetForm() {
  form.value = {
    config_type: typeOptions.value[0]?.code || '',
    name: '',
    display_name: '',
    sort_order: 0,
    enabled: true
  }
}

function openCreate() {
  editingId.value = null
  resetForm()
  dialogVisible.value = true
}

function openEdit(item: ConfigItem) {
  editingId.value = item.id
  form.value = {
    config_type: item.config_type,
    name: item.name,
    display_name: item.display_name,
    sort_order: item.sort_order,
    enabled: item.enabled
  }
  dialogVisible.value = true
}

async function submit() {
  if (!form.value.display_name.trim()) {
    ElMessage.error('显示名不能为空')
    return
  }
  try {
    if (editingId.value) {
      await updateConfigItem(editingId.value, {
        display_name: form.value.display_name,
        enabled: form.value.enabled,
        sort_order: form.value.sort_order
      })
    } else {
      if (!form.value.config_type || !form.value.name.trim()) {
        ElMessage.error('类型和编码不能为空')
        return
      }
      await createConfigItem(form.value)
    }
    dialogVisible.value = false
    await load()
    ElMessage.success('保存成功')
  } catch {
    ElMessage.error('保存失败')
  }
}

async function remove(item: ConfigItem) {
  if (item.is_builtin) {
    ElMessage.warning('内置配置项不允许删除')
    return
  }
  try {
    await ElMessageBox.confirm(`确认删除配置项「${item.display_name}」？`, '删除确认', {
      type: 'warning'
    })
    await deleteConfigItem(item.id)
    await load()
    ElMessage.success('删除成功')
  } catch {
    // ignore cancel
  }
}

watch(filterType, async () => {
  try {
    currentPage.value = 1
    const data = await fetchConfigItems(filterType.value, currentPage.value, pageSize.value)
    items.value = { list: data.list, total: data.total }
  } catch {
    items.value = { list: [], total: 0 }
  }
})

onMounted(async () => {
  try {
    await load()
  } catch {
    ElMessage.error('配置项初始化失败')
  }
})
</script>

<style scoped>
.config-page {
  padding: 24px;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  margin-bottom: 18px;
}
.toolbar h2 {
  margin: 0;
}
.toolbar p {
  margin: 8px 0 0;
  color: #64748b;
}
.toolbar-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}
.pagination-wrap {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
