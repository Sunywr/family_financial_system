<template>
  <section class="panel page">
    <div class="head">
      <h2>债务</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（分类/备注/ID）" style="width: 280px;" />
        <el-button @click="search">查询</el-button>
      </div>
    </div>

    <div v-if="dashboardContextLabel" class="context-banner">
      <el-alert :title="`当前筛选来自首页：${dashboardContextLabel}`" type="info" :closable="false" show-icon />
      <div class="context-actions">
        <el-button text type="primary" @click="goDashboard">返回首页</el-button>
      </div>
    </div>

    <el-tabs v-model="activeTab" @tab-change="handleTabChange">
      <el-tab-pane label="信用卡债务" name="credit_card">
        <div v-if="isMobileView" v-loading="loading" class="mobile-cards">
          <article v-for="row in debts.list" :key="row.id" class="mobile-card panel">
            <div class="mobile-card-head">
              <strong>#{{ row.id }} · {{ row.category_name }}</strong>
              <span>{{ row.start_date }}</span>
            </div>
            <div class="mobile-card-row">
              <span>金额</span>
              <strong>{{ row.amount }}</strong>
            </div>
            <div class="mobile-card-row">
              <span>结束日期</span>
              <span>{{ row.end_date || '--' }}</span>
            </div>
            <div class="mobile-card-row">
              <span>还款截止</span>
              <span>{{ row.repay_deadline || '--' }}</span>
            </div>
            <div class="mobile-card-row">
              <span>期数</span>
              <span>{{ row.paid_period_count }}/{{ row.period_count }}</span>
            </div>
            <div class="mobile-card-row">
              <span>单位</span>
              <el-tag>{{ periodMap[row.period_unit] || row.period_unit }}</el-tag>
            </div>
            <div class="mobile-card-row">
              <span>状态</span>
              <el-tag :type="row.status === 'pending' ? 'warning' : 'success'">
                {{ statusMap[row.status] || row.status }}
              </el-tag>
            </div>
            <p v-if="row.remark" class="mobile-remark">{{ row.remark }}</p>
            <div class="mobile-card-actions">
              <el-button size="small" type="primary" plain @click="openEditDialog(row)">编辑</el-button>
            </div>
          </article>
          <div v-if="!debts.list.length" class="mobile-empty">暂无{{ activeTabLabel }}</div>
        </div>

        <el-table v-else v-loading="loading" :data="debts.list" stripe>
          <el-table-column prop="id" label="ID" width="90" />
          <el-table-column prop="start_date" label="开始日期" width="120" />
          <el-table-column prop="end_date" label="结束日期" width="120">
            <template #default="{ row }">
              {{ row.end_date || '--' }}
            </template>
          </el-table-column>
          <el-table-column prop="repay_deadline" label="还款截止日期" width="140">
            <template #default="{ row }">
              {{ row.repay_deadline || '--' }}
            </template>
          </el-table-column>
          <el-table-column prop="category_name" label="分类" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column prop="period_count" label="总期数" width="100" />
          <el-table-column prop="paid_period_count" label="已还期数" width="100" />
          <el-table-column label="单位" width="120">
            <template #default="{ row }">
              <el-tag>{{ periodMap[row.period_unit] || row.period_unit }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="120">
            <template #default="{ row }">
              <el-tag :type="row.status === 'pending' ? 'warning' : 'success'">
                {{ statusMap[row.status] || row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="remark" label="备注" min-width="220" />
          <el-table-column label="操作" width="100" fixed="right">
            <template #default="{ row }">
              <el-button text type="primary" @click="openEditDialog(row)">编辑</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="非信用卡债务" name="non_credit_card">
        <div v-if="isMobileView" v-loading="loading" class="mobile-cards">
          <article v-for="row in debts.list" :key="row.id" class="mobile-card panel">
            <div class="mobile-card-head">
              <strong>#{{ row.id }} · {{ row.category_name }}</strong>
              <span>{{ row.start_date }}</span>
            </div>
            <div class="mobile-card-row">
              <span>金额</span>
              <strong>{{ row.amount }}</strong>
            </div>
            <div class="mobile-card-row">
              <span>结束日期</span>
              <span>{{ row.end_date || '--' }}</span>
            </div>
            <div class="mobile-card-row">
              <span>还款截止</span>
              <span>{{ row.repay_deadline || '--' }}</span>
            </div>
            <div class="mobile-card-row">
              <span>期数</span>
              <span>{{ row.paid_period_count }}/{{ row.period_count }}</span>
            </div>
            <div class="mobile-card-row">
              <span>单位</span>
              <el-tag>{{ periodMap[row.period_unit] || row.period_unit }}</el-tag>
            </div>
            <div class="mobile-card-row">
              <span>状态</span>
              <el-tag :type="row.status === 'pending' ? 'warning' : 'success'">
                {{ statusMap[row.status] || row.status }}
              </el-tag>
            </div>
            <p v-if="row.remark" class="mobile-remark">{{ row.remark }}</p>
            <div class="mobile-card-actions">
              <el-button size="small" type="primary" plain @click="openEditDialog(row)">编辑</el-button>
            </div>
          </article>
          <div v-if="!debts.list.length" class="mobile-empty">暂无{{ activeTabLabel }}</div>
        </div>

        <el-table v-else v-loading="loading" :data="debts.list" stripe>
          <el-table-column prop="id" label="ID" width="90" />
          <el-table-column prop="start_date" label="开始日期" width="120" />
          <el-table-column prop="end_date" label="结束日期" width="120">
            <template #default="{ row }">
              {{ row.end_date || '--' }}
            </template>
          </el-table-column>
          <el-table-column prop="repay_deadline" label="还款截止日期" width="140">
            <template #default="{ row }">
              {{ row.repay_deadline || '--' }}
            </template>
          </el-table-column>
          <el-table-column prop="category_name" label="分类" width="120" />
          <el-table-column prop="amount" label="金额" width="120" />
          <el-table-column prop="period_count" label="总期数" width="100" />
          <el-table-column prop="paid_period_count" label="已还期数" width="100" />
          <el-table-column label="单位" width="120">
            <template #default="{ row }">
              <el-tag>{{ periodMap[row.period_unit] || row.period_unit }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="120">
            <template #default="{ row }">
              <el-tag :type="row.status === 'pending' ? 'warning' : 'success'">
                {{ statusMap[row.status] || row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="remark" label="备注" min-width="220" />
          <el-table-column label="操作" width="100" fixed="right">
            <template #default="{ row }">
              <el-button text type="primary" @click="openEditDialog(row)">编辑</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="debts.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadData"
        @size-change="loadData"
      />
    </div>

    <el-dialog v-model="editDialogVisible" title="编辑债务" width="640px" destroy-on-close>
      <el-form label-width="110px">
        <el-form-item label="开始日期">
          <el-date-picker v-model="editForm.start_date" type="date" value-format="YYYY-MM-DD" />
        </el-form-item>
        <el-form-item label="结束日期">
          <el-date-picker v-model="editForm.end_date" type="date" value-format="YYYY-MM-DD" clearable />
        </el-form-item>
        <el-form-item label="还款截止日期">
          <el-date-picker
            v-model="editForm.repay_deadline"
            type="date"
            value-format="YYYY-MM-DD"
            clearable
          />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="editForm.category_id" style="width: 100%" filterable>
            <el-option v-for="item in debtCategories" :key="item.id" :label="item.display_name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="金额">
          <el-input v-model="editForm.amount" />
        </el-form-item>
        <el-form-item label="支付方式">
          <el-select v-model="editForm.payment_method" style="width: 100%">
            <el-option label="信用卡" value="credit_card" />
            <el-option label="非信用卡" value="cash" />
          </el-select>
        </el-form-item>
        <el-form-item label="总期数">
          <el-input-number v-model="editForm.period_count" :min="1" :max="999" />
          <span class="hint">非信用卡保存时会按起止时间月份数自动重算</span>
        </el-form-item>
        <el-form-item label="期数单位">
          <el-select v-model="editForm.period_unit" style="width: 100%">
            <el-option label="天" value="day" />
            <el-option label="月" value="month" />
            <el-option label="年" value="year" />
          </el-select>
        </el-form-item>
        <el-form-item label="单位跨度">
          <el-input-number v-model="editForm.period_value" :min="1" :max="999" />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="editForm.status" style="width: 100%">
            <el-option label="待还" value="pending" />
            <el-option label="已还" value="settled" />
            <el-option label="已取消" value="cancelled" />
            <el-option label="待还(兼容)" value="debt_pending" />
          </el-select>
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="editForm.remark" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveDebt">保存</el-button>
      </template>
    </el-dialog>

  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { fetchDebts, type Debt, updateDebt } from '@/api/debts'
import { fetchConfigItems, type ConfigItem } from '@/api/config'

const route = useRoute()
const router = useRouter()
const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const loading = ref(false)
const activeTab = ref('credit_card')
const isMobileView = ref(typeof window !== 'undefined' ? window.innerWidth <= 900 : false)
const debts = ref<{ list: Debt[]; total: number }>({ list: [], total: 0 })
const debtCategories = ref<ConfigItem[]>([])
const editDialogVisible = ref(false)
const editingDebtId = ref<number | null>(null)
const saving = ref(false)
const editForm = reactive({
  start_date: '',
  end_date: '',
  repay_deadline: '',
  category_id: 0,
  amount: '',
  period_count: 1,
  period_unit: 'month',
  period_value: 1,
  payment_method: 'cash',
  status: 'pending',
  remark: ''
})
const periodMap: Record<string, string> = { day: '天', week: '周', month: '月', year: '年' }
const statusMap: Record<string, string> = { pending: '待还', settled: '已还', cancelled: '已取消' }
const dashboardContextMap: Record<string, string> = {
  'cycle-debt-summary': '周期债务摘要',
  'cycle-debt-row': '周期债务明细'
}
const dashboardContextLabel =
  route.query.from === 'dashboard' && typeof route.query.context === 'string'
    ? dashboardContextMap[route.query.context] || '首页钻取'
    : ''
const activeTabLabel = computed(() =>
  activeTab.value === 'credit_card' ? '信用卡债务' : '非信用卡债务'
)

function activePaymentMethodFilter(): 'credit_card' | 'non_credit_card' {
  return activeTab.value === 'credit_card' ? 'credit_card' : 'non_credit_card'
}

async function loadData() {
  loading.value = true
  try {
    debts.value = await fetchDebts(
      keyword.value,
      currentPage.value,
      pageSize.value,
      activePaymentMethodFilter()
    )
  } finally {
    loading.value = false
  }
}

function toMonthSpan(startDate: string, endDate?: string | null) {
  if (!startDate || !endDate) return 1
  const start = new Date(startDate)
  const end = new Date(endDate)
  if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) return 1
  const months =
    (end.getFullYear() - start.getFullYear()) * 12 +
    (end.getMonth() - start.getMonth()) +
    1
  return Math.max(1, months)
}

function isNonCreditPaymentMethod(method: string) {
  return method !== 'credit_card'
}

function openEditDialog(row: Debt) {
  editingDebtId.value = row.id
  editForm.start_date = row.start_date
  editForm.end_date = row.end_date || ''
  editForm.repay_deadline = row.repay_deadline || ''
  editForm.category_id = row.category_id
  editForm.amount = row.amount
  editForm.period_count = row.period_count
  editForm.period_unit = row.period_unit
  editForm.period_value = row.period_value
  editForm.payment_method = row.payment_method
  editForm.status = row.status
  editForm.remark = row.remark || ''
  editDialogVisible.value = true
}

async function saveDebt() {
  if (!editingDebtId.value) return
  if (!editForm.start_date) {
    ElMessage.warning('开始日期不能为空')
    return
  }
  if (!editForm.category_id) {
    ElMessage.warning('请选择分类')
    return
  }

  const periodCount = isNonCreditPaymentMethod(editForm.payment_method)
    ? toMonthSpan(editForm.start_date, editForm.end_date || editForm.repay_deadline || undefined)
    : editForm.period_count

  saving.value = true
  try {
    await updateDebt(editingDebtId.value, {
      start_date: editForm.start_date,
      end_date: editForm.end_date || null,
      repay_deadline: editForm.repay_deadline || null,
      category_id: editForm.category_id,
      amount: editForm.amount,
      period_count: periodCount,
      period_unit: editForm.period_unit,
      period_value: editForm.period_value,
      payment_method: editForm.payment_method,
      status: editForm.status,
      remark: editForm.remark || null
    })
    ElMessage.success('债务已更新')
    editDialogVisible.value = false
    await loadData()
  } catch {
    ElMessage.error('更新失败')
  } finally {
    saving.value = false
  }
}

function search() {
  currentPage.value = 1
  void loadData()
}

function handleTabChange() {
  currentPage.value = 1
  void loadData()
}

function goDashboard() {
  void router.push({ name: 'dashboard' })
}

function syncViewportMode() {
  isMobileView.value = window.innerWidth <= 900
}

onMounted(async () => {
  syncViewportMode()
  window.addEventListener('resize', syncViewportMode)
  try {
    const categoryData = await fetchConfigItems('debt_category', 1, 200)
    debtCategories.value = categoryData.list
    if (route.query.tab === 'non_credit_card') {
      activeTab.value = 'non_credit_card'
    }
    if (typeof route.query.keyword === 'string') {
      keyword.value = route.query.keyword
    }
    await loadData()
  } catch {
    ElMessage.error('债务页面初始化失败')
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', syncViewportMode)
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
  top: 26px;
  right: 3%;
  width: 180px;
  height: 180px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 188, 96, 0.12) 0%, rgba(255, 188, 96, 0) 72%);
  filter: blur(8px);
  pointer-events: none;
  animation: debtsHalo 15s ease-in-out infinite alternate;
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

.pagination-wrap {
  margin-top: 18px;
  display: flex;
  justify-content: flex-end;
  padding: 12px 14px 0;
}

.context-banner {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px;
  border-radius: 18px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.82), rgba(246, 249, 252, 0.72)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.06), rgba(255, 190, 102, 0.07));
  border: 1px solid rgba(148, 163, 184, 0.16);
}

.context-actions {
  display: flex;
  justify-content: flex-end;
}

.page :deep(.el-tabs__nav-wrap::after) {
  height: 1px;
  background: rgba(148, 163, 184, 0.16);
}

.page :deep(.el-tabs__item) {
  transition: color 0.2s ease, transform 0.2s ease;
}

.page :deep(.el-tabs__item.is-active) {
  transform: translateY(-1px);
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

.hint {
  margin-left: 10px;
  color: #909399;
  font-size: 12px;
}

.mobile-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mobile-card {
  padding: 12px;
  border-radius: 14px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.88), rgba(247, 250, 252, 0.74)),
    linear-gradient(135deg, rgba(76, 132, 255, 0.06), rgba(255, 190, 102, 0.07));
}

.mobile-card-head {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  color: #475569;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.14);
}

.mobile-card-row {
  margin-top: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.mobile-remark {
  margin: 10px 0 0;
  font-size: 13px;
  color: #475569;
}

.mobile-card-actions {
  margin-top: 10px;
  display: flex;
  gap: 8px;
}

.mobile-empty {
  padding: 18px 10px;
  text-align: center;
  color: #94a3b8;
}

@keyframes debtsHalo {
  from {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.46;
  }
  to {
    transform: translate3d(-14px, 10px, 0) scale(1.1);
    opacity: 0.88;
  }
}

</style>
