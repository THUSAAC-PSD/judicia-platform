/**
 * Plugin Registry for Micro Frontend Management
 * Handles plugin registration, discovery, and lifecycle management
 */

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { MicrofrontendConfig, PluginMetadata, ComponentRegistration, RouteRegistration } from '../types';
import RemoteComponentLoader, { preloadRemoteComponent } from './RemoteComponentLoader';

// Plugin Registry State
interface PluginRegistryState {
  plugins: Map<string, PluginMetadata>;
  microfrontends: Map<string, MicrofrontendConfig>;
  components: Map<string, ComponentRegistration>;
  routes: Map<string, RouteRegistration>;
  loading: Set<string>;
  errors: Map<string, Error>;
}

// Plugin Registry Actions
interface PluginRegistryActions {
  registerPlugin: (plugin: PluginMetadata, microfrontend?: MicrofrontendConfig) => Promise<void>;
  unregisterPlugin: (pluginId: string) => void;
  getPlugin: (pluginId: string) => PluginMetadata | undefined;
  getMicrofrontend: (pluginId: string) => MicrofrontendConfig | undefined;
  getComponent: (componentName: string) => ComponentRegistration | undefined;
  getRoute: (routePath: string) => RouteRegistration | undefined;
  getAllPlugins: () => PluginMetadata[];
  getAllComponents: () => ComponentRegistration[];
  getAllRoutes: () => RouteRegistration[];
  preloadPlugin: (pluginId: string) => Promise<void>;
  isPluginLoaded: (pluginId: string) => boolean;
  getPluginError: (pluginId: string) => Error | undefined;
}

// Context
const PluginRegistryContext = createContext<PluginRegistryState & PluginRegistryActions | null>(null);

// Provider Props
interface PluginRegistryProviderProps {
  children: ReactNode;
  initialPlugins?: PluginMetadata[];
  autoDiscovery?: boolean;
  discoveryEndpoint?: string;
}

export const PluginRegistryProvider: React.FC<PluginRegistryProviderProps> = ({
  children,
  initialPlugins = [],
  autoDiscovery = true,
  discoveryEndpoint = '/api/plugins/discovery',
}) => {
  const [state, setState] = useState<PluginRegistryState>({
    plugins: new Map(),
    microfrontends: new Map(),
    components: new Map(),
    routes: new Map(),
    loading: new Set(),
    errors: new Map(),
  });

  // Plugin auto-discovery
  useEffect(() => {
    if (autoDiscovery) {
      discoverPlugins();
    }
    
    // Load initial plugins
    initialPlugins.forEach(plugin => {
      registerPlugin(plugin);
    });
  }, [autoDiscovery, discoveryEndpoint]);

  const discoverPlugins = async () => {
    try {
      const response = await fetch(discoveryEndpoint);
      if (!response.ok) {
        throw new Error(`Plugin discovery failed: ${response.statusText}`);
      }
      
      const discovered = await response.json();
      const plugins: PluginMetadata[] = discovered.plugins || [];
      const microfrontends: Record<string, MicrofrontendConfig> = discovered.microfrontends || {};
      
      for (const plugin of plugins) {
        const microfrontend = microfrontends[plugin.id];
        await registerPlugin(plugin, microfrontend);
      }
    } catch (error) {
      console.error('Plugin auto-discovery failed:', error);
    }
  };

  const registerPlugin = async (plugin: PluginMetadata, microfrontend?: MicrofrontendConfig) => {
    setState(prev => ({
      ...prev,
      loading: new Set([...prev.loading, plugin.id]),
      errors: new Map([...prev.errors].filter(([key]) => key !== plugin.id)),
    }));

    try {
      // Register plugin metadata
      const newPlugins = new Map(state.plugins);
      newPlugins.set(plugin.id, plugin);

      // Register microfrontend config if provided
      const newMicrofrontends = new Map(state.microfrontends);
      if (microfrontend) {
        newMicrofrontends.set(plugin.id, microfrontend);
      }

      // Register components
      const newComponents = new Map(state.components);
      plugin.components.forEach(comp => {
        newComponents.set(comp.name, comp);
      });

      // Register routes
      const newRoutes = new Map(state.routes);
      plugin.routes.forEach(route => {
        newRoutes.set(route.path, route);
      });

      setState(prev => ({
        ...prev,
        plugins: newPlugins,
        microfrontends: newMicrofrontends,
        components: newComponents,
        routes: newRoutes,
        loading: new Set([...prev.loading].filter(id => id !== plugin.id)),
      }));

      // Preload critical components
      if (microfrontend) {
        await preloadRemoteComponent(microfrontend);
      }

      console.log(`Plugin ${plugin.id} registered successfully`);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      setState(prev => ({
        ...prev,
        loading: new Set([...prev.loading].filter(id => id !== plugin.id)),
        errors: new Map([...prev.errors, [plugin.id, err]]),
      }));
      console.error(`Failed to register plugin ${plugin.id}:`, error);
    }
  };

  const unregisterPlugin = (pluginId: string) => {
    setState(prev => {
      const plugin = prev.plugins.get(pluginId);
      if (!plugin) return prev;

      // Remove plugin
      const newPlugins = new Map(prev.plugins);
      newPlugins.delete(pluginId);

      // Remove microfrontend
      const newMicrofrontends = new Map(prev.microfrontends);
      newMicrofrontends.delete(pluginId);

      // Remove components
      const newComponents = new Map(prev.components);
      plugin.components.forEach(comp => {
        newComponents.delete(comp.name);
      });

      // Remove routes
      const newRoutes = new Map(prev.routes);
      plugin.routes.forEach(route => {
        newRoutes.delete(route.path);
      });

      // Clear loading and errors
      const newLoading = new Set(prev.loading);
      newLoading.delete(pluginId);
      const newErrors = new Map(prev.errors);
      newErrors.delete(pluginId);

      return {
        ...prev,
        plugins: newPlugins,
        microfrontends: newMicrofrontends,
        components: newComponents,
        routes: newRoutes,
        loading: newLoading,
        errors: newErrors,
      };
    });
  };

  const getPlugin = (pluginId: string) => state.plugins.get(pluginId);
  const getMicrofrontend = (pluginId: string) => state.microfrontends.get(pluginId);
  const getComponent = (componentName: string) => state.components.get(componentName);
  const getRoute = (routePath: string) => state.routes.get(routePath);
  const getAllPlugins = () => Array.from(state.plugins.values());
  const getAllComponents = () => Array.from(state.components.values());
  const getAllRoutes = () => Array.from(state.routes.values());
  const isPluginLoaded = (pluginId: string) => state.plugins.has(pluginId) && !state.loading.has(pluginId);
  const getPluginError = (pluginId: string) => state.errors.get(pluginId);

  const preloadPlugin = async (pluginId: string) => {
    const microfrontend = state.microfrontends.get(pluginId);
    if (microfrontend) {
      await preloadRemoteComponent(microfrontend);
    }
  };

  const value = {
    ...state,
    registerPlugin,
    unregisterPlugin,
    getPlugin,
    getMicrofrontend,
    getComponent,
    getRoute,
    getAllPlugins,
    getAllComponents,
    getAllRoutes,
    preloadPlugin,
    isPluginLoaded,
    getPluginError,
  };

  return (
    <PluginRegistryContext.Provider value={value}>
      {children}
    </PluginRegistryContext.Provider>
  );
};

