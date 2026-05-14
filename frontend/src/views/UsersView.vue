<template>
  <section class="panel" style="padding: 24px;">
    <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap; margin-bottom: 18px;">
      <div>
        <h2 style="margin: 0;">用户</h2>
        <p style="margin: 8px 0 0; color: #64748b;">阶段二已接入后端用户分页接口。</p>
      </div>
      <el-tag>{{ users.total }} users</el-tag>
    </div>

    <el-table :data="users.list" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="username" label="用户名" />
      <el-table-column prop="display_name" label="显示名" />
      <el-table-column prop="role" label="角色" />
      <el-table-column label="状态" width="100">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'">
            {{ row.enabled ? '启用' : '禁用' }}
          </el-tag>
        </template>
      </el-table-column>
    </el-table>
    <div class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[20, 50, 100]"
        :total="users.total"
        layout="total, sizes, prev, pager, next"
        @current-change="loadUsers"
        @size-change="loadUsers"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { fetchUsers, type User } from '@/api/users'

const currentPage = ref(1)
const pageSize = ref(20)
const users = ref<{ list: User[]; total: number }>({
  list: [],
  total: 0
})

async function loadUsers() {
  const data = await fetchUsers(currentPage.value, pageSize.value)
  users.value = { list: data.list, total: data.total }
}

onMounted(async () => {
  try {
    await loadUsers()
  } catch {
    users.value = { list: [], total: 0 }
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
