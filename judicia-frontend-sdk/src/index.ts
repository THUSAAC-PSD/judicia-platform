/**
 * @judicia/frontend-sdk
 * 
 * TypeScript SDK for developing Judicia Platform frontend components and micro frontends.
 * 
 * Features:
 * - Type-safe plugin component development
 * - Event-driven communication between plugins
 * - State management utilities
 * - Theme system integration
 * - Micro frontend component registration
 * - Platform API integration
 */

// Core types and interfaces
export * from './types';

// Component system
export * from './components';

// State management
export * from './store';

// Event system
export * from './events';

// Platform integration
export * from './platform';

// Utilities
export * from './utils';

// Hooks for React integration
export * from './hooks';

// Theme system
export * from './theme';

// Micro frontend utilities
export * from './microfrontend';

// Version information
export const VERSION = '1.0.0';

// Default export for convenience
export { JudiciaFrontendSDK as default } from './sdk';