<template>
  <section class="panel page">
    <div class="head">
      <h2>账单</h2>
      <div class="actions">
        <el-input v-model="keyword" placeholder="全文检索（分类/备注/ID）" style="width: 280px;" />
        <el-button @click="loadData">查询</el-button>
        <el-button @click="openTagManager">标签管理</el-button>
        <el-button type="primary" @click="dialogVisible = true">新增账单</el-button>
      </div>
    </div>

    <el-table :data="bills.list" stripe>
      <el-table-column prop="id" label="ID" width="90" />
      <el-table-column prop="account_date" label="日期" width="120" />
      <el-table-column prop="category_name" label="分类" width="120" />
      <el-table-column label="类型" width="120">
        <template #default="{ row }">
          <el-tag>{{ billTypeMap[row.bill_type] || row.bill_type }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="方式" width="120">
        <template #default="{ row }">
          <el-tag type="info">{{ paymentMap[row.payment_method] || row.payment_method }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="amount" label="金额" width="120" />
      <el-table-column label="联动状态" width="180">
        <template #default="{ row }">
          <el-tag size="small" type="warning">{{ formatSpecialStatus(row) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="标签" min-width="180">
        <template #default="{ row }">
          <el-tag v-for="tag in row.tags || []" :key="tag" class="mr-6">{{ tag }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="remark" label="备注" min-width="220" />
    </el-table>

    <el-dialog v-model="dialogVisible" title="新增账单" width="760px">
      <el-form label-position="top">
        <div class="grid-2">
          <el-form-item label="用户">
            <el-select v-model="form.user_id">
              <el-option v-for="user in users" :key="user.id" :label="user.display_name" :value="user.id" />
            </el-select>
          </el-form-item>
          <el-form-item label="日期">
            <el-date-picker v-model="form.account_date" type="date" value-format="YYYY-MM-DD" style="width: 100%;" />
          </el-form-item>
          <el-form-item label="分类">
            <el-select v-model="form.category_id">
              <el-option
                v-for="item in categories"
                :key="item.id"
                :label="item.display_name"
                :value="item.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item label="账单类型">
            <el-select v-model="form.bill_type">
              <el-option v-for="item in billTypeOptions" :key="item" :label="billTypeMap[item] || item" :value="item" />
            </el-select>
          </el-form-item>
          <el-form-item label="支付方式">
            <el-select v-model="form.payment_method">
              <el-option
                v-for="item in options.payment_methods"
                :key="item"
                :label="paymentMap[item] || item"
                :value="item"
              />
            </el-select>
          </el-form-item>
          <el-form-item label="金额">
            <el-input v-model="form.amount" />
          </el-form-item>
        </div>
        <el-form-item label="标签">
          <el-select
            v-model="form.tags"
            multiple
            filterable
            allow-create
            default-first-option
            placeholder="输入并回车可实时创建标签"
          >
            <el-option v-for="tag in billTags" :key="tag.id" :label="tag.name" :value="tag.name" />
          </el-select>
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitBill">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="tagDialogVisible" title="标签管理" width="620px">
      <div class="actions mb-12">
        <el-input v-model="tagKeyword" placeholder="检索标签" style="width: 220px;" />
        <el-button @click="loadBillTags">查询</el-button>
      </div>
      <el-table :data="billTags" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column label="标签名">
          <template #default="{ row }">
            <el-input v-model="row.name" />
          </template>
        </el-table-column>
        <el-table-column label="颜色" width="140">
          <template #default="{ row }">
            <el-input v-model="row.color" />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="180">
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
import { fetchConfigItems, fetchCreditCards, type ConfigItem } from '@/api/config'
import { fetchUsers, type User } from '@/api/users'
import {
  createBillTag,
  deleteBillTag,
  fetchBillTags,
  updateBillTag,
  type BillTag
} from '@/api/bill-tags'

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
  presale: '预售'
}

const keyword = ref('')
const dialogVisible = ref(false)
const tagDialogVisible = ref(false)
const tagKeyword = ref('')
const newTagName = ref('')
const bills = ref<{ list: Bill[]; total: number }>({ list: [], total: 0 })
const users = ref<User[]>([])
const categories = ref<ConfigItem[]>([])
const billTags = ref<BillTag[]>([])
const options = reactive<BillOptions>({
  payment_methods: [],
  normal_bill_types: [],
  investment_bill_types: [],
  transfer_target_types: []
})

const form = reactive({
  user_id: 0,
  account_date: '',
  category_id: 0,
  bill_type: 'expense',
  payment_method: 'cash',
  is_fixed_asset: false,
  amount: '',
  remark: '',
  tags: [] as string[]
})

const selectedCategory = computed(() => categories.value.find((item) => item.id === form.category_id))
const isInvestmentCategory = computed(() => ['stock', 'wealth'].includes(selectedCategory.value?.name ?? ''))
const billTypeOptions = computed(() =>
  isInvestmentCategory.value ? options.investment_bill_types : options.normal_bill_types
)

function formatSpecialStatus(row: Bill) {
  const sourceType = row.special_status.replace('_pending', '')
  return `${sourceType || 'none'}:${row.id}`
}

async function loadBillTags() {
  const data = await fetchBillTags(tagKeyword.value)
  billTags.value = data.list
}

async function loadData() {
  const [billData, optionData, userData, categoryData] = await Promise.all([
    fetchBills(keyword.value),
    fetchBillOptions(),
    fetchUsers(),
    fetchConfigItems('account_category'),
    fetchCreditCards()
  ])
  bills.value = billData
  Object.assign(options, optionData)
  users.value = userData.list
  categories.value = categoryData.list
  await loadBillTags()
  if (!form.user_id && users.value.length > 0) {
    form.user_id = users.value[0].id
  }
  if (!form.category_id && categories.value.length > 0) {
    form.category_id = categories.value[0].id
  }
  if (!form.account_date) {
    form.account_date = new Date().toISOString().slice(0, 10)
  }
}

async function submitBill() {
  try {
    await createBill({
      user_id: form.user_id,
      account_date: form.account_date,
      category_id: form.category_id,
      bill_type: form.bill_type,
      payment_method: form.payment_method,
      is_fixed_asset: form.is_fixed_asset,
      amount: form.amount,
      tags: form.tags,
      remark: form.remark || undefined
    })
    ElMessage.success('账单已创建')
    dialogVisible.value = false
    await loadData()
  } catch {
    ElMessage.error('创建失败，请检查输入')
  }
}

function openTagManager() {
  tagDialogVisible.value = true
}

async function addTag() {
  const name = newTagName.value.trim()
  if (!name) {
    return
  }
  await createBillTag({ user_id: form.user_id, name })
  newTagName.value = ''
  await loadBillTags()
}

async function saveTag(tag: BillTag) {
  await updateBillTag(tag.id, { name: tag.name, color: tag.color || undefined })
  ElMessage.success('标签已保存')
}

async function removeTag(id: number) {
  await deleteBillTag(id)
  await loadBillTags()
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
.page {
  padding: 24px;
}
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}
.head h2 {
  margin: 0;
}
.actions {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}
.grid-2 {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
.mb-12 {
  margin-bottom: 12px;
}
.mt-12 {
  margin-top: 12px;
}
.mr-6 {
  margin-right: 6px;
}
</style>
