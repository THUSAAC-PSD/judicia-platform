/**
 * Component Registry for managing plugin components
 */

import { ComponentType } from 'react';
import { ComponentRegistration, ComponentPropSchema } from '../types';

export class ComponentRegistry {
  private components = new Map<string, ComponentRegistration>();

  /**
   * Register a component
   */
  register(registration: ComponentRegistration): void {
    if (this.components.has(registration.name)) {
      console.warn(`Component ${registration.name} is already registered. Overwriting.`);
    }

    // Validate component registration
    this.validateRegistration(registration);

    this.components.set(registration.name, registration);
  }

  /**
   * Get a component by name
   */
  get(name: string): ComponentRegistration | null {
    return this.components.get(name) || null;
  }

  /**
   * Check if a component is registered
   */
  has(name: string): boolean {
    return this.components.has(name);
  }

  /**
   * Get all registered component names
   */
  getNames(): string[] {
    return Array.from(this.components.keys());
  }

  /**
   * Get all registered components
   */
  getAll(): ComponentRegistration[] {
    return Array.from(this.components.values());
  }

  /**
   * Unregister a component
   */
  unregister(name: string): boolean {
    return this.components.delete(name);
  }

  /**
   * Clear all registered components
   */
  clear(): void {
    this.components.clear();
  }

  /**
   * Validate props against component schema
   */
  validateProps(componentName: string, props: Record<string, any>): {
    valid: boolean;
    errors: string[];
  } {
    const registration = this.get(componentName);
    if (!registration || !registration.props) {
      return { valid: true, errors: [] };
    }

    const errors: string[] = [];
    const schema = registration.props;

    // Check required props
    for (const [propName, propDef] of Object.entries(schema)) {
      if (propDef.required && !(propName in props)) {
        errors.push(`Missing required prop: ${propName}`);
      }
    }

    // Validate provided props
    for (const [propName, value] of Object.entries(props)) {
      if (!(propName in schema)) {
        continue; // Allow unknown props for flexibility
      }

      const propDef = schema[propName];
      
      // Type validation
      if (!this.validatePropType(value, propDef.type)) {
        errors.push(`Invalid type for prop ${propName}: expected ${propDef.type}, got ${typeof value}`);
      }

      // Custom validation
      if (propDef.validation && !propDef.validation(value)) {
        errors.push(`Validation failed for prop ${propName}`);
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  private validateRegistration(registration: ComponentRegistration): void {
    if (!registration.name) {
      throw new Error('Component registration must have a name');
    }

    if (!registration.component) {
      throw new Error('Component registration must have a component');
    }

    if (typeof registration.component !== 'function') {
      throw new Error('Component must be a React component function or class');
    }
  }

  private validatePropType(value: any, expectedType: string): boolean {
    switch (expectedType) {
      case 'string':
        return typeof value === 'string';
      case 'number':
        return typeof value === 'number' && !isNaN(value);
      case 'boolean':
        return typeof value === 'boolean';
      case 'object':
        return typeof value === 'object' && value !== null && !Array.isArray(value);
      case 'array':
        return Array.isArray(value);
      default:
        return true; // Allow unknown types
    }
  }
}