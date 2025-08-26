/**
 * Remote Component Loader for Module Federation
 * Handles dynamic loading and lazy loading of micro frontend components
 */

import React, { Suspense, lazy, ComponentType, useState, useEffect } from 'react';
import { MicrofrontendConfig, RemoteComponent } from '../types';

// Dynamic import for remote components
const loadComponent = (url: string, scope: string, module: string): Promise<ComponentType<any>> => {
  return new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.src = url;
    script.onload = () => {
      // @ts-ignore - Module federation global
      const container = window[scope];
      if (!container) {
        reject(new Error(`Remote container ${scope} not found`));
        return;
      }

      container.init(__webpack_share_scopes__.default);
      container.get(module).then((factory: () => any) => {
        const Module = factory();
        resolve(Module.default || Module);
      }).catch(reject);
    };
    script.onerror = () => reject(new Error(`Failed to load remote script: ${url}`));
    
    if (!document.head.querySelector(`script[src="${url}"]`)) {
      document.head.appendChild(script);
    }
  });
};

// Component cache to avoid reloading
const componentCache = new Map<string, ComponentType<any>>();

interface RemoteComponentLoaderProps {
  config: MicrofrontendConfig;
  fallback?: React.ComponentType;
  onLoad?: (component: ComponentType<any>) => void;
  onError?: (error: Error) => void;
  props?: Record<string, any>;
}

export const RemoteComponentLoader: React.FC<RemoteComponentLoaderProps> = ({
  config,
  fallback: Fallback,
  onLoad,
  onError,
  props = {},
}) => {
  const [remoteComponent, setRemoteComponent] = useState<RemoteComponent>({
    component: null as any,
    loading: true,
    error: undefined,
  });

  const cacheKey = `${config.name}-${config.scope}-${config.module}`;

  useEffect(() => {
    const loadRemoteComponent = async () => {
      try {
        // Check cache first
        let Component = componentCache.get(cacheKey);
        
        if (!Component) {
          // Load CSS dependencies
          if (config.css) {
            config.css.forEach(cssUrl => {
              if (!document.head.querySelector(`link[href="${cssUrl}"]`)) {
                const link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = cssUrl;
                document.head.appendChild(link);
              }
            });
          }

          // Load the remote component
          if (config.type === 'systemjs') {
            // SystemJS loader (alternative to module federation)
            const System = (window as any).System;
            if (!System) {
              throw new Error('SystemJS not available');
            }
            const module = await System.import(config.url);
            Component = module[config.module] || module.default;
          } else {
            // Module federation loader
            Component = await loadComponent(config.url, config.scope, config.module);
          }

          componentCache.set(cacheKey, Component);
        }

        setRemoteComponent({
          component: Component,
          loading: false,
          error: undefined,
        });

        onLoad?.(Component);
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        setRemoteComponent({
          component: config.fallback || null,
          loading: false,
          error: err,
        });
        onError?.(err);
      }
    };

    loadRemoteComponent();
  }, [cacheKey, config, onLoad, onError]);

  if (remoteComponent.loading) {
    return Fallback ? <Fallback /> : (
      <div className="flex items-center justify-center p-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        <span className="ml-2">Loading component...</span>
      </div>
    );
  }

  if (remoteComponent.error) {
    const ErrorFallback = config.fallback || Fallback;
    if (ErrorFallback) {
      return <ErrorFallback />;
    }
    
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Failed to load component</h3>
        <p className="text-red-600 text-sm mt-1">{remoteComponent.error.message}</p>
        <details className="mt-2">
          <summary className="text-red-700 cursor-pointer">Component Details</summary>
          <pre className="text-xs mt-1 bg-red-100 p-2 rounded">
            {JSON.stringify(config, null, 2)}
          </pre>
        </details>
      </div>
    );
  }

  const Component = remoteComponent.component;
  if (!Component) {
    return null;
  }

  return (
    <Suspense fallback={Fallback ? <Fallback /> : <div>Loading...</div>}>
      <Component {...props} {...config.props} />
    </Suspense>
  );
};

// Hook for using remote components
export const useRemoteComponent = (config: MicrofrontendConfig) => {
  const [state, setState] = useState<RemoteComponent>({
    component: null as any,
    loading: true,
    error: undefined,
  });

  useEffect(() => {
    const cacheKey = `${config.name}-${config.scope}-${config.module}`;
    
    const loadComponent = async () => {
      try {
        let Component = componentCache.get(cacheKey);
        
        if (!Component) {
          if (config.type === 'systemjs') {
            const System = (window as any).System;
            if (!System) {
              throw new Error('SystemJS not available');
            }
            const module = await System.import(config.url);
            Component = module[config.module] || module.default;
          } else {
            Component = await loadComponent(config.url, config.scope, config.module);
          }
          
          componentCache.set(cacheKey, Component);
        }

        setState({
          component: Component,
          loading: false,
          error: undefined,
        });
      } catch (error) {
        setState({
          component: config.fallback || null,
          loading: false,
          error: error instanceof Error ? error : new Error(String(error)),
        });
      }
    };

    loadComponent();
  }, [config]);

  return state;
};

// Utility function to preload components
export const preloadRemoteComponent = async (config: MicrofrontendConfig): Promise<ComponentType<any>> => {
  const cacheKey = `${config.name}-${config.scope}-${config.module}`;
  
  let Component = componentCache.get(cacheKey);
  if (Component) {
    return Component;
  }

  if (config.type === 'systemjs') {
    const System = (window as any).System;
    if (!System) {
      throw new Error('SystemJS not available');
    }
    const module = await System.import(config.url);
    Component = module[config.module] || module.default;
  } else {
    Component = await loadComponent(config.url, config.scope, config.module);
  }

  componentCache.set(cacheKey, Component);
  return Component;
};

export default RemoteComponentLoader;