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

    <div v-if="dashboardContextLabel" class="context-banner">
      <el-alert :title="`当前筛选来自首页：${dashboardContextLabel}`" type="info" :closable="false" show-icon />
      <div class="context-actions">
        <el-button text type="primary" @click="goDashboard">返回首页</el-button>
      </div>
    </div>

    <div v-if="isMobileView" class="mobile-cards">
      <article v-for="row in bills.list" :key="row.id" class="mobile-card panel">
        <div class="mobile-card-head">
          <strong>#{{ row.id }} · {{ row.category_name }}</strong>
          <span>{{ row.account_date }}</span>
        </div>
        <div class="mobile-card-row">
          <span>金额</span>
          <strong>{{ row.amount }}</strong>
        </div>
        <div class="mobile-card-row">
          <span>类型</span>
          <el-tag :type="billTypeTagType(row.bill_type)">{{ billTypeMap[row.bill_type] || row.bill_type }}</el-tag>
        </div>
        <div class="mobile-card-row">
          <span>方式</span>
          <el-tag :type="paymentMethodTagType(row.payment_method)">{{ formatPaymentMethod(row) }}</el-tag>
        </div>
        <div v-if="formatRelation(row)" class="mobile-card-row">
          <span>关联</span>
          <el-tag size="small" type="warning">{{ formatRelation(row) }}</el-tag>
        </div>
        <div v-if="row.tags?.length" class="mobile-tags">
          <el-tag v-for="tag in row.tags || []" :key="tag" size="small" class="mr-4">{{ tag }}</el-tag>
        </div>
        <p v-if="row.remark" class="mobile-remark">{{ row.remark }}</p>
        <div class="mobile-card-actions">
          <el-button size="small" @click="copyBill(row)">复制</el-button>
          <el-button size="small" type="primary" plain @click="editBill(row)">修改</el-button>
          <el-button size="small" type="danger" plain @click="removeBill(row)">删除</el-button>
        </div>
      </article>
      <div v-if="!bills.list.length" class="mobile-empty">暂无账单数据</div>
    </div>

    <el-table v-else :data="bills.list" stripe>
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
          <el-tag :type="paymentMethodTagType(row.payment_method)">{{ formatPaymentMethod(row) }}</el-tag>
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
      <el-table-column label="操作" width="220" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="copyBill(row)">复制</el-button>
          <el-button size="small" type="primary" plain @click="editBill(row)">修改</el-button>
          <el-button size="small" type="danger" plain @click="removeBill(row)">删除</el-button>
        </template>
      </el-table-column>
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
    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="760px" @closed="resetForm">
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
              @click="form.bill_type = bt; if (form.billGroup === 'investment') form.investment_action = bt"
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

        <template v-if="form.billGroup === 'investment' && form.category_id">
          <el-form-item label="支付方式">
            <el-tag :type="form.payment_method === 'stock_account' ? 'warning' : 'success'" size="large">
              {{ paymentMap[form.payment_method] || form.payment_method }}（自动）
            </el-tag>
          </el-form-item>

          <el-form-item v-if="isOpenPosition" label="产品名称">
            <el-input v-model="form.product_name" placeholder="例：贵州茅台" />
          </el-form-item>

          <el-form-item v-if="isOpenPosition" label="产品代码">
            <el-input v-model="form.product_code" placeholder="例：600519" />
          </el-form-item>

          <el-form-item v-if="isOpenPosition" label="机构名称">
            <el-select
              v-model="form.organization_category"
              placeholder="请选择机构分类"
              style="width: 100%; margin-bottom: 8px;"
              @change="onOrganizationCategoryChange"
            >
              <el-option
                v-for="category in institutionCategories"
                :key="category.code"
                :label="category.name"
                :value="category.code"
              />
            </el-select>
            <el-select
              v-model="form.organization_name"
              filterable
              placeholder="请选择机构名称"
              style="width: 100%;"
            >
              <el-option
                v-for="name in organizationNameOptions"
                :key="name"
                :label="name"
                :value="name"
              />
            </el-select>
          </el-form-item>

          <el-form-item :label="shareAmountLabel">
            <el-input v-model="form.share_amount" :placeholder="shareAmountPlaceholder" />
            <div v-if="currentRelatedInvestment && !isOpenPosition" class="field-hint">
              当前持仓：{{ currentRelatedInvestment.total_shares }}
              <template v-if="form.bill_type === 'reduce_position'">，卖出份额不能超过当前持仓</template>
            </div>
          </el-form-item>

          <el-form-item v-if="!isOpenPosition" label="关联投资">
            <el-select v-model="form.related_investment_id" placeholder="请选择关联投资" style="width: 100%;" clearable>
              <el-option
                v-for="inv in investments"
                :key="inv.id"
                :label="`${inv.name}（${inv.code}）`"
                :value="inv.id"
              />
            </el-select>
          </el-form-item>
        </template>

        <template v-else-if="form.billGroup === 'normal'">
          <el-form-item label="现金/信用卡">
            <div class="btn-group">
              <el-button
                :type="form.payment_base === 'cash' ? 'primary' : 'default'"
                size="small"
                @click="setPaymentBase('cash')"
              >现金</el-button>
              <el-button
                :type="form.payment_base === 'credit_card' ? 'primary' : 'default'"
                size="small"
                @click="setPaymentBase('credit_card')"
              >信用卡</el-button>
            </div>
          </el-form-item>

          <el-form-item v-if="form.payment_base === 'credit_card'" label="关联信用卡">
            <el-select v-model="form.credit_card_id" placeholder="请选择信用卡" style="width: 100%;" clearable>
              <el-option v-for="card in creditCards" :key="card.id" :label="card.name" :value="card.id" />
            </el-select>
          </el-form-item>

          <el-form-item label="是否分期">
            <div class="btn-group">
              <el-button
                :type="!form.is_installment ? 'primary' : 'default'"
                size="small"
                @click="setInstallment(false)"
              >否</el-button>
              <el-button
                :type="form.is_installment ? 'primary' : 'default'"
                size="small"
                @click="setInstallment(true)"
              >是</el-button>
            </div>
          </el-form-item>

          <el-form-item v-if="form.is_installment" label="分期期数">
            <el-input-number v-model="form.installment_months" :min="1" :max="120" style="width: 180px;" />
          </el-form-item>

          <el-form-item label="是否预售">
            <div class="btn-group">
              <el-button
                :type="!form.is_presale ? 'primary' : 'default'"
                size="small"
                @click="setPresale(false)"
              >否</el-button>
              <el-button
                :type="form.is_presale ? 'primary' : 'default'"
                size="small"
                @click="setPresale(true)"
              >是</el-button>
            </div>
          </el-form-item>

          <el-form-item label="是否资产">
            <div class="btn-group">
              <el-button
                :type="!form.is_fixed_asset ? 'primary' : 'default'"
                size="small"
                @click="setAsset(false)"
              >否</el-button>
              <el-button
                :type="form.is_fixed_asset ? 'primary' : 'default'"
                size="small"
                @click="setAsset(true)"
              >是</el-button>
            </div>
          </el-form-item>

          <el-form-item v-if="form.is_fixed_asset" label="关联资产">
            <el-select v-model="form.related_asset_id" placeholder="请选择关联资产" style="width: 100%;" clearable>
              <el-option v-for="asset in assets" :key="asset.id" :label="asset.name" :value="asset.id" />
            </el-select>
          </el-form-item>
        </template>

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
            remote
            :remote-method="remoteSearchTags"
            :loading="tagSearchLoading"
            placeholder="搜索或输入并回车创建新标签"
            style="width: 100%;"
          >
            <el-option v-for="tag in searchedTags" :key="tag.id" :label="tag.name" :value="tag.name" />
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
        <el-input v-model="tagKeyword" placeholder="检索标签" style="width: 220px;" @keyup.enter="searchTagManager" />
        <el-button @click="searchTagManager">查询</el-button>
      </div>
      <el-table :data="tagManagerTags" stripe>
        <el-table-column prop="id" label="ID" width="70" />
        <el-table-column label="标签名">
          <template #default="{ row }">
            <el-input v-model="row.name" size="small" />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="160">
          <template #default="{ row }">
            <el-button size="small" @click="saveTag(row)">保存</el-button>
            <el-button size="small" type="danger" @click="removeTag(row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-wrap">
        <el-pagination
          :current-page="tagManagerPage"
          :page-size="tagManagerPageSize"
          :page-sizes="[20, 50, 100]"
          :total="tagManagerTotal"
          layout="total, sizes, prev, pager, next"
          @current-change="onTagManagerPageChange"
          @size-change="onTagManagerPageSizeChange"
        />
      </div>
      <div class="actions mt-12">
        <el-input v-model="newTagName" placeholder="新标签名" style="width: 220px;" />
        <el-button type="primary" @click="addTag">新增标签</el-button>
      </div>
    </el-dialog>

    <el-button v-if="isMobileView" class="mobile-fab" type="primary" circle @click="openCreate">+</el-button>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { createBill, deleteBill, fetchBillOptions, fetchBills, updateBill, type Bill, type BillOptions } from '@/api/bills'
