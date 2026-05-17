import { client, type ApiResponse } from './client'

export interface InstitutionProvider {
  name: string
  source_module: string
  source_method: string
}

export interface InstitutionCategory {
  code: string
  name: string
  investment_type: string
  providers: InstitutionProvider[]
}

export async function fetchInstitutionCategories(investmentType?: 'stock' | 'wealth') {
  const response = await client.get<ApiResponse<InstitutionCategory[]>>('/config/institutions', {
    params: { investment_type: investmentType }
  })
  return response.data.data
}
