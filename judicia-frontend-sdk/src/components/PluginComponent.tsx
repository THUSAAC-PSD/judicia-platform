/**
 * Generic component wrapper for plugin components
 */

import React, { ComponentType, useEffect, useState } from 'react';
import { useJudicia } from '../hooks';
import { ComponentRegistration } from '../types';

export interface PluginComponentProps {
  name: string;
  props?: Record<string, any>;
  fallback?: ComponentType;
  onError?: (error: Error) => void;
}

export function PluginComponent({ 
  name, 
  props = {}, 
  fallback: Fallback,
  onError 
}: PluginComponentProps) {
  const sdk = useJudicia();
  const [registration, setRegistration] = useState<ComponentRegistration | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const loadComponent = async () => {
      try {
        setLoading(true);
        setError(null);

        // Try to get the component from registry
        let componentReg = sdk.getComponent(name);
        
        if (!componentReg) {
          // Component not found - could try to load it dynamically
          setError(new Error(`Component "${name}" not found`));
          return;
        }

        // Validate props if schema is provided
        if (componentReg.props) {
          const validation = sdk.components.validateProps(name, props);
          if (!validation.valid) {
            setError(new Error(`Invalid props for component "${name}": ${validation.errors.join(', ')}`));
            return;
          }
        }

        setRegistration(componentReg);
      } catch (err) {
        const error = err instanceof Error ? err : new Error('Unknown error');
        setError(error);
        onError?.(error);
      } finally {
        setLoading(false);
      }
    };

    loadComponent();
  }, [name, props, sdk, onError]);

  if (loading) {
    return <div className="judicia-plugin-component-loading">Loading component...</div>;
  }

  if (error) {
    if (Fallback) {
      return <Fallback />;
    }
    return (
      <div className="judicia-plugin-component-error">
        <p>Failed to load component "{name}"</p>
        <p>{error.message}</p>
      </div>
    );
  }

  if (!registration) {
    if (Fallback) {
      return <Fallback />;
    }
    return (
      <div className="judicia-plugin-component-not-found">
        Component "{name}" not found
      </div>
    );
  }

  const Component = registration.component;
  
  return <Component {...props} />;
}