import { fetchAssets, type Asset } from '@/api/assets'
import { fetchConfigItems, fetchCreditCards, type ConfigItem, type CreditCard } from '@/api/config'
import { fetchUsers, type User } from '@/api/users'
import {
  createBillTag,
  deleteBillTag,
  fetchBillTags,
  fetchBillTagsPage,
  syncBillTagsFromLegacy,
  fetchTopBillTags,
  updateBillTag,
  type BillTag
} from '@/api/bill-tags'
import { getStoredUser } from '@/api/client'
import { fetchInvestments, type Investment } from '@/api/investments'
import { fetchInstitutionCategories, type InstitutionCategory } from '@/api/institutions'
const route = useRoute()
const router = useRouter()

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

function paymentMethodTagType(paymentMethod: string): '' | 'success' | 'warning' {
  return paymentMethod === 'cash' ? 'success' : 'warning'
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
    if (transferType === 'system_user') {
      return row.transfer_target_user_name
        ? `${label}·${row.transfer_target_user_name}`
        : row.transfer_target_user_id
          ? `${label}#${row.transfer_target_user_id}`
          : label
    }
    if (transferType === 'investment') {
      return row.related_investment_name
        ? `${label}·${row.related_investment_name}`
        : row.related_investment_id
          ? `${label}#${row.related_investment_id}`
          : label
    }
    if (transferType === 'asset') {
      return row.related_asset_name
        ? `${label}·${row.related_asset_name}`
        : row.related_asset_id
          ? `${label}#${row.related_asset_id}`
          : label
    }
    return label
  }

  if (row.payment_method === 'presale') return '预售'
  if (row.related_investment_name) return `投资·${row.related_investment_name}`
  if (row.related_investment_id) return `投资#${row.related_investment_id}`
  if (row.related_asset_name) return `资产·${row.related_asset_name}`
  if (row.related_asset_id) return `资产#${row.related_asset_id}`
  if (row.related_debt_id) return `债务#${row.related_debt_id}`

  const status = row.special_status || ''
  if (status === 'debt_pending' && (row.credit_card_name || row.credit_card_id)) {
    return `债务·${row.credit_card_name || creditCardDisplay(row.credit_card_id)}`
  }
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
const isMobileView = ref(typeof window !== 'undefined' ? window.innerWidth <= 900 : false)
const tagDialogVisible = ref(false)
const tagKeyword = ref('')
const newTagName = ref('')
const bills = ref<{ list: Bill[]; total: number }>({ list: [], total: 0 })
const users = ref<User[]>([])
const creditCards = ref<CreditCard[]>([])
const assets = ref<Asset[]>([])
const categories = ref<ConfigItem[]>([])
const allTags = ref<BillTag[]>([])
const topTags = ref<BillTag[]>([])
const tagManagerTags = ref<BillTag[]>([])
const searchedTags = ref<BillTag[]>([])
const tagSearchLoading = ref(false)
const investments = ref<Investment[]>([])
const institutionCategories = ref<InstitutionCategory[]>([])
const tagManagerPage = ref(1)
const tagManagerPageSize = ref(20)
const tagManagerTotal = ref(0)
const triedLegacyTagSync = ref(false)
const options = reactive<BillOptions>({
  payment_methods: [],
  normal_bill_types: [],
  investment_bill_types: [],
  transfer_target_types: []
})

