/**
 * React hooks for Judicia Platform integration
 */

import { useEffect, useState, useCallback } from 'react';
import { useJudiciaStore } from '../store';
import { JudiciaFrontendSDK } from '../sdk';
import { JudiciaPlatformContext, PlatformEvent, ComponentRegistration } from '../types';

/**
 * Hook to access the Judicia SDK instance
 */
export function useJudicia(): JudiciaFrontendSDK {
  return JudiciaFrontendSDK.getInstance();
}

/**
 * Hook to access platform context
 */
export function usePlatformContext(): JudiciaPlatformContext {
  return useJudiciaStore(state => state.platform);
}

/**
 * Hook to access current user
 */
export function useUser() {
  return useJudiciaStore(state => state.platform.user);
}

/**
 * Hook to check user capabilities
 */
export function useCapabilities() {
  const user = useUser();
  
  return {
    permissions: user?.permissions || [],
    roles: user?.roles || [],
    hasPermission: useCallback((permission: string) => {
      return user?.permissions?.includes(permission) || false;
    }, [user?.permissions]),
    hasRole: useCallback((role: string) => {
      return user?.roles?.includes(role) || false;
    }, [user?.roles]),
    hasAnyPermission: useCallback((permissions: string[]) => {
      return permissions.some(permission => user?.permissions?.includes(permission));
    }, [user?.permissions]),
    hasAllPermissions: useCallback((permissions: string[]) => {
      return permissions.every(permission => user?.permissions?.includes(permission));
    }, [user?.permissions]),
  };
}

/**
 * Hook to manage theme
 */
export function useTheme() {
  const theme = useJudiciaStore(state => state.platform.theme);
  const sdk = useJudicia();
  
  return {
    theme,
    setTheme: useCallback((newTheme: 'light' | 'dark' | 'auto') => {
      sdk.updateContext({ theme: newTheme });
    }, [sdk]),
    isDark: theme === 'dark' || (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches),
  };
}

/**
 * Hook to subscribe to platform events
 */
export function usePlatformEvent<T = any>(
  eventType: string,
  handler: (event: PlatformEvent<T>) => void,
  dependencies: any[] = []
) {
  const sdk = useJudicia();
  
  useEffect(() => {
    return sdk.onPlatformEvent(eventType, handler);
  }, [sdk, eventType, ...dependencies]);
}

/**
 * Hook to emit platform events
 */
export function useEventEmitter() {
  const sdk = useJudicia();
  
  return useCallback(<T = any>(type: string, payload: T, metadata?: Record<string, any>) => {
    sdk.emitPlatformEvent(type, payload, metadata);
  }, [sdk]);
}

/**
 * Hook to manage notifications
 */
