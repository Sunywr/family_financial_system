<template>
  <section class="panel page">
    <div class="head">
      <h2>账单</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（分类/备注）" style="width: 240px;" @keyup.enter="search" />
        <el-date-picker
          v-model="dateRange"
          type="daterange"
          start-placeholder="开始日期"
          end-placeholder="结束日期"
          value-format="YYYY-MM-DD"
          style="width: 280px;"
        />
        <el-select v-model="filterCategoryId" clearable placeholder="按分类" style="width: 180px;">
          <el-option v-for="cat in categories" :key="cat.id" :label="cat.display_name" :value="cat.id" />
        </el-select>
        <el-select v-model="filterMethodKey" clearable placeholder="按方式" style="width: 200px;">
          <el-option label="现金" value="cash" />
          <el-option v-for="card in creditCards" :key="card.id" :label="`信用卡 · ${card.name}`" :value="`credit_card:${card.id}`" />
        </el-select>
        <el-button @click="search">查询</el-button>
        <el-button @click="resetFilters">重置筛选</el-button>
        <el-button @click="openTagManager">标签管理</el-button>
        <el-button type="primary" @click="openCreate">新增账单</el-button>
      </div>
    </div>

    <el-table :data="bills.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="account_date" label="日期" width="110" />
      <el-table-column prop="category_name" label="分类" width="110" />
      <el-table-column label="类型" width="100">
        <template #default="{ row }">
          <el-tag :type="billTypeTagType(row.bill_type)">{{ billTypeMap[row.bill_type] || row.bill_type }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="方式" width="180">
        <template #default="{ row }">
          <el-tag type="info">{{ formatPaymentMethod(row) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="amount" label="金额" width="110" />
      <el-table-column label="关联" width="220">
        <template #default="{ row }">
          <el-tag v-if="formatRelation(row)" size="small" type="warning">
            {{ formatRelation(row) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="标签" min-width="160">
        <template #default="{ row }">
          <el-tag v-for="tag in row.tags || []" :key="tag" size="small" class="mr-4">{{ tag }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="remark" label="备注" min-width="180" show-overflow-tooltip />
    </el-table>

    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="bills.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadBills"
        @size-change="loadBills"
      />
    </div>

    <!-- 新增账单弹窗 -->
    <el-dialog v-model="dialogVisible" title="新增账单" width="760px" @closed="resetForm">
      <el-form label-position="top">
        <el-form-item v-if="isAdmin" label="用户">
          <el-select v-model="form.user_id" style="width: 100%;">
            <el-option v-for="user in users" :key="user.id" :label="user.display_name" :value="user.id" />
          </el-select>
        </el-form-item>

        <div class="grid-2">
          <el-form-item label="日期">
            <el-date-picker v-model="form.account_date" type="date" value-format="YYYY-MM-DD" style="width: 100%;" />
          </el-form-item>
          <el-form-item label="金额">
            <el-input v-model="form.amount" placeholder="0.00" />
          </el-form-item>
        </div>

        <el-form-item label="账单大类">
          <div class="btn-group">
            <el-button
              v-for="group in billGroups"
              :key="group.key"
              :type="form.billGroup === group.key ? 'primary' : 'default'"
              size="small"
              @click="selectBillGroup(group.key)"
            >{{ group.label }}</el-button>
          </div>
        </el-form-item>

        <el-form-item v-if="form.billGroup" label="账单类型">
          <div class="btn-group">
            <el-button
              v-for="bt in currentBillTypes"
              :key="bt"
              :type="form.bill_type === bt ? 'primary' : 'default'"
              size="small"
              @click="form.bill_type = bt"
            >{{ billTypeMap[bt] || bt }}</el-button>
          </div>
        </el-form-item>

        <el-form-item v-if="form.billGroup" label="分类">
          <div class="btn-group">
            <el-button
              v-for="cat in visibleCategories"
              :key="cat.id"
              :type="form.category_id === cat.id ? 'primary' : 'default'"
              size="small"
              @click="selectCategory(cat.id)"
            >{{ cat.display_name }}</el-button>
          </div>
        </el-form-item>

        <el-form-item label="支付方式">
          <div class="btn-group">
            <el-button
              v-for="pm in availablePaymentMethods"
              :key="pm"
              :type="form.payment_method === pm ? 'primary' : 'default'"
              size="small"
              @click="form.payment_method = pm"
            >{{ paymentMap[pm] || pm }}</el-button>
          </div>
        </el-form-item>

        <el-form-item label="标签（点击快速添加）">
          <div class="btn-group mb-8">
            <el-button
              v-for="tag in topTags"
              :key="tag.id"
              :type="form.tags.includes(tag.name) ? 'success' : 'default'"
              size="small"
              @click="toggleTag(tag.name)"
            >{{ tag.name }}</el-button>
          </div>
          <el-select
            v-model="form.tags"
            multiple
            filterable
            allow-create
            default-first-option
            placeholder="或输入并回车创建新标签"
            style="width: 100%;"
          >
            <el-option v-for="tag in allTags" :key="tag.id" :label="tag.name" :value="tag.name" />
          </el-select>
        </el-form-item>

        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :disabled="!form.billGroup || !form.category_id || !form.bill_type" @click="submitBill">保存</el-button>
      </template>
    </el-dialog>

    <!-- 标签管理弹窗 -->
    <el-dialog v-model="tagDialogVisible" title="标签管理" width="620px">
      <div class="actions mb-12">
        <el-input v-model="tagKeyword" placeholder="检索标签" style="width: 220px;" @keyup.enter="loadAllTags" />
        <el-button @click="loadAllTags">查询</el-button>
      </div>
      <el-table :data="allTags" stripe>
        <el-table-column prop="id" label="ID" width="70" />
        <el-table-column label="标签名">
          <template #default="{ row }">
            <el-input v-model="row.name" size="small" />
          </template>
        </el-table-column>
        <el-table-column label="颜色" width="130">
          <template #default="{ row }">
            <el-input v-model="row.color" size="small" />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="160">
          <template #default="{ row }">
            <el-button size="small" @click="saveTag(row)">保存</el-button>
            <el-button size="small" type="danger" @click="removeTag(row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="actions mt-12">
        <el-input v-model="newTagName" placeholder="新标签名" style="width: 220px;" />
        <el-button type="primary" @click="addTag">新增标签</el-button>
      </div>
    </el-dialog>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { createBill, fetchBillOptions, fetchBills, type Bill, type BillOptions } from '@/api/bills'
import { fetchConfigItems, fetchCreditCards, type ConfigItem, type CreditCard } from '@/api/config'
import { fetchUsers, type User } from '@/api/users'
import {
  createBillTag,
  deleteBillTag,
  fetchBillTags,
  fetchTopBillTags,
  updateBillTag,
  type BillTag
} from '@/api/bill-tags'
import { getStoredUser } from '@/api/client'

const billTypeMap: Record<string, string> = {
  income: '收入',
  expense: '支出',
  refund: '退款',
  open_position: '建仓',
  add_position: '加仓',
  reduce_position: '减仓',
  dividend: '分红'
}
const paymentMap: Record<string, string> = {
  cash: '现金',
  credit_card: '信用卡',
  installment: '分期',
  presale: '预售',
  stock_account: '股票账户'
}
const relationLabelMap: Record<string, string> = {
  transfer: '转账',
  investment: '投资',
  debt: '债务',
  asset: '资产',
  budget: '预算',
  presale: '预售',
  system_user: '用户'
}
const billGroups: { key: 'normal' | 'investment'; label: string }[] = [
  { key: 'normal', label: '正常账单' },
  { key: 'investment', label: '股票/理财账单' }
]

function billTypeTagType(bt: string): '' | 'success' | 'info' | 'warning' | 'danger' {
  if (['income', 'refund', 'dividend', 'reduce_position'].includes(bt)) return 'success'
  if (['expense', 'open_position', 'add_position'].includes(bt)) return 'danger'
  return 'info'
}

const creditCardNameMap = ref<Record<number, string>>({})

function creditCardDisplay(id?: number | null) {
  if (!id) return ''
  return creditCardNameMap.value[id] || `信用卡#${id}`
}

function formatPaymentMethod(row: Bill) {
  if (row.payment_method === 'credit_card') {
    return creditCardDisplay(row.credit_card_id) || paymentMap[row.payment_method] || row.payment_method
  }
  return paymentMap[row.payment_method] || row.payment_method
}

function formatRelation(row: Bill) {
  if (row.special_status === 'imported') return ''

  const transferType = row.transfer_target_type || ''
  if (transferType) {
    const label = relationLabelMap[transferType] || transferType
    if (transferType === 'system_user' && row.transfer_target_user_id) return `${label}#${row.transfer_target_user_id}`
    if (transferType === 'investment' && row.related_investment_id) return `${label}#${row.related_investment_id}`
    if (transferType === 'asset' && row.related_asset_id) return `${label}#${row.related_asset_id}`
    return label
  }

  if (row.payment_method === 'presale') return '预售'
  if (row.related_investment_id) return `投资#${row.related_investment_id}`
  if (row.related_asset_id) return `资产#${row.related_asset_id}`

  const status = row.special_status || ''
  if (status === 'debt_pending' && row.credit_card_id) return `债务·${creditCardDisplay(row.credit_card_id)}`
  if (!status || status === 'none') return ''
  const key = status.replace('_pending', '')
  return relationLabelMap[key] || key
}

const currentPage = ref(1)
const pageSize = ref(20)
const keyword = ref('')
const dateRange = ref<[string, string] | []>([])
const filterCategoryId = ref<number | undefined>(undefined)
const filterMethodKey = ref<string | undefined>(undefined)
const dialogVisible = ref(false)
const tagDialogVisible = ref(false)
const tagKeyword = ref('')
const newTagName = ref('')
const bills = ref<{ list: Bill[]; total: number }>({ list: [], total: 0 })
const users = ref<User[]>([])
const creditCards = ref<CreditCard[]>([])
const categories = ref<ConfigItem[]>([])
const allTags = ref<BillTag[]>([])
const topTags = ref<BillTag[]>([])
const options = reactive<BillOptions>({
  payment_methods: [],
  normal_bill_types: [],
  investment_bill_types: [],
  transfer_target_types: []
})

const currentUser = getStoredUser()
const isAdmin = computed(() => currentUser?.username === 'admin')
const currentUserId = computed(() => currentUser?.id ?? 0)

const form = reactive({
  user_id: 0,
  account_date: '',
  category_id: 0,
  billGroup: '' as '' | 'normal' | 'investment',
  bill_type: '',
  payment_method: 'cash',
  is_fixed_asset: false,
  amount: '',
  remark: '',
  tags: [] as string[]
})

const currentBillTypes = computed(() =>
  form.billGroup === 'investment' ? options.investment_bill_types : options.normal_bill_types
)
const visibleCategories = computed(() => {
  if (form.billGroup === 'investment') {
    return categories.value.filter((c) => ['stock', 'wealth'].includes(c.name))
  }
  return categories.value.filter((c) => !['stock', 'wealth'].includes(c.name))
})
const availablePaymentMethods = computed(() => {
  if (form.billGroup === 'investment') return ['cash', 'stock_account']
  return options.payment_methods.filter((pm) => pm !== 'stock_account')
})

function selectBillGroup(group: 'normal' | 'investment') {
  form.billGroup = group
  form.bill_type = ''
  form.category_id = 0
  form.payment_method = group === 'investment' ? 'stock_account' : 'cash'
}

function selectCategory(id: number) {
  form.category_id = id
}

function toggleTag(name: string) {
  const idx = form.tags.indexOf(name)
  if (idx >= 0) form.tags.splice(idx, 1)
  else form.tags.push(name)
}

function resetForm() {
  form.user_id = isAdmin.value ? (users.value[0]?.id ?? 0) : currentUserId.value
  form.account_date = new Date().toISOString().slice(0, 10)
  form.category_id = 0
  form.billGroup = ''
  form.bill_type = ''
  form.payment_method = 'cash'
  form.is_fixed_asset = false
  form.amount = ''
  form.remark = ''
  form.tags = []
}

async function loadBills() {
  try {
    const start_date = dateRange.value.length === 2 ? dateRange.value[0] : undefined
    const end_date = dateRange.value.length === 2 ? dateRange.value[1] : undefined
    const isCreditCardFilter = (filterMethodKey.value || '').startsWith('credit_card:')
    const credit_card_id = isCreditCardFilter
      ? Number((filterMethodKey.value || '').split(':')[1])
      : undefined
    const payment_method = isCreditCardFilter ? 'credit_card' : filterMethodKey.value

    const data = await fetchBills(keyword.value, currentPage.value, pageSize.value, {
      category_id: filterCategoryId.value,
      payment_method,
      credit_card_id,
      start_date,
      end_date
    })
    bills.value = data
  } catch {
    ElMessage.error('加载账单失败')
  }
}

function search() {
  currentPage.value = 1
  loadBills()
}

function resetFilters() {
  keyword.value = ''
  dateRange.value = []
  filterCategoryId.value = undefined
  filterMethodKey.value = undefined
  search()
}

async function loadAllTags() {
  try {
    const data = await fetchBillTags(tagKeyword.value)
    allTags.value = data.list
  } catch {
    allTags.value = []
  }
}

async function loadTopTags() {
  try {
    topTags.value = await fetchTopBillTags(currentUserId.value)
  } catch {
    topTags.value = []
  }
}

async function loadData() {
  await loadBills()
  const [optionResult, userResult, categoryResult, creditCardResult] = await Promise.allSettled([
    fetchBillOptions(),
    fetchUsers(),
    fetchConfigItems('account_category'),
    fetchCreditCards()
  ])
  if (optionResult.status === 'fulfilled') Object.assign(options, optionResult.value)
  if (userResult.status === 'fulfilled') users.value = userResult.value.list
  if (categoryResult.status === 'fulfilled') categories.value = categoryResult.value.list
  if (creditCardResult.status === 'fulfilled') {
    creditCards.value = creditCardResult.value.list
    creditCardNameMap.value = Object.fromEntries(creditCards.value.map((item) => [item.id, item.name]))
  }
  await Promise.allSettled([loadAllTags(), loadTopTags()])
  resetForm()
}

function openCreate() {
  resetForm()
  dialogVisible.value = true
}

async function submitBill() {
  if (!form.bill_type) { ElMessage.warning('请选择账单类型'); return }
  if (!form.category_id) { ElMessage.warning('请选择分类'); return }
  try {
    await createBill({
      user_id: isAdmin.value ? form.user_id : currentUserId.value,
      account_date: form.account_date,
      category_id: form.category_id,
      bill_type: form.bill_type,
      payment_method: form.payment_method,
      is_fixed_asset: form.is_fixed_asset,
      amount: form.amount,
      tags: form.tags.length ? form.tags : undefined,
      remark: form.remark || undefined
    })
    ElMessage.success('账单已创建')
    dialogVisible.value = false
    await loadBills()
  } catch {
    ElMessage.error('创建失败，请检查输入')
  }
}

function openTagManager() {
  tagDialogVisible.value = true
}

async function addTag() {
  const name = newTagName.value.trim()
  if (!name) return
  try {
    await createBillTag({ user_id: currentUserId.value, name })
    newTagName.value = ''
    await Promise.allSettled([loadAllTags(), loadTopTags()])
  } catch {
    ElMessage.error('新增标签失败')
  }
}

async function saveTag(tag: BillTag) {
  try {
    await updateBillTag(tag.id, { name: tag.name, color: tag.color || undefined })
    ElMessage.success('标签已保存')
  } catch {
    ElMessage.error('保存失败')
  }
}

async function removeTag(id: number) {
  try {
    await deleteBillTag(id)
    await loadAllTags()
  } catch {
    ElMessage.error('删除失败')
  }
}

onMounted(async () => {
  try {
    await loadData()
  } catch {
    ElMessage.error('账单页面初始化失败')
  }
})
</script>

<style scoped>
.page { padding: 24px; }
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}
.head h2 { margin: 0; }
.actions { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.grid-2 { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; }
.btn-group { display: flex; flex-wrap: wrap; gap: 8px; }
.mb-8 { margin-bottom: 8px; }
.mb-12 { margin-bottom: 12px; }
.mt-12 { margin-top: 12px; }
.mr-4 { margin-right: 4px; }
.pagination-wrap { margin-top: 16px; display: flex; justify-content: flex-end; }
</style>