const currentUser = getStoredUser()
const isAdmin = computed(() => currentUser?.username === 'admin')
const currentUserId = computed(() => currentUser?.id ?? 0)
const dashboardContextMap: Record<string, string> = {
  'credit-card-pending': '信用卡待还',
  'credit-card-row': '信用卡对账',
  'pending-window': '待处理账单窗口'
}
const dashboardContextLabel = computed(() => {
  if (route.query.from !== 'dashboard') return ''
  const key = typeof route.query.context === 'string' ? route.query.context : ''
  return dashboardContextMap[key] || '首页钻取'
})
const editingBillId = ref<number | null>(null)
const dialogMode = ref<'create' | 'copy' | 'edit'>('create')
const dialogTitle = computed(() => {
  if (dialogMode.value === 'edit') return '修改账单'
  if (dialogMode.value === 'copy') return '复制账单'
  return '新增账单'
})

const form = reactive({
  user_id: 0,
  account_date: '',
  category_id: 0,
  billGroup: '' as '' | 'normal' | 'investment',
  bill_type: '',
  payment_method: 'cash',
  payment_base: 'cash' as 'cash' | 'credit_card',
  credit_card_id: undefined as number | undefined,
  is_installment: false,
  installment_months: 12,
  is_presale: false,
  is_fixed_asset: false,
  related_asset_id: undefined as number | undefined,
  amount: '',
  remark: '',
  tags: [] as string[],
  // investment-specific fields
  investment_action: '',
  related_investment_id: undefined as number | undefined,
  product_code: '',
  product_name: '',
  organization_category: '',
  organization_name: '',
  share_amount: ''
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
const selectedCategoryName = computed(
  () => categories.value.find((c) => c.id === form.category_id)?.name ?? ''
)
const isOpenPosition = computed(() => form.bill_type === 'open_position')
const currentRelatedInvestment = computed(() =>
  investments.value.find((investment) => investment.id === form.related_investment_id)
)
const selectedInstitutionCategory = computed(() =>
  institutionCategories.value.find((category) => category.code === form.organization_category)
)
const organizationNameOptions = computed(() => {
  return (selectedInstitutionCategory.value?.providers || []).map((provider) => provider.name)
})
const shareAmountLabel = computed(() => {
  if (form.bill_type === 'open_position') return '购买份额'
  if (form.bill_type === 'add_position') return '本次加仓份额'
  if (form.bill_type === 'reduce_position') return '本次卖出份额'
  if (form.bill_type === 'dividend') return '本次分红对应份额'
  return '份数/份额'
})
const shareAmountPlaceholder = computed(() => {
  if (form.bill_type === 'open_position') return '填写建仓时购买的份额'
  if (form.bill_type === 'add_position') return '填写本次加仓购买的份额'
  if (form.bill_type === 'reduce_position') return '填写本次减仓卖出的份额'
  if (form.bill_type === 'dividend') return '填写本次分红对应的份额'
  return '填写本次操作份额'
})

function parsePositiveDecimal(value: string) {
  const parsed = Number(value)
  if (!Number.isFinite(parsed) || parsed <= 0) return undefined
  return parsed
}

function selectBillGroup(group: 'normal' | 'investment') {
  form.billGroup = group
  form.bill_type = ''
  form.category_id = 0
  form.payment_method = 'cash'
  form.payment_base = 'cash'
  form.credit_card_id = undefined
  form.is_installment = false
  form.installment_months = 12
  form.is_presale = false
  form.is_fixed_asset = false
  form.related_asset_id = undefined
  form.investment_action = ''
  form.related_investment_id = undefined
  form.product_code = ''
  form.product_name = ''
  form.organization_category = ''
  form.organization_name = ''
  form.share_amount = ''
  investments.value = []
  institutionCategories.value = []
}

function setPaymentBase(base: 'cash' | 'credit_card') {
  form.payment_base = base
  if (base === 'cash') {
    form.credit_card_id = undefined
    form.is_installment = false
    form.installment_months = 12
  }
}

function setInstallment(enabled: boolean) {
  form.is_installment = enabled
  if (enabled) {
    form.payment_base = 'credit_card'
    form.is_presale = false
    if (!form.installment_months || form.installment_months <= 0) {
      form.installment_months = 12
    }
  }
}

function setPresale(enabled: boolean) {
  form.is_presale = enabled
  if (enabled) {
    form.is_installment = false
    form.installment_months = 12
  }
}

function setAsset(enabled: boolean) {
  form.is_fixed_asset = enabled
  if (!enabled) {
    form.related_asset_id = undefined
  }
}

function resolvePaymentMethod() {
  if (form.billGroup === 'investment') return form.payment_method
  if (form.is_presale) return 'presale'
  return form.payment_base
}

function selectCategory(id: number) {
  form.category_id = id
  void loadTopTags(id)
  const catName = categories.value.find((c) => c.id === id)?.name ?? ''
  if (catName === 'stock') {
    form.payment_method = 'stock_account'
    form.organization_category = ''
    form.organization_name = ''
    void loadInvestmentList('stock')
    void loadInstitutionCategories('stock')
  } else if (catName === 'wealth') {
    form.payment_method = 'cash'
    form.organization_category = ''
    form.organization_name = ''
    void loadInvestmentList('wealth')
    void loadInstitutionCategories('wealth')
  } else {
    investments.value = []
    institutionCategories.value = []
    form.organization_category = ''
    form.organization_name = ''
  }
  form.related_investment_id = undefined
}

async function loadInstitutionCategories(investmentType: 'stock' | 'wealth') {
  try {
    institutionCategories.value = await fetchInstitutionCategories(investmentType)
    syncOrganizationCategoryByName(form.organization_name)
    const categoryStillValid = institutionCategories.value.some(
      (category) => category.code === form.organization_category
    )
    if (!categoryStillValid) {
      form.organization_category = ''
    }
    if (!form.organization_category && institutionCategories.value.length > 0) {
      form.organization_category = institutionCategories.value[0].code
    }
    if (form.organization_name && !organizationNameOptions.value.includes(form.organization_name)) {
      form.organization_name = ''
    }
  } catch {
    institutionCategories.value = []
    form.organization_category = ''
    form.organization_name = ''
  }
}

function syncOrganizationCategoryByName(name: string) {
  const normalized = name.trim()
  if (!normalized) return
  const matched = institutionCategories.value.find((category) =>
    category.providers.some((provider) => provider.name === normalized)
  )
  if (matched) {
    form.organization_category = matched.code
  }
}

function onOrganizationCategoryChange() {
  if (form.organization_name && !organizationNameOptions.value.includes(form.organization_name)) {
    form.organization_name = ''
  }
}

async function loadInvestmentList(investmentType: string) {
  try {
    const data = await fetchInvestments({ investment_type: investmentType, show_sold: false, page_size: 200 })
    investments.value = data.list
  } catch {
    investments.value = []
  }
}

function toggleTag(name: string) {
  const idx = form.tags.indexOf(name)
  if (idx >= 0) form.tags.splice(idx, 1)
  else form.tags.push(name)
}

function resetForm() {
  editingBillId.value = null
  dialogMode.value = 'create'
  form.user_id = isAdmin.value ? (users.value[0]?.id ?? 0) : currentUserId.value
  form.account_date = new Date().toISOString().slice(0, 10)
  form.category_id = 0
  form.billGroup = ''
  form.bill_type = ''
  form.payment_method = 'cash'
  form.payment_base = 'cash'
  form.credit_card_id = undefined
  form.is_installment = false
  form.installment_months = 12
  form.is_presale = false
  form.is_fixed_asset = false
  form.related_asset_id = undefined
  form.amount = ''
  form.remark = ''
  form.tags = []
  form.investment_action = ''
  form.related_investment_id = undefined
  form.product_code = ''
  form.product_name = ''
  form.organization_category = ''
  form.organization_name = ''
  form.share_amount = ''
  investments.value = []
  institutionCategories.value = []
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

function goDashboard() {
  void router.push({ name: 'dashboard' })
}

function syncViewportMode() {
  isMobileView.value = window.innerWidth <= 900
}

function applyRouteQuery() {
  const startDate = typeof route.query.start_date === 'string' ? route.query.start_date : ''
  const endDate = typeof route.query.end_date === 'string' ? route.query.end_date : ''
  keyword.value = typeof route.query.keyword === 'string' ? route.query.keyword : ''
  filterCategoryId.value =
    typeof route.query.category_id === 'string' ? Number(route.query.category_id) || undefined : undefined
  if (startDate && endDate) {
    dateRange.value = [startDate, endDate]
  }
  const paymentMethod = typeof route.query.payment_method === 'string' ? route.query.payment_method : ''
  const creditCardId = typeof route.query.credit_card_id === 'string' ? route.query.credit_card_id : ''
  if (paymentMethod === 'credit_card' && creditCardId) {
    filterMethodKey.value = `credit_card:${creditCardId}`
  } else if (paymentMethod) {
    filterMethodKey.value = paymentMethod
  }
}

async function loadAllTags() {
  try {
    const data = await fetchBillTags('')
    allTags.value = data.list

    if (allTags.value.length === 0 && !triedLegacyTagSync.value) {
      triedLegacyTagSync.value = true
      if (currentUserId.value > 0) {
        const syncResult = await syncBillTagsFromLegacy(currentUserId.value)
        if (syncResult.imported > 0) {
          const refreshed = await fetchBillTags('')
          allTags.value = refreshed.list
          ElMessage.success(`已从生产库同步 ${syncResult.imported} 个标签`)
        }
      }
    }
  } catch {
    allTags.value = []
  }
}

async function loadTagManagerPage() {
  try {
    const data = await fetchBillTagsPage(tagManagerPage.value, tagManagerPageSize.value, tagKeyword.value)
    tagManagerTags.value = data.list
    tagManagerTotal.value = data.total
  } catch {
    tagManagerTags.value = []
    tagManagerTotal.value = 0
  }
}

function searchTagManager() {
  tagManagerPage.value = 1
  loadTagManagerPage()
}

function onTagManagerPageChange(page: number) {
  tagManagerPage.value = page
  loadTagManagerPage()
}

function onTagManagerPageSizeChange(size: number) {
  tagManagerPageSize.value = size
  tagManagerPage.value = 1
  loadTagManagerPage()
}

async function loadTopTags(categoryId?: number) {
  try {
    topTags.value = await fetchTopBillTags(categoryId)
    searchedTags.value = topTags.value
  } catch {
    topTags.value = []
    searchedTags.value = []
  }
}

async function remoteSearchTags(keyword: string) {
  if (!keyword) {
    searchedTags.value = topTags.value
    return
  }
  tagSearchLoading.value = true
  try {
    const data = await fetchBillTagsPage(1, 30, keyword)
    searchedTags.value = data.list
  } catch {
    searchedTags.value = []
  } finally {
    tagSearchLoading.value = false
  }
}

async function loadData() {
  await loadBills()
  const [optionResult, userResult, categoryResult, creditCardResult, assetResult] = await Promise.allSettled([
    fetchBillOptions(),
    fetchUsers(),
    fetchConfigItems('account_category'),
    fetchCreditCards(),
    fetchAssets('', 1, 500)
  ])
  if (optionResult.status === 'fulfilled') Object.assign(options, optionResult.value)
  if (userResult.status === 'fulfilled') users.value = userResult.value.list
  if (categoryResult.status === 'fulfilled') categories.value = categoryResult.value.list
  if (creditCardResult.status === 'fulfilled') {
    creditCards.value = creditCardResult.value.list
    creditCardNameMap.value = Object.fromEntries(creditCards.value.map((item) => [item.id, item.name]))
  }
  if (assetResult.status === 'fulfilled') assets.value = assetResult.value.list
  await Promise.allSettled([loadAllTags(), loadTopTags(), loadTagManagerPage()])
  // Note: loadTopTags no longer requires userId
  resetForm()
}

function openCreate() {
  resetForm()
  dialogVisible.value = true
}

function fillFormByBill(row: Bill, mode: 'copy' | 'edit') {
  dialogMode.value = mode
  editingBillId.value = mode === 'edit' ? row.id : null
  form.user_id = row.user_id
  form.account_date = row.account_date
  form.category_id = row.category_id
  form.bill_type = row.bill_type
  form.billGroup = ['open_position', 'add_position', 'reduce_position', 'dividend'].includes(row.bill_type)
    ? 'investment'
    : 'normal'

  form.payment_method = row.payment_method
  form.payment_base = row.payment_method === 'credit_card' || row.payment_method === 'installment'
    ? 'credit_card'
    : 'cash'
  form.credit_card_id = row.credit_card_id ?? undefined
  form.is_installment = Boolean(row.is_installment)
  form.installment_months = row.installment_months ?? 12
  form.is_presale = row.payment_method === 'presale'
  form.is_fixed_asset = Boolean(row.is_fixed_asset)
  form.related_asset_id = row.related_asset_id ?? undefined
  form.amount = row.amount
  form.remark = row.remark || ''
  form.tags = [...(row.tags || [])]
  // investment fields
  form.investment_action = row.investment_action ?? ''
  form.related_investment_id = row.related_investment_id ?? undefined
  form.product_code = row.product_code ?? ''
  form.product_name = row.product_name ?? ''
  form.organization_category = ''
  form.organization_name = row.organization_name ?? ''
  form.share_amount = ''
  if (form.billGroup === 'investment') {
    const catName = categories.value.find((c) => c.id === row.category_id)?.name ?? ''
    if (catName === 'stock' || catName === 'wealth') {
      void loadInvestmentList(catName)
      void loadInstitutionCategories(catName)
    }
  }
}

function copyBill(row: Bill) {
  fillFormByBill(row, 'copy')
  dialogVisible.value = true
}

function editBill(row: Bill) {
  fillFormByBill(row, 'edit')
  dialogVisible.value = true
}

async function removeBill(row: Bill) {
  try {
    await ElMessageBox.confirm(`确定删除账单 #${row.id} 吗？`, '删除确认', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消'
    })
    await deleteBill(row.id)
    ElMessage.success('账单已删除')
    await loadBills()
  } catch (error) {
    if (error === 'cancel' || error === 'close') return
    ElMessage.error('删除失败')
  }
}

async function submitBill() {
  if (!form.bill_type) { ElMessage.warning('请选择账单类型'); return }
  if (!form.category_id) { ElMessage.warning('请选择分类'); return }
  const payment_method = resolvePaymentMethod()
  if (form.billGroup === 'normal' && payment_method === 'credit_card' && !form.credit_card_id) {
    ElMessage.warning('请选择关联信用卡')
    return
  }
  if (form.billGroup === 'normal' && form.is_installment && (!form.installment_months || form.installment_months <= 0)) {
    ElMessage.warning('请填写分期期数')
    return
  }
  if (form.billGroup === 'normal' && form.is_fixed_asset && !form.related_asset_id) {
    ElMessage.warning('请选择关联资产')
    return
  }
  if (form.billGroup === 'investment') {
    if (!form.investment_action) { ElMessage.warning('请选择投资操作类型'); return }
    if (form.bill_type === 'open_position') {
      if (!form.product_name.trim()) { ElMessage.warning('请填写产品名称'); return }
      if (!form.product_code.trim()) { ElMessage.warning('请填写产品代码'); return }
      if (!form.organization_category.trim()) { ElMessage.warning('请选择机构分类'); return }
      if (!form.organization_name.trim()) { ElMessage.warning('请填写机构名称'); return }
      if (!form.share_amount.trim()) { ElMessage.warning('请填写购买份数'); return }
    } else {
      if (!form.related_investment_id) { ElMessage.warning('请选择关联投资'); return }
      if (!form.share_amount.trim()) { ElMessage.warning('请填写份数/份额'); return }
      if (form.bill_type === 'reduce_position') {
        const currentHolding = parsePositiveDecimal(currentRelatedInvestment.value?.total_shares || '')
        const reduceShares = parsePositiveDecimal(form.share_amount)
        if (currentHolding !== undefined && reduceShares !== undefined && reduceShares > currentHolding) {
          ElMessage.warning('减仓份额不能超过当前持仓份额')
          return
        }
      }
    }
  }

  const payload = {
    account_date: form.account_date,
    category_id: form.category_id,
    bill_type: form.bill_type,
    payment_method,
    is_fixed_asset: form.is_fixed_asset,
    amount: form.amount,
    tags: form.tags.length ? form.tags : undefined,
    remark: form.remark || undefined,
    credit_card_id: payment_method === 'credit_card' ? form.credit_card_id : undefined,
    is_installment: form.is_installment || undefined,
    installment_months: form.is_installment ? form.installment_months : undefined,
    related_asset_id: form.is_fixed_asset ? form.related_asset_id : undefined,
    investment_action: form.billGroup === 'investment' ? form.investment_action : undefined,
    product_code: form.billGroup === 'investment' && form.bill_type === 'open_position' ? form.product_code : undefined,
    product_name: form.billGroup === 'investment' && form.bill_type === 'open_position' ? form.product_name : undefined,
    organization_name: form.billGroup === 'investment' && form.bill_type === 'open_position' ? form.organization_name : undefined,
    share_amount: form.billGroup === 'investment' ? form.share_amount : undefined,
    related_investment_id: form.billGroup === 'investment' && form.bill_type !== 'open_position' ? form.related_investment_id : undefined
  }

  try {
    if (dialogMode.value === 'edit' && editingBillId.value) {
      await updateBill(editingBillId.value, payload)
      ElMessage.success('账单已更新')
    } else {
      await createBill({
        user_id: isAdmin.value ? form.user_id : currentUserId.value,
        ...payload
      })
      ElMessage.success(dialogMode.value === 'copy' ? '账单已复制' : '账单已创建')
    }
    dialogVisible.value = false
    await loadBills()
  } catch {
    ElMessage.error('保存失败，请检查输入')
  }
}

function openTagManager() {
  tagManagerPage.value = 1
  loadTagManagerPage()
  tagDialogVisible.value = true
}

async function addTag() {
  const name = newTagName.value.trim()
  if (!name) return
  try {
    await createBillTag({ name })
    newTagName.value = ''
    await Promise.allSettled([loadAllTags(), loadTopTags(), loadTagManagerPage()])
  } catch {
    ElMessage.error('新增标签失败')
  }
}

async function saveTag(tag: BillTag) {
  try {
    await updateBillTag(tag.id, { name: tag.name })
    ElMessage.success('标签已保存')
    await loadTagManagerPage()
  } catch {
    ElMessage.error('保存失败')
  }
}

async function removeTag(id: number) {
  try {
    await deleteBillTag(id)
    await Promise.allSettled([loadAllTags(), loadTopTags(), loadTagManagerPage()])
  } catch {
    ElMessage.error('删除失败')
  }
}

onMounted(async () => {
  syncViewportMode()
  window.addEventListener('resize', syncViewportMode)
  try {
    applyRouteQuery()
    await loadData()
  } catch {
    ElMessage.error('账单页面初始化失败')
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', syncViewportMode)
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
.field-hint { margin-top: 6px; color: #6b7280; font-size: 12px; line-height: 1.5; }
.mb-8 { margin-bottom: 8px; }
.mb-12 { margin-bottom: 12px; }
.mt-12 { margin-top: 12px; }
.mr-4 { margin-right: 4px; }
.pagination-wrap { margin-top: 16px; display: flex; justify-content: flex-end; }
.context-banner { margin-bottom: 16px; display: flex; flex-direction: column; gap: 8px; }
.context-actions { display: flex; justify-content: flex-end; }

.mobile-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mobile-card {
  padding: 12px;
  border-radius: 14px;
}

.mobile-card-head {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  color: #475569;
}

.mobile-card-row {
  margin-top: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.mobile-tags {
  margin-top: 8px;
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
  flex-wrap: wrap;
}

.mobile-empty {
  padding: 18px 10px;
  text-align: center;
  color: #94a3b8;
}

.mobile-fab {
  position: fixed;
  right: 18px;
  bottom: 18px;
  z-index: 40;
  width: 44px;
  height: 44px;
  box-shadow: 0 12px 24px rgba(37, 99, 235, 0.3);
}
</style>
