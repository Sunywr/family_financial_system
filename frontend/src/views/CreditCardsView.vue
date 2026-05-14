<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">信用卡配置</h2>
        <p style="margin: 8px 0 0; color: #64748b;">阶段二已接入信用卡分页接口。</p>
      </div>
      <el-tag>{{ cards.total }} cards</el-tag>
    </div>

    <el-table :data="cards.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="user_id" label="用户ID" width="100" />
      <el-table-column prop="name" label="名称" min-width="180" />
      <el-table-column prop="billing_day" label="账单日" width="100" />
      <el-table-column prop="repayment_day" label="还款日" width="100" />
      <el-table-column prop="credit_limit" label="额度" min-width="120" />
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
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { fetchCreditCards, type CreditCard } from '@/api/config'

const currentPage = ref(1)
const pageSize = ref(20)
const cards = ref<{ list: CreditCard[]; total: number }>({
  list: [],
  total: 0
})

async function loadCards() {
  const data = await fetchCreditCards(currentPage.value, pageSize.value)
  cards.value = { list: data.list, total: data.total }
}

onMounted(async () => {
  try {
    await loadCards()
  } catch {
    cards.value = { list: [], total: 0 }
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
