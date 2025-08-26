/**
 * Core TypeScript types for Judicia Platform frontend development
 */

import { ReactNode, ComponentType } from 'react';

// ============================================================================
// Platform Core Types
// ============================================================================

export interface JudiciaUser {
  id: string;
  username: string;
  email: string;
  displayName: string;
  roles: string[];
  permissions: string[];
  avatar?: string;
  preferences: Record<string, any>;
  lastActive: Date;
}

export interface JudiciaPlatformContext {
  user: JudiciaUser | null;
  theme: 'light' | 'dark' | 'auto';
  locale: string;
  timezone: string;
  contestId?: string;
  problemId?: string;
  submissionId?: string;
  isAdmin: boolean;
  isModerator: boolean;
  features: string[];
}

// ============================================================================
// Plugin System Types
// ============================================================================

export interface PluginMetadata {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  homepage?: string;
  repository?: string;
  license?: string;
  dependencies: string[];
  capabilities: PluginCapability[];
  components: ComponentRegistration[];
  routes: RouteRegistration[];
}

export enum PluginCapability {
  // Data access
  READ_PROBLEMS = 'read_problems',
  WRITE_PROBLEMS = 'write_problems',
  READ_SUBMISSIONS = 'read_submissions',
  WRITE_SUBMISSIONS = 'write_submissions',
  READ_CONTESTS = 'read_contests',
  WRITE_CONTESTS = 'write_contests',
  READ_USERS = 'read_users',
  WRITE_USERS = 'write_users',
  
  // UI capabilities
  REGISTER_COMPONENTS = 'register_components',
  REGISTER_ROUTES = 'register_routes',
  EMIT_EVENTS = 'emit_events',
  SUBSCRIBE_EVENTS = 'subscribe_events',
  
  // Platform integration
  NOTIFICATIONS = 'notifications',
  FILE_STORAGE = 'file_storage',
  EXTERNAL_API = 'external_api',
  ADMIN_OPERATIONS = 'admin_operations',
}

// ============================================================================
// Component System Types
// ============================================================================

export interface ComponentRegistration {
  name: string;
  component: ComponentType<any>;
  props?: ComponentPropSchema;
  slots?: string[];
  events?: ComponentEvent[];
  styles?: ComponentStyles;
  lazy?: boolean;
}

export interface ComponentPropSchema {
  [key: string]: {
    type: 'string' | 'number' | 'boolean' | 'object' | 'array';
    required?: boolean;
    default?: any;
    description?: string;
    validation?: (value: any) => boolean;
  };
}

export interface ComponentEvent {
  name: string;
  description?: string;
  payload?: ComponentPropSchema;
}

export interface ComponentStyles {
  css?: string;
  classNames?: Record<string, string>;
  cssVariables?: Record<string, string>;
}

// ============================================================================
// Event System Types
// ============================================================================

export interface PlatformEvent<T = any> {
  id: string;
  type: string;
  source: string;
  timestamp: Date;
  payload: T;
  metadata?: Record<string, any>;
}

export type EventHandler<T = any> = (event: PlatformEvent<T>) => void | Promise<void>;

export interface EventEmitter {
  on<T = any>(eventType: string, handler: EventHandler<T>): void;
  off<T = any>(eventType: string, handler: EventHandler<T>): void;
  emit<T = any>(eventType: string, payload: T, metadata?: Record<string, any>): void;
  once<T = any>(eventType: string, handler: EventHandler<T>): void;
}

// Standard platform events
export interface PlatformEvents {
  'user.login': { user: JudiciaUser };
  'user.logout': { userId: string };
  'contest.start': { contestId: string; contest: any };
  'contest.end': { contestId: string; contest: any };
  'submission.created': { submissionId: string; submission: any };
  'submission.judged': { submissionId: string; result: any };
  'problem.opened': { problemId: string; problem: any };
  'theme.changed': { theme: string };
  'locale.changed': { locale: string };
  'route.changed': { path: string; params: Record<string, string> };
}

// ============================================================================
// State Management Types
// ============================================================================

export interface GlobalState {
  platform: JudiciaPlatformContext;
  plugins: Record<string, PluginState>;
  ui: UIState;
  cache: CacheState;
}

export interface PluginState {
  id: string;
  loaded: boolean;
  error?: string;
  data: Record<string, any>;
}

export interface UIState {
  sidebarCollapsed: boolean;
  modalStack: string[];
  notifications: Notification[];
  loading: Record<string, boolean>;
}

export interface CacheState {
  problems: Record<string, any>;
  submissions: Record<string, any>;
  contests: Record<string, any>;
  users: Record<string, any>;
}

// ============================================================================
// Routing Types
// ============================================================================

export interface RouteRegistration {
  path: string;
  component: ComponentType<any>;
  title?: string;
  icon?: string;
  requiresAuth?: boolean;
  requiredCapabilities?: PluginCapability[];
  exact?: boolean;
  children?: RouteRegistration[];
}

export interface NavigationItem {
  id: string;
  label: string;
  path: string;
  icon?: string;
  badge?: string | number;
  children?: NavigationItem[];
  capabilities?: PluginCapability[];
}

// ============================================================================
// Theme System Types
// ============================================================================

export interface ThemeDefinition {
  name: string;
  displayName: string;
  colors: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    error: string;
    background: string;
    surface: string;
    text: string;
    textSecondary: string;
    border: string;
    [key: string]: string;
  };
  typography: {
    fontFamily: string;
    fontSize: Record<string, string>;
    fontWeight: Record<string, string>;
    lineHeight: Record<string, string>;
  };
  spacing: Record<string, string>;
  borderRadius: Record<string, string>;
  shadows: Record<string, string>;
  breakpoints: Record<string, string>;
  cssVariables: Record<string, string>;
}

// ============================================================================
// API Types
// ============================================================================

export interface APIResponse<T = any> {
  data: T;
  success: boolean;
  message?: string;
  pagination?: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
  metadata?: Record<string, any>;
}

export interface APIError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: Date;
}

// ============================================================================
// Notification System Types
// ============================================================================

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message?: string;
  duration?: number;
  persistent?: boolean;
  actions?: NotificationAction[];
  timestamp: Date;
}

export interface NotificationAction {
  label: string;
  action: string | (() => void);
  style?: 'primary' | 'secondary' | 'danger';
}

// ============================================================================
// Form System Types
// ============================================================================

export interface FormField {
  name: string;
  type: 'text' | 'email' | 'password' | 'number' | 'select' | 'textarea' | 'checkbox' | 'radio' | 'file' | 'date' | 'datetime';
  label: string;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  readonly?: boolean;
  options?: FormSelectOption[];
  validation?: FormFieldValidation;
  description?: string;
  defaultValue?: any;
}

export interface FormSelectOption {
  value: string | number;
  label: string;
  disabled?: boolean;
}

export interface FormFieldValidation {
  pattern?: RegExp;
  minLength?: number;
  maxLength?: number;
  min?: number;
  max?: number;
  custom?: (value: any) => string | null;
}

// ============================================================================
// Micro Frontend Types
// ============================================================================

export interface MicrofrontendConfig {
  name: string;
  url: string;
  scope: string;
  module: string;
  type: 'module' | 'systemjs';
  fallback?: ComponentType;
  props?: Record<string, any>;
  css?: string[];
}

export interface RemoteComponent {
  component: ComponentType<any>;
  loading: boolean;
  error?: Error;
}

// ============================================================================
// Utility Types
// ============================================================================

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>;

export type EventMap = Record<string, any>;

export type ComponentProps<T extends ComponentType<any>> = T extends ComponentType<infer P> ? P : never;