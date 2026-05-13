<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">余额校准</h2>
        <p style="margin: 8px 0 0; color: #64748b;">首页统计会优先使用最新不晚于结束日的校准记录作为现金基线。</p>
      </div>
      <el-tag>{{ calibrations.total }} records</el-tag>
    </div>

    <el-table :data="calibrations.list" stripe>
      <el-table-column prop="calibration_date" label="校准日期" width="140" />
      <el-table-column prop="cash_balance" label="现金余额" width="140" />
      <el-table-column prop="remark" label="备注" min-width="220" />
    </el-table>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { fetchBalanceCalibrations, type BalanceCalibration } from '@/api/balance-calibrations'

const calibrations = ref<{ list: BalanceCalibration[]; total: number }>({ list: [], total: 0 })

onMounted(async () => {
  try {
    const data = await fetchBalanceCalibrations()
    calibrations.value = { list: data.list, total: data.total }
  } catch {
    ElMessage.error('余额校准页面初始化失败')
  }
})
</script>