export function useNotifications() {
  const notifications = useJudiciaStore(state => state.ui.notifications);
  const store = useJudiciaStore();
  
  return {
    notifications,
    addNotification: useCallback((notification: any) => {
      const id = `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      const fullNotification = {
        id,
        timestamp: new Date(),
        ...notification,
      };
      
      store.setState(state => ({
        ...state,
        ui: {
          ...state.ui,
          notifications: [...state.ui.notifications, fullNotification],
        },
      }));
      
      // Auto-remove non-persistent notifications
      if (!notification.persistent && notification.duration !== 0) {
        const duration = notification.duration || 5000;
        setTimeout(() => {
          store.setState(state => ({
            ...state,
            ui: {
              ...state.ui,
              notifications: state.ui.notifications.filter(n => n.id !== id),
            },
          }));
        }, duration);
      }
      
      return id;
    }, [store]),
    removeNotification: useCallback((id: string) => {
      store.setState(state => ({
        ...state,
        ui: {
          ...state.ui,
          notifications: state.ui.notifications.filter(n => n.id !== id),
        },
      }));
    }, [store]),
    clearNotifications: useCallback(() => {
      store.setState(state => ({
        ...state,
        ui: {
          ...state.ui,
          notifications: [],
        },
      }));
    }, [store]),
  };
}

/**
 * Hook to manage loading states
 */
export function useLoading(key?: string) {
  const loading = useJudiciaStore(state => state.ui.loading);
  const store = useJudiciaStore();
  
  return {
    loading: key ? loading[key] || false : loading,
    setLoading: useCallback((keyOrState: string | boolean, state?: boolean) => {
      if (typeof keyOrState === 'boolean') {
        // Single loading state
        store.setState(prevState => ({
          ...prevState,
          ui: {
            ...prevState.ui,
            loading: { ...prevState.ui.loading, default: keyOrState },
          },
        }));
      } else {
        // Named loading state
        store.setState(prevState => ({
          ...prevState,
          ui: {
            ...prevState.ui,
            loading: { ...prevState.ui.loading, [keyOrState]: state || false },
          },
        }));
      }
    }, [store]),
  };
}

/**
 * Hook to dynamically load and use a component
 */
export function useDynamicComponent(name: string): {
  component: ComponentRegistration | null;
  loading: boolean;
  error: Error | null;
} {
  const [state, setState] = useState({
    component: null as ComponentRegistration | null,
    loading: true,
    error: null as Error | null,
  });
  
  const sdk = useJudicia();
  
  useEffect(() => {
    const loadComponent = async () => {
      try {
        setState(prev => ({ ...prev, loading: true, error: null }));
        
        // First check if component is already registered
        let component = sdk.getComponent(name);
        
        if (!component) {
          // Try to load it dynamically (this would typically involve loading a plugin)
          // For now, we'll just check again after a short delay
          await new Promise(resolve => setTimeout(resolve, 100));
          component = sdk.getComponent(name);
        }
        
        setState({
          component,
          loading: false,
          error: component ? null : new Error(`Component '${name}' not found`),
        });
        
      } catch (error) {
        setState({
          component: null,
          loading: false,
          error: error instanceof Error ? error : new Error('Unknown error'),
        });
      }
    };
    
    loadComponent();
  }, [name, sdk]);
  
  return state;
}

/**
 * Hook for API calls with automatic loading state management
 */
export function useAPI() {
  const sdk = useJudicia();
  const { setLoading } = useLoading();
  
  return {
    get: useCallback(async <T = any>(path: string, loadingKey?: string) => {
      try {
        if (loadingKey) setLoading(loadingKey, true);
        return await sdk.api.get<T>(path);
      } finally {
        if (loadingKey) setLoading(loadingKey, false);
      }
    }, [sdk.api, setLoading]),
    
    post: useCallback(async <T = any>(path: string, data?: any, loadingKey?: string) => {
      try {
        if (loadingKey) setLoading(loadingKey, true);
        return await sdk.api.post<T>(path, data);
      } finally {
        if (loadingKey) setLoading(loadingKey, false);
      }
    }, [sdk.api, setLoading]),
    
    put: useCallback(async <T = any>(path: string, data?: any, loadingKey?: string) => {
      try {
        if (loadingKey) setLoading(loadingKey, true);
        return await sdk.api.put<T>(path, data);
      } finally {
        if (loadingKey) setLoading(loadingKey, false);
      }
    }, [sdk.api, setLoading]),
    
    delete: useCallback(async <T = any>(path: string, loadingKey?: string) => {
      try {
        if (loadingKey) setLoading(loadingKey, true);
        return await sdk.api.delete<T>(path);
      } finally {
        if (loadingKey) setLoading(loadingKey, false);
      }
    }, [sdk.api, setLoading]),
  };
}

/**
 * Hook for persisting data to local storage with the store
 */
export function usePersistedState<T>(key: string, defaultValue: T): [T, (value: T) => void] {
  const [state, setState] = useState<T>(() => {
    try {
      const item = window.localStorage.getItem(`judicia:${key}`);
      return item ? JSON.parse(item) : defaultValue;
    } catch {
      return defaultValue;
    }
  });
  
  const setValue = useCallback((value: T) => {
    try {
      setState(value);
      window.localStorage.setItem(`judicia:${key}`, JSON.stringify(value));
    } catch (error) {
      console.warn(`Failed to persist state for key ${key}:`, error);
    }
  }, [key]);
  
  return [state, setValue];
}