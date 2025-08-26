/**
 * Simple routing system for plugin routes
 */

import { RouteRegistration } from '../types';

export class Router {
  private routes = new Map<string, RouteRegistration>();

  /**
   * Add a route
   */
  addRoute(route: RouteRegistration): void {
    this.routes.set(route.path, route);
  }

  /**
   * Remove a route
   */
  removeRoute(path: string): boolean {
    return this.routes.delete(path);
  }

  /**
   * Get a route by path
   */
  getRoute(path: string): RouteRegistration | null {
    return this.routes.get(path) || null;
  }

  /**
   * Get all routes
   */
  getAllRoutes(): RouteRegistration[] {
    return Array.from(this.routes.values());
  }

  /**
   * Clear all routes
   */
  clear(): void {
    this.routes.clear();
  }
}

export type { RouteRegistration, NavigationItem } from '../types';