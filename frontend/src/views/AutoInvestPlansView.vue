<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; gap: 16px;">
      <div>
        <h2 style="margin: 0;">定投计划</h2>
        <p style="margin: 8px 0 0; color: #64748b;">管理定期投资计划，调度任务每日自动按周期生成投资账单。</p>
      </div>
      <el-button type="primary" @click="openCreateDialog">新建计划</el-button>
    </div>

    <div style="display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap;">
      <el-select v-model="filterStatus" placeholder="状态筛选" clearable style="width: 140px;" @change="loadData">
        <el-option label="进行中" value="active" />
        <el-option label="已暂停" value="paused" />
        <el-option label="已结束" value="stopped" />
      </el-select>
    </div>

    <el-table v-loading="loading" :data="plans.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="name" label="计划名称" min-width="160" />
      <el-table-column prop="investment_name" label="投资标的" min-width="160" />
      <el-table-column prop="category_name" label="账单分类" min-width="120" />
      <el-table-column prop="amount" label="每期金额" width="120" />
      <el-table-column label="周期" width="90">
        <template #default="{ row }">
          {{ row.cycle_months }}个月
        </template>
      </el-table-column>
      <el-table-column prop="start_date" label="开始日期" width="120" />
      <el-table-column label="结束日期" width="120">
        <template #default="{ row }">
          {{ row.end_date || '—' }}
        </template>
      </el-table-column>
      <el-table-column label="上次生成" width="120">
        <template #default="{ row }">
          {{ row.last_generated_date || '—' }}
        </template>
      </el-table-column>
      <el-table-column label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="statusTagType(row.status)">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="160" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="openEditDialog(row)">编辑</el-button>
          <el-button size="small" type="danger" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="plans.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadData"
        @size-change="loadData"
      />
    </div>

    <!-- Create/Edit Dialog -->
    <el-dialog v-model="dialogVisible" :title="editingId ? '编辑定投计划' : '新建定投计划'" width="520px">
      <el-form :model="form" label-width="100px">
        <el-form-item label="计划名称">
          <el-input v-model="form.name" placeholder="例：每月定投沪深300" />
        </el-form-item>
        <el-form-item label="投资标的ID">
          <el-input-number v-model="form.investment_id" :min="1" style="width: 100%;" />
        </el-form-item>
        <el-form-item label="账单分类ID">
          <el-input-number v-model="form.category_id" :min="1" style="width: 100%;" />
        </el-form-item>
        <el-form-item label="每期金额">
          <el-input v-model="form.amount" placeholder="例：500.00" />
        </el-form-item>
        <el-form-item label="周期(月)">
          <el-input-number v-model="form.cycle_months" :min="1" :max="120" style="width: 100%;" />
        </el-form-item>
        <el-form-item label="开始日期">
          <el-date-picker v-model="form.start_date" type="date" value-format="YYYY-MM-DD" style="width: 100%;" />
        </el-form-item>
        <el-form-item label="结束日期">
          <el-date-picker v-model="form.end_date" type="date" value-format="YYYY-MM-DD" clearable style="width: 100%;" />
        </el-form-item>
        <el-form-item v-if="editingId" label="状态">
          <el-select v-model="form.status" style="width: 100%;">
            <el-option label="进行中" value="active" />
            <el-option label="已暂停" value="paused" />
            <el-option label="已结束" value="stopped" />
          </el-select>
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { requireCurrentUserId } from '@/api/client'
import {
  listAutoInvestPlans,
  createAutoInvestPlan,
  updateAutoInvestPlan,
  deleteAutoInvestPlan,
  type AutoInvestPlan,
} from '@/api/auto_invest_plans'

const plans = ref<{ list: AutoInvestPlan[]; total: number }>({ list: [], total: 0 })
const currentPage = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const filterStatus = ref<string | null>(null)

const dialogVisible = ref(false)
const saving = ref(false)
const editingId = ref<number | null>(null)

const emptyForm = () => ({
  investment_id: 0,
  name: '',
  category_id: 0,
  amount: '',
  cycle_months: 1,
  start_date: '',
  end_date: null as string | null,
  status: 'active',
  remark: null as string | null,
})
const form = ref(emptyForm())

async function loadData() {
  loading.value = true
  try {
    const userId = requireCurrentUserId()
    const res = await listAutoInvestPlans({
      page: currentPage.value,
      page_size: pageSize.value,
      user_id: userId,
      status: filterStatus.value || undefined,
    })
    if (res.data.code === 0) {
      plans.value = res.data.data
    } else {
      ElMessage.error(res.data.message || '加载失败')
    }
  } catch (e) {
    ElMessage.error('加载失败')
  } finally {
    loading.value = false
  }
}

function openCreateDialog() {
  editingId.value = null
  form.value = emptyForm()
  dialogVisible.value = true
}

function openEditDialog(row: AutoInvestPlan) {
  editingId.value = row.id
  form.value = {
    investment_id: row.investment_id,
    name: row.name,
    category_id: row.category_id,
    amount: row.amount,
    cycle_months: row.cycle_months,
    start_date: row.start_date,
    end_date: row.end_date ?? null,
    status: row.status,
    remark: row.remark ?? null,
  }
  dialogVisible.value = true
}

async function handleSave() {
  if (!form.value.name || !form.value.amount || !form.value.start_date) {
    ElMessage.warning('请填写计划名称、金额、开始日期')
    return
  }
  saving.value = true
  try {
    const userId = requireCurrentUserId()
    if (editingId.value) {
      const res = await updateAutoInvestPlan(editingId.value, {
        name: form.value.name,
        category_id: form.value.category_id,
        amount: form.value.amount,
        cycle_months: form.value.cycle_months,
        start_date: form.value.start_date,
        end_date: form.value.end_date || null,
        status: form.value.status,
        remark: form.value.remark || null,
      })
      if (res.data.code === 0) {
        ElMessage.success('更新成功')
        dialogVisible.value = false
        loadData()
      } else {
        ElMessage.error(res.data.message || '更新失败')
      }
    } else {
      const res = await createAutoInvestPlan({
        user_id: userId,
        investment_id: form.value.investment_id,
        name: form.value.name,
        category_id: form.value.category_id,
        amount: form.value.amount,
        cycle_months: form.value.cycle_months,
        start_date: form.value.start_date,
        end_date: form.value.end_date || null,
        remark: form.value.remark || null,
      })
      if (res.data.code === 0) {
        ElMessage.success('创建成功')
        dialogVisible.value = false
        loadData()
      } else {
        ElMessage.error(res.data.message || '创建失败')
      }
    }
  } catch (e) {
    ElMessage.error('操作失败')
  } finally {
    saving.value = false
  }
}

async function handleDelete(row: AutoInvestPlan) {
  try {
    await ElMessageBox.confirm(`确认删除定投计划「${row.name}」？`, '确认删除', {
      type: 'warning',
    })
  } catch {
    return
  }
  try {
    const res = await deleteAutoInvestPlan(row.id)
    if (res.data.code === 0) {
      ElMessage.success('删除成功')
      loadData()
    } else {
      ElMessage.error(res.data.message || '删除失败')
    }
  } catch (e) {
    ElMessage.error('删除失败')
  }
}

function statusLabel(status: string) {
  const map: Record<string, string> = { active: '进行中', paused: '已暂停', stopped: '已结束' }
  return map[status] ?? status
}

function statusTagType(status: string): 'success' | 'warning' | 'info' | 'danger' {
  if (status === 'active') return 'success'
  if (status === 'paused') return 'warning'
  return 'info'
}

onMounted(loadData)
</script>

<style scoped>
.pagination-wrap {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
