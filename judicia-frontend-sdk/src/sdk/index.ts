/**
 * Main SDK class for Judicia Platform frontend integration
 */

import { EventEmitter } from 'eventemitter3';
import { 
  PluginMetadata, 
  JudiciaPlatformContext, 
  ComponentRegistration, 
  RouteRegistration,
  PlatformEvent,
  ThemeDefinition,
  MicrofrontendConfig,
  RemoteComponent
} from '../types';
import { createStore } from '../store';
import { ThemeManager } from '../theme';
import { APIClient } from '../platform/api';
import { ComponentRegistry } from '../components/registry';
import { Router } from '../routing';

export interface SDKConfig {
  apiBaseUrl?: string;
  enableDevTools?: boolean;
  theme?: ThemeDefinition;
  plugins?: string[];
  microfrontends?: MicrofrontendConfig[];
}

export class JudiciaFrontendSDK extends EventEmitter {
  private static instance: JudiciaFrontendSDK | null = null;
  
  public readonly version = '1.0.0';
  public readonly store = createStore();
  public readonly theme = new ThemeManager();
  public readonly api: APIClient;
  public readonly components = new ComponentRegistry();
  public readonly router = new Router();
  
  private config: SDKConfig;
  private initialized = false;
  private plugins = new Map<string, PluginMetadata>();
  private microfrontends = new Map<string, RemoteComponent>();

  constructor(config: SDKConfig = {}) {
    super();
    
    if (JudiciaFrontendSDK.instance) {
      throw new Error('JudiciaFrontendSDK is a singleton. Use JudiciaFrontendSDK.getInstance() instead.');
    }
    
    this.config = {
      apiBaseUrl: '/api/v1',
      enableDevTools: process.env.NODE_ENV === 'development',
      ...config
    };
    
    this.api = new APIClient({
      baseURL: this.config.apiBaseUrl!,
      timeout: 30000,
    });
    
    JudiciaFrontendSDK.instance = this;
  }

  static getInstance(config?: SDKConfig): JudiciaFrontendSDK {
    if (!JudiciaFrontendSDK.instance) {
      JudiciaFrontendSDK.instance = new JudiciaFrontendSDK(config);
    }
    return JudiciaFrontendSDK.instance;
  }

  /**
   * Initialize the SDK with platform context
   */
  async initialize(context: JudiciaPlatformContext): Promise<void> {
    if (this.initialized) {
      console.warn('JudiciaFrontendSDK is already initialized');
      return;
    }

    try {
      // Initialize store with platform context
      this.store.setState(state => ({
        ...state,
        platform: context
      }));

      // Initialize theme
      if (this.config.theme) {
        this.theme.setTheme(this.config.theme);
      }
      this.theme.setMode(context.theme);

      // Load configured plugins
      if (this.config.plugins?.length) {
        await Promise.all(
          this.config.plugins.map(pluginId => this.loadPlugin(pluginId))
        );
      }

      // Load configured microfrontends
      if (this.config.microfrontends?.length) {
        await Promise.all(
          this.config.microfrontends.map(config => this.loadMicrofrontend(config))
        );
      }

      // Set up event listeners
      this.setupEventListeners();

      this.initialized = true;
      this.emit('sdk:initialized', { context });

      if (this.config.enableDevTools) {
        this.setupDevTools();
      }

    } catch (error) {
      this.emit('sdk:initialization-failed', { error });
      throw error;
    }
  }

  /**
   * Load a plugin by ID
   */
  async loadPlugin(pluginId: string): Promise<PluginMetadata> {
    try {
      // Check if plugin is already loaded
      if (this.plugins.has(pluginId)) {
        return this.plugins.get(pluginId)!;
      }

      this.emit('plugin:loading', { pluginId });

      // Fetch plugin metadata and code from platform
      const response = await this.api.get(`/plugins/${pluginId}/metadata`);
      const metadata: PluginMetadata = response.data;

      // Load plugin components
      if (metadata.components?.length) {
        for (const componentReg of metadata.components) {
          await this.registerComponent(componentReg);
        }
      }

      // Register plugin routes
      if (metadata.routes?.length) {
        for (const route of metadata.routes) {
          this.router.addRoute(route);
        }
      }

      this.plugins.set(pluginId, metadata);
      
      this.store.setState(state => ({
        ...state,
        plugins: {
          ...state.plugins,
          [pluginId]: {
            id: pluginId,
            loaded: true,
            data: {}
          }
        }
      }));

      this.emit('plugin:loaded', { pluginId, metadata });
      return metadata;

    } catch (error) {
      this.store.setState(state => ({
        ...state,
        plugins: {
          ...state.plugins,
          [pluginId]: {
            id: pluginId,
            loaded: false,
            error: error instanceof Error ? error.message : 'Unknown error',
            data: {}
          }
        }
      }));

      this.emit('plugin:load-failed', { pluginId, error });
      throw error;
    }
  }

