<template>
  <section class="panel page">
    <div class="head">
      <div>
        <h2 style="margin: 0;">信用卡配置</h2>
        <p style="margin: 8px 0 0; color: #64748b;">支持信用卡分页、创建、修改和删除。</p>
      </div>
      <div class="actions">
        <el-tag>{{ cards.total }} cards</el-tag>
        <el-button type="primary" @click="openCreate">新增信用卡</el-button>
      </div>
    </div>

    <el-table v-loading="loading" :data="cards.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="user_id" label="用户ID" width="100" />
      <el-table-column prop="name" label="名称" min-width="180" />
      <el-table-column prop="billing_day" label="账单日" width="100" />
      <el-table-column prop="repayment_day" label="还款日" width="100" />
      <el-table-column prop="credit_limit" label="额度" min-width="120" />
      <el-table-column label="启用" width="100">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'">{{ row.enabled ? '启用' : '停用' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="180">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" type="danger" @click="removeCard(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="cards.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadCards"
        @size-change="loadCards"
      />
    </div>

    <el-dialog v-model="dialogVisible" :title="editingId ? '编辑信用卡' : '新增信用卡'" width="520px" @closed="resetForm">
      <el-form label-width="100px">
        <el-form-item v-if="isAdmin" label="用户">
          <el-select v-model="form.user_id" style="width: 100%;">
            <el-option v-for="user in users" :key="user.id" :label="user.display_name" :value="user.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="名称">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="账单日">
          <el-input-number v-model="form.billing_day" :min="1" :max="31" />
        </el-form-item>
        <el-form-item label="还款日">
          <el-input-number v-model="form.repayment_day" :min="1" :max="31" />
        </el-form-item>
        <el-form-item label="额度">
          <el-input v-model="form.credit_limit" placeholder="10000.00" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="form.enabled" />
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
import { computed, onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  createCreditCard,
  deleteCreditCard,
  fetchCreditCards,
  updateCreditCard,
  type CreditCard
} from '@/api/config'
import { getStoredUser, requireCurrentUserId } from '@/api/client'
import { fetchUsers, type User } from '@/api/users'

const currentPage = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const cards = ref<{ list: CreditCard[]; total: number }>({
  list: [],
  total: 0
})
const dialogVisible = ref(false)
const editingId = ref<number | null>(null)
const users = ref<User[]>([])
const currentUserId = requireCurrentUserId()
const currentUser = getStoredUser()
const isAdmin = computed(() => currentUser?.username === 'admin')
const form = ref({
  user_id: currentUserId,
  name: '',
  billing_day: 1,
  repayment_day: 1,
  credit_limit: '',
  enabled: true
})

function resetForm() {
  form.value = {
    user_id: currentUserId,
    name: '',
    billing_day: 1,
    repayment_day: 1,
    credit_limit: '',
    enabled: true
  }
}

async function loadCards() {
  loading.value = true
  try {
    const data = await fetchCreditCards(currentPage.value, pageSize.value)
    cards.value = { list: data.list, total: data.total }
  } finally {
    loading.value = false
  }
}

async function loadUsers() {
  if (!isAdmin.value) return
  const data = await fetchUsers(1, 100)
  users.value = data.list
}

function openCreate() {
  editingId.value = null
  resetForm()
  if (isAdmin.value && users.value.length > 0) {
    form.value.user_id = users.value[0].id
  }
  dialogVisible.value = true
}

function openEdit(row: CreditCard) {
  editingId.value = row.id
  form.value = {
    user_id: row.user_id,
    name: row.name,
    billing_day: row.billing_day,
    repayment_day: row.repayment_day,
    credit_limit: row.credit_limit,
    enabled: row.enabled
  }
  dialogVisible.value = true
}

async function submitForm() {
  const name = form.value.name.trim()
  const credit_limit = form.value.credit_limit.trim()
  if (!name || !credit_limit) {
    ElMessage.warning('名称和额度不能为空')
    return
  }

  try {
    if (editingId.value) {
      await updateCreditCard(editingId.value, {
        name,
        billing_day: form.value.billing_day,
        repayment_day: form.value.repayment_day,
        credit_limit,
        enabled: form.value.enabled
      })
    } else {
      await createCreditCard({
        user_id: isAdmin.value ? form.value.user_id : currentUserId,
        name,
        billing_day: form.value.billing_day,
        repayment_day: form.value.repayment_day,
        credit_limit,
        enabled: form.value.enabled
      })
    }
    dialogVisible.value = false
    await loadCards()
    ElMessage.success('信用卡已保存')
  } catch (error: any) {
    ElMessage.error(error?.response?.data?.message || '信用卡保存失败')
  }
}

async function removeCard(row: CreditCard) {
  try {
    await ElMessageBox.confirm(
      `确认删除信用卡“${row.name}”吗？如果仍有关联账单，系统会拒绝删除。`,
      '删除确认',
      { type: 'warning' }
    )
  } catch {
    return
  }

  try {
    await deleteCreditCard(row.id)
    await loadCards()
    ElMessage.success('信用卡已删除')
  } catch (error: any) {
    ElMessage.error(error?.response?.data?.message || '删除失败')
  }
}

onMounted(async () => {
  try {
    await Promise.all([loadCards(), loadUsers()])
  } catch {
    ElMessage.error('信用卡页面初始化失败')
    cards.value = { list: [], total: 0 }
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
  animation: cardsHalo 16s ease-in-out infinite alternate;
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  margin-bottom: 18px;
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

.page :deep(.el-dialog__body) {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.94), rgba(247, 250, 253, 0.84)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.04), rgba(255, 190, 102, 0.05));
}

.pagination-wrap {
  margin-top: 18px;
  display: flex;
  justify-content: flex-end;
  padding: 12px 14px 0;
}

@keyframes cardsHalo {
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
