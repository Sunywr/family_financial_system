<template>
  <section class="audit-page">
    <div class="metric-grid" style="margin-bottom: 18px;">
      <article class="panel metric-card">
        <div class="metric-label">旧库总行数</div>
        <div class="metric-value">{{ summary?.totals.source_rows ?? 0 }}</div>
      </article>
      <article class="panel metric-card">
        <div class="metric-label">新库总行数</div>
        <div class="metric-value">{{ summary?.totals.target_rows ?? 0 }}</div>
      </article>
      <article class="panel metric-card">
        <div class="metric-label">完全匹配模块</div>
        <div class="metric-value">
          {{ summary?.totals.matched_modules ?? 0 }}/{{ summary?.totals.total_modules ?? 0 }}
        </div>
      </article>
      <article class="panel metric-card">
        <div class="metric-label">权限快照</div>
        <div class="metric-value">{{ summary?.totals.permission_snapshots ?? 0 }}</div>
      </article>
    </div>

    <section class="panel audit-section">
      <div class="audit-section-head">
        <div>
          <h2 class="audit-title">迁移状态</h2>
          <p class="audit-subtitle">
            对比旧 Django 数据库 `pfm` 和新系统 `ffs` 的核心模块数量。
          </p>
        </div>
        <div class="audit-status-group">
          <el-tag :type="summary?.legacy_database_configured ? 'success' : 'warning'">
            {{ summary?.legacy_database_configured ? '已配置旧库连接' : '未配置旧库连接' }}
          </el-tag>
          <el-tag :type="summary?.legacy_database_reachable ? 'success' : 'danger'">
            {{ summary?.legacy_database_reachable ? '旧库可连接' : '旧库不可连接' }}
          </el-tag>
        </div>
      </div>

      <el-table :data="summary?.modules ?? []" stripe>
        <el-table-column prop="label" label="模块" min-width="120" />
        <el-table-column prop="source_table" label="旧表" min-width="180" />
        <el-table-column prop="target_table" label="新表" min-width="180" />
        <el-table-column prop="source_count" label="旧库数量" width="110" />
        <el-table-column prop="target_count" label="新库数量" width="110" />
        <el-table-column label="差值" width="100">
          <template #default="{ row }">
            <span :class="row.difference === 0 ? 'diff-ok' : 'diff-warn'">
              {{ row.difference > 0 ? `+${row.difference}` : row.difference }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="110">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row.status)">
              {{ statusLabel(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="note" label="说明" min-width="280" show-overflow-tooltip />
      </el-table>
    </section>

    <section class="panel audit-section">
      <div class="audit-section-head">
        <div>
          <h2 class="audit-title">用户与权限映射</h2>
          <p class="audit-subtitle">
            HFS 用 `role` 替代 Django 细粒度权限，原权限明细已归档到 `intel_items`。
          </p>
        </div>
      </div>

      <el-table :data="summary?.users ?? []" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="username" label="用户名" min-width="150" />
        <el-table-column prop="display_name" label="显示名" min-width="150" />
        <el-table-column prop="role" label="角色" width="120" />
        <el-table-column label="启用" width="100">
          <template #default="{ row }">
            <el-tag :type="row.enabled ? 'success' : 'info'">
              {{ row.enabled ? '是' : '否' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="权限快照" width="120">
          <template #default="{ row }">
            <el-tag :type="row.permission_snapshot_imported ? 'success' : 'warning'">
              {{ row.permission_snapshot_imported ? '已导入' : '缺失' }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
    </section>

    <section class="panel audit-section">
      <div class="audit-section-head">
        <div>
          <h2 class="audit-title">迁移备注</h2>
        </div>
      </div>

      <ul class="audit-remarks">
        <li v-for="remark in summary?.remarks ?? []" :key="remark">
          {{ remark }}
        </li>
      </ul>
    </section>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import {
  fetchMigrationAuditSummary,
  type MigrationAuditSummary
} from '@/api/migration-audit'

const summary = ref<MigrationAuditSummary | null>(null)

function statusTagType(status: string) {
  if (status === 'matched') {
    return 'success'
  }
  if (status === 'merged') {
    return 'warning'
  }
  return 'danger'
}

function statusLabel(status: string) {
  if (status === 'matched') {
    return '一致'
  }
  if (status === 'merged') {
    return '合并'
  }
  return '异常'
}

async function loadSummary() {
  summary.value = await fetchMigrationAuditSummary()
}

onMounted(async () => {
  try {
    await loadSummary()
  } catch {
    ElMessage.error('迁移审计数据加载失败')
  }
})
</script>

<style scoped>
.audit-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.audit-section {
  padding: 24px;
}

.audit-section-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 18px;
}

.audit-title {
  margin: 0;
  font-size: 20px;
  color: #0f172a;
}

.audit-subtitle {
  margin: 8px 0 0;
  color: #64748b;
}

.audit-status-group {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.diff-ok {
  color: #15803d;
  font-weight: 700;
}

.diff-warn {
  color: #b45309;
  font-weight: 700;
}

.audit-remarks {
  margin: 0;
  padding-left: 18px;
  color: #334155;
  line-height: 1.7;
}
</style>