  /**
   * Load a microfrontend configuration
   */
  async loadMicrofrontend(config: MicrofrontendConfig): Promise<RemoteComponent> {
    try {
      this.emit('microfrontend:loading', { name: config.name });

      let component: any;

      if (config.type === 'module') {
        // Dynamic import for ES modules
        const module = await import(/* webpackIgnore: true */ config.url);
        component = module[config.module] || module.default;
      } else if (config.type === 'systemjs') {
        // SystemJS loading
        if (typeof (window as any).System !== 'undefined') {
          const module = await (window as any).System.import(config.url);
          component = module[config.module] || module.default;
        } else {
          throw new Error('SystemJS is not available');
        }
      }

      const remoteComponent: RemoteComponent = {
        component,
        loading: false,
        error: undefined
      };

      this.microfrontends.set(config.name, remoteComponent);
      this.emit('microfrontend:loaded', { name: config.name, config });

      return remoteComponent;

    } catch (error) {
      const remoteComponent: RemoteComponent = {
        component: config.fallback || (() => null),
        loading: false,
        error: error instanceof Error ? error : new Error('Unknown error')
      };

      this.microfrontends.set(config.name, remoteComponent);
      this.emit('microfrontend:load-failed', { name: config.name, error });

      return remoteComponent;
    }
  }

  /**
   * Register a component in the global registry
   */
  async registerComponent(registration: ComponentRegistration): Promise<void> {
    this.components.register(registration);
    this.emit('component:registered', { registration });
  }

  /**
   * Get a registered component by name
   */
  getComponent(name: string): ComponentRegistration | null {
    return this.components.get(name);
  }

  /**
   * Get a microfrontend by name
   */
  getMicrofrontend(name: string): RemoteComponent | null {
    return this.microfrontends.get(name) || null;
  }

  /**
   * Emit a platform event
   */
  emitPlatformEvent<T = any>(type: string, payload: T, metadata?: Record<string, any>): void {
    const event: PlatformEvent<T> = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      source: 'sdk',
      timestamp: new Date(),
      payload,
      metadata
    };

    this.emit('platform:event', event);
    this.emit(`platform:event:${type}`, event);
  }

  /**
   * Subscribe to platform events
   */
  onPlatformEvent<T = any>(type: string, handler: (event: PlatformEvent<T>) => void): () => void {
    this.on(`platform:event:${type}`, handler);
    
    // Return unsubscribe function
    return () => {
      this.off(`platform:event:${type}`, handler);
    };
  }

  /**
   * Get current platform context
   */
  getContext(): JudiciaPlatformContext | null {
    return this.store.getState().platform;
  }

  /**
   * Update platform context
   */
  updateContext(updates: Partial<JudiciaPlatformContext>): void {
    this.store.setState(state => ({
      ...state,
      platform: {
        ...state.platform,
        ...updates
      }
    }));

    this.emit('context:updated', updates);
  }

  /**
   * Destroy the SDK instance
   */
  destroy(): void {
    this.removeAllListeners();
    this.components.clear();
    this.microfrontends.clear();
    this.plugins.clear();
    this.initialized = false;
    JudiciaFrontendSDK.instance = null;
  }

  private setupEventListeners(): void {
    // Listen for theme changes
    this.store.subscribe(
      state => state.platform.theme,
      (theme) => {
        this.theme.setMode(theme);
        this.emit('theme:changed', { theme });
      }
    );

    // Listen for user changes
    this.store.subscribe(
      state => state.platform.user,
      (user) => {
        this.emit('user:changed', { user });
      }
    );
  }

  private setupDevTools(): void {
    if (typeof window !== 'undefined') {
      (window as any).__JUDICIA_SDK__ = {
        version: this.version,
        instance: this,
        store: this.store,
        components: this.components,
        plugins: Array.from(this.plugins.entries()),
        microfrontends: Array.from(this.microfrontends.entries()),
        theme: this.theme,
        api: this.api
      };

      console.log(`🎯 Judicia Frontend SDK v${this.version} initialized`);
      console.log('DevTools available at window.__JUDICIA_SDK__');
    }
  }
}

// Default export
export default JudiciaFrontendSDK;