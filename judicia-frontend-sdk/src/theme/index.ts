/**
 * Theme management system
 */

import { ThemeDefinition } from '../types';

export class ThemeManager {
  private currentTheme: ThemeDefinition | null = null;
  private mode: 'light' | 'dark' | 'auto' = 'auto';

  setTheme(theme: ThemeDefinition): void {
    this.currentTheme = theme;
    this.applyTheme();
  }

  setMode(mode: 'light' | 'dark' | 'auto'): void {
    this.mode = mode;
    this.applyTheme();
  }

  getTheme(): ThemeDefinition | null {
    return this.currentTheme;
  }

  getMode(): 'light' | 'dark' | 'auto' {
    return this.mode;
  }

  private applyTheme(): void {
    if (!this.currentTheme) return;

    const root = document.documentElement;
    
    // Apply CSS variables
    Object.entries(this.currentTheme.cssVariables || {}).forEach(([key, value]) => {
      root.style.setProperty(`--judicia-${key}`, value);
    });

    // Apply colors
    Object.entries(this.currentTheme.colors).forEach(([key, value]) => {
      root.style.setProperty(`--judicia-color-${key}`, value);
    });

    // Apply typography
    Object.entries(this.currentTheme.typography.fontSize).forEach(([key, value]) => {
      root.style.setProperty(`--judicia-text-${key}`, value);
    });

    // Apply spacing
    Object.entries(this.currentTheme.spacing).forEach(([key, value]) => {
      root.style.setProperty(`--judicia-space-${key}`, value);
    });
  }
}

export type { ThemeDefinition } from '../types';