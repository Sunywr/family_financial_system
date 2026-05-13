import { defineStore } from 'pinia'
import type { CurrentUser } from '@/api/auth'
import { clearStoredUser, getStoredUser, setStoredUser } from '@/api/client'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    token: localStorage.getItem('ffs_token') || '',
    user: getStoredUser() as CurrentUser | null
  }),
  actions: {
    setSession(token: string, user: CurrentUser) {
      this.token = token
      this.user = user
      localStorage.setItem('ffs_token', token)
      setStoredUser(user)
    },
    setUser(user: CurrentUser) {
      this.user = user
      setStoredUser(user)
    },
    clear() {
      this.token = ''
      this.user = null
      localStorage.removeItem('ffs_token')
      clearStoredUser()
    }
  }
})
