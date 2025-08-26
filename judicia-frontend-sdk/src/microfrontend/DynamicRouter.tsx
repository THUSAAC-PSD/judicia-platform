/**
 * Dynamic Router for Plugin Routes
 * Handles dynamic route registration and rendering from plugins
 */

import React, { useMemo, useState, useEffect } from 'react';
import { Router, Route, Switch, useLocation, navigate } from 'wouter';
import { RouteRegistration, PluginCapability } from '../types';
import { usePluginRegistry } from './PluginRegistry';
import RemoteComponentLoader from './RemoteComponentLoader';
import { useJudiciaSDK } from '../sdk';

interface DynamicRouterProps {
  basePath?: string;
  fallback?: React.ComponentType;
  loadingComponent?: React.ComponentType;
  errorComponent?: React.ComponentType<{ error: Error }>;
}

export const DynamicRouter: React.FC<DynamicRouterProps> = ({
  basePath = '',
  fallback: Fallback,
  loadingComponent: LoadingComponent,
  errorComponent: ErrorComponent,
}) => {
  const registry = usePluginRegistry();
  const sdk = useJudiciaSDK();
  const [location] = useLocation();

  // Get all routes from registered plugins
  const routes = useMemo(() => {
    const allRoutes = registry.getAllRoutes();
    
    // Filter routes based on user capabilities
    return allRoutes.filter(route => {
      if (!route.requiresAuth) return true;
      if (!sdk.user) return false;
      
      if (route.requiredCapabilities?.length) {
        return route.requiredCapabilities.every(capability => 
          sdk.user?.permissions.includes(capability)
        );
      }
      
      return true;
    });
  }, [registry.routes, sdk.user]);

  // Build nested routes structure
  const routeTree = useMemo(() => {
    const tree: RouteRegistration[] = [];
    const routeMap = new Map<string, RouteRegistration>();
    
    // First pass: create route map
    routes.forEach(route => {
      routeMap.set(route.path, route);
    });
    
    // Second pass: build tree structure
    routes.forEach(route => {
      if (route.children) {
        route.children = route.children.filter(child => {
          const childRoute = routeMap.get(child.path);
          return childRoute && (!childRoute.requiresAuth || sdk.user);
        });
      }
      
      // Add to tree if it's a root route (no parent path)
      const isRootRoute = !routes.some(other => 
        other.children?.some(child => child.path === route.path)
      );
      
      if (isRootRoute) {
        tree.push(route);
      }
    });
    
    return tree;
  }, [routes, sdk.user]);

  // Route change handler
  useEffect(() => {
    const currentRoute = routes.find(route => {
      const routePath = basePath + route.path;
      return location === routePath || location.startsWith(routePath + '/');
    });
    
    if (currentRoute) {
      // Emit route change event
      sdk.events.emit('route.changed', {
        path: location,
        params: {}, // Would need proper parameter parsing
        route: currentRoute,
      });
      
      // Update document title
      if (currentRoute.title) {
        document.title = `${currentRoute.title} - Judicia Platform`;
      }
    }
  }, [location, routes, sdk.events, basePath]);

  const renderRoute = (route: RouteRegistration, isNested = false) => {
    const routePath = isNested ? route.path : basePath + route.path;
    
    return (
      <Route
        key={route.path}
        path={routePath}
        component={() => <RouteRenderer route={route} />}
      />
    );
  };

  const renderNestedRoutes = (parentRoute: RouteRegistration) => {
    if (!parentRoute.children?.length) return null;
    
    return (
      <Switch>
        {parentRoute.children.map(childRoute => renderRoute(childRoute, true))}
        <Route>
          {Fallback ? <Fallback /> : <div>Route not found</div>}
        </Route>
      </Switch>
    );
  };

  return (
    <Router base={basePath}>
      <Switch>
        {routeTree.map(route => renderRoute(route))}
        <Route>
          {Fallback ? <Fallback /> : (
            <div className="flex flex-col items-center justify-center min-h-64">
              <h2 className="text-2xl font-semibold text-gray-800 mb-2">Page Not Found</h2>
              <p className="text-gray-600 mb-4">The requested page could not be found.</p>
              <button
                onClick={() => navigate('/')}
                className="px-4 py-2 bg-primary text-white rounded hover:bg-primary/90"
              >
                Go Home
              </button>
            </div>
          )}
        </Route>
      </Switch>
    </Router>
  );
};

// Route Renderer Component
interface RouteRendererProps {
  route: RouteRegistration;
}

