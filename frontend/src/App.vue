<template>
  <router-view />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { fetchCurrentUser } from '@/api/auth'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

onMounted(async () => {
  if (!authStore.token || authStore.user) {
    return
  }
  try {
    const user = await fetchCurrentUser()
    authStore.setUser(user)
  } catch {
    authStore.clear()
  }
})
</script>
