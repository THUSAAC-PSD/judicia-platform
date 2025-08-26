/**
 * Platform API client
 */

import { APIResponse, APIError } from '../types';

export interface APIClientConfig {
  baseURL: string;
  timeout?: number;
  defaultHeaders?: Record<string, string>;
}

export class APIClient {
  private baseURL: string;
  private timeout: number;
  private defaultHeaders: Record<string, string>;

  constructor(config: APIClientConfig) {
    this.baseURL = config.baseURL.replace(/\/$/, '');
    this.timeout = config.timeout || 30000;
    this.defaultHeaders = {
      'Content-Type': 'application/json',
      ...config.defaultHeaders,
    };
  }

  async request<T = any>(
    method: string,
    path: string,
    data?: any,
    options: RequestInit = {}
  ): Promise<APIResponse<T>> {
    const url = `${this.baseURL}${path}`;
    const controller = new AbortController();
    
    const timeoutId = setTimeout(() => {
      controller.abort();
    }, this.timeout);

    try {
      const response = await fetch(url, {
        method: method.toUpperCase(),
        headers: {
          ...this.defaultHeaders,
          ...options.headers,
        },
        body: data ? JSON.stringify(data) : undefined,
        signal: controller.signal,
        ...options,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error: APIError = {
          code: response.status.toString(),
          message: response.statusText,
          timestamp: new Date(),
        };

        try {
          const errorData = await response.json();
          error.message = errorData.message || error.message;
          error.details = errorData.details;
        } catch {
          // Ignore JSON parsing errors for error responses
        }

        throw error;
      }

      const result: APIResponse<T> = await response.json();
      return result;

    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new Error('Request timeout');
        }
        throw error;
      }

      throw new Error('Unknown error occurred');
    }
  }

  async get<T = any>(path: string, options?: RequestInit): Promise<APIResponse<T>> {
    return this.request<T>('GET', path, undefined, options);
  }

  async post<T = any>(path: string, data?: any, options?: RequestInit): Promise<APIResponse<T>> {
    return this.request<T>('POST', path, data, options);
  }

  async put<T = any>(path: string, data?: any, options?: RequestInit): Promise<APIResponse<T>> {
    return this.request<T>('PUT', path, data, options);
  }

  async patch<T = any>(path: string, data?: any, options?: RequestInit): Promise<APIResponse<T>> {
    return this.request<T>('PATCH', path, data, options);
  }

  async delete<T = any>(path: string, options?: RequestInit): Promise<APIResponse<T>> {
    return this.request<T>('DELETE', path, undefined, options);
  }

  setAuthToken(token: string): void {
    this.defaultHeaders.Authorization = `Bearer ${token}`;
  }

  removeAuthToken(): void {
    delete this.defaultHeaders.Authorization;
  }

  setHeader(key: string, value: string): void {
    this.defaultHeaders[key] = value;
  }

  removeHeader(key: string): void {
    delete this.defaultHeaders[key];
  }
}