// Hook to use the plugin registry
export const usePluginRegistry = () => {
  const context = useContext(PluginRegistryContext);
  if (!context) {
    throw new Error('usePluginRegistry must be used within a PluginRegistryProvider');
  }
  return context;
};

// Dynamic Component - renders plugin components by name
interface DynamicComponentProps {
  name: string;
  pluginId?: string;
  props?: Record<string, any>;
  fallback?: React.ComponentType;
  onError?: (error: Error) => void;
}

export const DynamicComponent: React.FC<DynamicComponentProps> = ({
  name,
  pluginId,
  props = {},
  fallback,
  onError,
}) => {
  const registry = usePluginRegistry();
  const component = registry.getComponent(name);
  
  if (!component) {
    const error = new Error(`Component '${name}' not found in registry`);
    onError?.(error);
    
    if (fallback) {
      const Fallback = fallback;
      return <Fallback />;
    }
    
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <h3 className="text-yellow-800 font-medium">Component not found</h3>
        <p className="text-yellow-700 text-sm">Component '{name}' is not registered</p>
      </div>
    );
  }

  // If it's a remote component, use RemoteComponentLoader
  if (pluginId) {
    const microfrontend = registry.getMicrofrontend(pluginId);
    if (microfrontend) {
      return (
        <RemoteComponentLoader
          config={microfrontend}
          props={props}
          fallback={fallback}
          onError={onError}
        />
      );
    }
  }

  // Regular component
  const Component = component.component;
  return <Component {...props} />;
};

// Plugin Status Component - shows plugin loading/error states
interface PluginStatusProps {
  pluginId: string;
  children?: ReactNode;
}

export const PluginStatus: React.FC<PluginStatusProps> = ({ pluginId, children }) => {
  const registry = usePluginRegistry();
  const plugin = registry.getPlugin(pluginId);
  const isLoading = registry.loading.has(pluginId);
  const error = registry.getPluginError(pluginId);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-4">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
        <span className="ml-2">Loading plugin {plugin?.name || pluginId}...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Plugin Error</h3>
        <p className="text-red-600 text-sm">{error.message}</p>
        <p className="text-red-500 text-xs mt-1">Plugin: {plugin?.name || pluginId}</p>
      </div>
    );
  }

  if (!plugin) {
    return (
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <p className="text-gray-600">Plugin {pluginId} not found</p>
      </div>
    );
  }

  return <>{children}</>;
};

export default PluginRegistryProvider;