<template>
  <div class="login-page">
    <div class="panel login-card">
      <h1>FFS</h1>
      <p class="login-desc">家庭财务系统登录</p>
      <el-form label-position="top" @submit.prevent>
        <el-form-item label="用户名">
          <el-input v-model="username" placeholder="请输入用户名" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input
            v-model="password"
            type="password"
            show-password
            placeholder="请输入密码"
            @keyup.enter="submit"
          />
        </el-form-item>
        <el-form-item label="验证码">
          <div class="captcha-row">
            <el-input
              v-model="captchaCode"
              placeholder="输入验证码"
              @keyup.enter="submit"
            />
            <img
              v-if="captchaImage"
              :src="captchaImage"
              class="captcha-image"
              alt="captcha"
              @click="refreshCaptcha"
            />
          </div>
        </el-form-item>
        <el-button type="primary" :loading="loading" style="width: 100%;" @click="submit">
          登录
        </el-button>
      </el-form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { fetchCaptcha, login } from '@/api/auth'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const username = ref('')
const password = ref('')
const captchaId = ref('')
const captchaCode = ref('')
const captchaImage = ref('')
const loading = ref(false)

async function refreshCaptcha() {
  const data = await fetchCaptcha()
  captchaId.value = data.code_id
  captchaImage.value = data.image_base64
  captchaCode.value = ''
}

async function submit() {
  if (!captchaId.value) {
    await refreshCaptcha()
  }
  loading.value = true
  try {
    const session = await login(
      username.value,
      password.value,
      captchaId.value,
      captchaCode.value
    )
    authStore.setSession(session.token, session.user)
    await router.push('/')
    ElMessage.success('登录成功')
  } catch {
    ElMessage.error('登录失败，请检查账号密码和验证码')
    await refreshCaptcha()
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await refreshCaptcha()
})
</script>

<style scoped>
.login-page {
  display: grid;
  min-height: 100vh;
  place-items: center;
  background: linear-gradient(135deg, #f1f5f9, #e0f2fe);
}

.login-card {
  width: min(420px, 100%);
  padding: 28px;
}

.login-desc {
  margin: 0 0 24px;
  color: #64748b;
}

.captcha-row {
  display: flex;
  gap: 12px;
}

.captcha-image {
  width: 140px;
  height: 42px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  cursor: pointer;
}
</style>
