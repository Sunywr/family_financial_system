<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">策略配置</h2>
        <p style="margin: 8px 0 0; color: #64748b;">阶段九接入股票和理财策略阈值，用于评分建议和冷静期约束。</p>
      </div>
      <el-button type="primary" @click="openCreate">新增策略</el-button>
    </div>

    <el-table v-loading="loading" :data="strategies.list" stripe>
      <el-table-column prop="strategy_name" label="策略名" min-width="160" />
      <el-table-column label="类型" width="100">
        <template #default="{ row }">
          <el-tag>{{ investmentTypeLabelMap[row.investment_type] || row.investment_type }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="目标代码" width="140">
        <template #default="{ row }">
          <span>{{ row.target_code || '通用策略' }}</span>
        </template>
      </el-table-column>
      <el-table-column label="风险等级" width="110">
        <template #default="{ row }">
          <el-tag :type="riskLevelTagType(row.risk_level)">
            {{ riskLevelLabelMap[row.risk_level] || row.risk_level }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="preferred_min_score" label="最低分" width="90" />
      <el-table-column prop="cooldown_days" label="冷静期" width="90" />
      <el-table-column prop="take_profit_rate" label="止盈阈值" width="110" />
      <el-table-column prop="stop_loss_rate" label="止损阈值" width="110" />
      <el-table-column label="启用" width="80">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'">{{ row.enabled ? '是' : '否' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="170">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" type="danger" @click="removeStrategy(row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="strategies.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadStrategies"
        @size-change="loadStrategies"
      />
    </div>

    <el-dialog v-model="dialogVisible" :title="editingId ? '编辑策略' : '新增策略'" width="520px">
      <el-form label-width="110px">
        <el-form-item label="用户 ID">
          <el-input-number v-model="form.user_id" :min="1" :disabled="Boolean(editingId)" />
        </el-form-item>
        <el-form-item label="投资类型">
          <el-select v-model="form.investment_type" style="width: 100%;">
            <el-option label="股票" value="stock" />
            <el-option label="理财" value="wealth" />
          </el-select>
        </el-form-item>
        <el-form-item label="策略名">
          <el-input v-model="form.strategy_name" />
        </el-form-item>
        <el-form-item label="目标代码">
          <el-input v-model="form.target_code" placeholder="留空表示通用策略" />
        </el-form-item>
        <el-form-item label="风险等级">
          <el-select v-model="form.risk_level" style="width: 100%;">
            <el-option label="低" value="low" />
            <el-option label="平衡" value="balanced" />
            <el-option label="高" value="high" />
          </el-select>
        </el-form-item>
        <el-form-item label="最低分">
          <el-input-number v-model="form.preferred_min_score" :min="0" :max="100" />
        </el-form-item>
        <el-form-item label="冷静期天数">
          <el-input-number v-model="form.cooldown_days" :min="0" :max="365" />
        </el-form-item>
        <el-form-item label="止盈阈值">
          <el-input v-model="form.take_profit_rate" placeholder="0.15" />
        </el-form-item>
        <el-form-item label="止损阈值">
          <el-input v-model="form.stop_loss_rate" placeholder="0.08" />
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="form.enabled" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.notes" type="textarea" :rows="3" />
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
import {
  createStrategy,
  deleteStrategy,
  fetchStrategies,
  updateStrategy,
  type StrategyConfig
} from '@/api/strategies'

const investmentTypeLabelMap: Record<string, string> = {
  stock: '股票',
  wealth: '理财'
}

const riskLevelLabelMap: Record<string, string> = {
  low: '低风险',
  balanced: '平衡型',
  high: '高风险'
}

const strategies = ref<{ list: StrategyConfig[]; total: number }>({ list: [], total: 0 })
const loading = ref(false)
const currentPage = ref(1)
const pageSize = ref(50)
const dialogVisible = ref(false)
const editingId = ref<number | null>(null)
const currentUserId = requireCurrentUserId()
const form = ref({
  user_id: currentUserId,
  investment_type: 'stock',
  target_code: '',
  strategy_name: '',
  enabled: true,
  risk_level: 'balanced',
  preferred_min_score: 68,
  cooldown_days: 3,
  take_profit_rate: '0.15',
  stop_loss_rate: '0.08',
  notes: ''
})

function resetForm() {
  form.value = {
    user_id: currentUserId,
    investment_type: 'stock',
    target_code: '',
    strategy_name: '',
    enabled: true,
    risk_level: 'balanced',
    preferred_min_score: 68,
    cooldown_days: 3,
    take_profit_rate: '0.15',
    stop_loss_rate: '0.08',
    notes: ''
  }
}

function riskLevelTagType(level: string): '' | 'success' | 'warning' | 'danger' {
  if (level === 'low') return 'success'
  if (level === 'balanced') return 'warning'
  if (level === 'high') return 'danger'
  return ''
}

async function loadStrategies() {
  loading.value = true
  try {
    const data = await fetchStrategies(currentPage.value, pageSize.value)
    strategies.value = { list: data.list, total: data.total }
  } finally {
    loading.value = false
  }
}

function openCreate() {
  editingId.value = null
  resetForm()
  dialogVisible.value = true
}

function openEdit(row: StrategyConfig) {
  editingId.value = row.id
  form.value = {
    user_id: row.user_id,
    investment_type: row.investment_type,
    target_code: row.target_code ?? '',
    strategy_name: row.strategy_name,
    enabled: row.enabled,
    risk_level: row.risk_level,
    preferred_min_score: row.preferred_min_score,
    cooldown_days: row.cooldown_days,
    take_profit_rate: row.take_profit_rate,
    stop_loss_rate: row.stop_loss_rate,
    notes: row.notes ?? ''
  }
  dialogVisible.value = true
}

async function submitForm() {
  try {
    const payload = {
      ...form.value,
      target_code: form.value.target_code || null,
      notes: form.value.notes || null
    }

    if (editingId.value) {
      await updateStrategy(editingId.value, {
        investment_type: payload.investment_type,
        target_code: payload.target_code,
        strategy_name: payload.strategy_name,
        enabled: payload.enabled,
        risk_level: payload.risk_level,
        preferred_min_score: payload.preferred_min_score,
        cooldown_days: payload.cooldown_days,
        take_profit_rate: payload.take_profit_rate,
        stop_loss_rate: payload.stop_loss_rate,
        notes: payload.notes
      })
    } else {
      await createStrategy(payload)
    }

    dialogVisible.value = false
    await loadStrategies()
    ElMessage.success('策略已保存')
  } catch {
    ElMessage.error('策略保存失败')
  }
}

async function removeStrategy(id: number) {
  try {
    await deleteStrategy(id)
    await loadStrategies()
    ElMessage.success('策略已删除')
  } catch {
    ElMessage.error('删除失败')
  }
}

onMounted(async () => {
  try {
    await loadStrategies()
  } catch {
    ElMessage.error('策略页面初始化失败')
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
