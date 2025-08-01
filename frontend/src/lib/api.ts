// API configuration and utilities
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api'

export class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`
    
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    })

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`)
    }

    return response.json()
  }

  // Problems
  async getProblems() {
    return this.request('/problems')
  }

  async getProblem(id: string) {
    return this.request(`/problems/${id}`)
  }

  // Contests
  async getContests() {
    return this.request('/contests')
  }

  async getContest(id: string) {
    return this.request(`/contests/${id}`)
  }

  // Submissions
  async getSubmissions(problemId?: string, contestId?: string) {
    const params = new URLSearchParams()
    if (problemId) params.append('problemId', problemId)
    if (contestId) params.append('contestId', contestId)
    
    const query = params.toString()
    return this.request(`/submissions${query ? `?${query}` : ''}`)
  }

  async submitSolution(problemId: string, language: string, sourceCode: string, contestId?: string) {
    return this.request('/submissions', {
      method: 'POST',
      body: JSON.stringify({
        problemId,
        language,
        sourceCode,
        contestId,
      }),
    })
  }

  // Leaderboard
  async getLeaderboard(contestId: string) {
    return this.request(`/contests/${contestId}/leaderboard`)
  }
}

export const apiClient = new ApiClient()