const RouteRenderer: React.FC<RouteRendererProps> = ({ route }) => {
  const registry = usePluginRegistry();
  const sdk = useJudiciaSDK();
  const [error, setError] = useState<Error | null>(null);

  // Check authentication
  if (route.requiresAuth && !sdk.user) {
    navigate('/login');
    return null;
  }

  // Check capabilities
  if (route.requiredCapabilities?.length && sdk.user) {
    const hasAllCapabilities = route.requiredCapabilities.every(capability =>
      sdk.user?.permissions.includes(capability)
    );
    
    if (!hasAllCapabilities) {
      return (
        <div className="bg-red-50 border border-red-200 rounded-lg p-6 max-w-md mx-auto mt-8">
          <h3 className="text-red-800 font-medium mb-2">Access Denied</h3>
          <p className="text-red-600">You don't have permission to access this page.</p>
          <p className="text-red-500 text-sm mt-2">
            Required capabilities: {route.requiredCapabilities.join(', ')}
          </p>
        </div>
      );
    }
  }

  // Find plugin that owns this route
  const ownerPlugin = registry.getAllPlugins().find(plugin =>
    plugin.routes.some(r => r.path === route.path)
  );

  // If it's a remote component, render with RemoteComponentLoader
  if (ownerPlugin) {
    const microfrontend = registry.getMicrofrontend(ownerPlugin.id);
    if (microfrontend) {
      return (
        <RemoteComponentLoader
          config={microfrontend}
          onError={setError}
          props={{
            route,
            location: window.location,
          }}
        />
      );
    }
  }

  // Render regular component
  const Component = route.component;
  if (!Component) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <h3 className="text-yellow-800 font-medium">Component Missing</h3>
        <p className="text-yellow-700">No component found for route: {route.path}</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Route Error</h3>
        <p className="text-red-600">{error.message}</p>
        <button
          onClick={() => setError(null)}
          className="mt-2 px-3 py-1 bg-red-100 text-red-800 rounded text-sm hover:bg-red-200"
        >
          Retry
        </button>
      </div>
    );
  }

  return <Component route={route} />;
};

// Navigation Helper Component
interface PluginNavigationProps {
  className?: string;
  onItemClick?: (route: RouteRegistration) => void;
}

export const PluginNavigation: React.FC<PluginNavigationProps> = ({
  className = '',
  onItemClick,
}) => {
  const registry = usePluginRegistry();
  const sdk = useJudiciaSDK();
  const [location] = useLocation();

  // Get navigation items from routes
  const navigationItems = useMemo(() => {
    return registry.getAllRoutes()
      .filter(route => {
        // Filter out routes that shouldn't appear in navigation
        if (!route.title) return false;
        if (route.path.includes('/:')) return false; // Dynamic routes
        if (route.requiresAuth && !sdk.user) return false;
        
        if (route.requiredCapabilities?.length && sdk.user) {
          return route.requiredCapabilities.every(capability =>
            sdk.user?.permissions.includes(capability)
          );
        }
        
        return true;
      })
      .sort((a, b) => a.path.localeCompare(b.path));
  }, [registry.routes, sdk.user]);

  const handleItemClick = (route: RouteRegistration) => {
    navigate(route.path);
    onItemClick?.(route);
  };

  return (
    <nav className={`plugin-navigation ${className}`}>
      <ul className="space-y-1">
        {navigationItems.map(route => (
          <li key={route.path}>
            <button
              onClick={() => handleItemClick(route)}
              className={`w-full text-left px-3 py-2 rounded-md transition-colors ${
                location === route.path
                  ? 'bg-primary text-white'
                  : 'text-gray-700 hover:bg-gray-100'
              }`}
            >
              <div className="flex items-center">
                {route.icon && <span className="mr-2">{route.icon}</span>}
                <span>{route.title}</span>
              </div>
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
};

// Breadcrumb Component
export const PluginBreadcrumb: React.FC = () => {
  const registry = usePluginRegistry();
  const [location] = useLocation();

  const breadcrumbs = useMemo(() => {
    const segments = location.split('/').filter(Boolean);
    const crumbs = [];
    
    let currentPath = '';
    for (const segment of segments) {
      currentPath += '/' + segment;
      const route = registry.getRoute(currentPath);
      
      crumbs.push({
        path: currentPath,
        title: route?.title || segment,
        isLast: currentPath === location,
      });
    }
    
    return crumbs;
  }, [location, registry]);

  if (breadcrumbs.length <= 1) return null;

  return (
    <nav aria-label="Breadcrumb" className="flex mb-4">
      <ol className="flex items-center space-x-2">
        {breadcrumbs.map((crumb, index) => (
          <li key={crumb.path} className="flex items-center">
            {index > 0 && <span className="text-gray-400 mx-2">/</span>}
            {crumb.isLast ? (
              <span className="text-gray-600 font-medium">{crumb.title}</span>
            ) : (
              <button
                onClick={() => navigate(crumb.path)}
                className="text-primary hover:underline"
              >
                {crumb.title}
              </button>
            )}
          </li>
        ))}
      </ol>
    </nav>
  );
};

export default DynamicRouter;