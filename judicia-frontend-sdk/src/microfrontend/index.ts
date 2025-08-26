/**
 * Micro frontend utilities and components
 */

// Core components
export { default as RemoteComponentLoader, useRemoteComponent, preloadRemoteComponent } from './RemoteComponentLoader';
export { 
  default as PluginRegistryProvider, 
  usePluginRegistry, 
  DynamicComponent, 
  PluginStatus 
} from './PluginRegistry';
export { 
  DynamicRouter, 
  PluginNavigation, 
  PluginBreadcrumb 
} from './DynamicRouter';

// Types
export type { MicrofrontendConfig, RemoteComponent } from '../types';