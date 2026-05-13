import axios from 'axios'

export interface ApiResponse<T> {
  code: number
  message: string
  data: T
}

export interface SessionUser {
  id: number
  username: string
  display_name: string
  role: string
}

const USER_STORAGE_KEY = 'ffs_user'

export const client = axios.create({
  baseURL: '/api',
  timeout: 10000
})

client.interceptors.request.use((config) => {
  const token = localStorage.getItem('ffs_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

export function getStoredUser() {
  const raw = localStorage.getItem(USER_STORAGE_KEY)
  if (!raw) {
    return null
  }
  try {
    return JSON.parse(raw) as SessionUser
  } catch {
    localStorage.removeItem(USER_STORAGE_KEY)
    return null
  }
}

export function setStoredUser(user: SessionUser) {
  localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(user))
}

export function clearStoredUser() {
  localStorage.removeItem(USER_STORAGE_KEY)
}

export function requireCurrentUserId() {
  const user = getStoredUser()
  if (!user?.id) {
    throw new Error('current user is not available')
  }
  return user.id
}

export function withCurrentUserId<T extends Record<string, unknown>>(params: T) {
  return {
    ...params,
    user_id: requireCurrentUserId()
  }
}
