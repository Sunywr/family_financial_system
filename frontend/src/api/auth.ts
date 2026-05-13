import { client, type ApiResponse } from './client'

export interface CurrentUser {
  id: number
  username: string
  display_name: string
  role: string
}

export interface LoginResponse {
  token: string
  user: CurrentUser
}

export interface CaptchaResponse {
  code_id: string
  image_base64: string
}

export async function fetchCaptcha() {
  const response = await client.get<ApiResponse<CaptchaResponse>>('/auth/captcha')
  return response.data.data
}

export async function login(
  username: string,
  password: string,
  captcha_id: string,
  captcha_code: string
) {
  const response = await client.post<ApiResponse<LoginResponse>>('/auth/login', {
    username,
    password,
    captcha_id,
    captcha_code
  })
  return response.data.data
}

export async function fetchCurrentUser() {
  const response = await client.get<ApiResponse<CurrentUser>>('/auth/me')
  return response.data.data
}

export async function logout() {
  const response = await client.post<ApiResponse<{ logged_out: boolean }>>('/auth/logout')
  return response.data.data
}
