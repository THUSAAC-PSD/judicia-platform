/**
 * Plugin Demo Component
 * Demonstrates micro frontend plugin loading and management
 */

import React, { useState, useEffect } from 'react';
import {
  PluginRegistryProvider,
  usePluginRegistry,
  DynamicComponent,
  DynamicRouter,
  PluginNavigation,
  RemoteComponentLoader,
  useJudiciaSDK,
} from '../../judicia-frontend-sdk/src';

// Mock plugin configurations for development
const MOCK_PLUGINS = [
  {
    id: 'standard-judge',
    name: 'Standard Judge',
    version: '1.0.0',
    author: 'Judicia Platform Team',
    description: 'Standard competitive programming judge plugin',
    dependencies: [],
    capabilities: ['read_problems', 'read_submissions', 'write_submissions'],
    components: [
      {
        name: 'JudgingStatus',
        component: null as any, // Will be loaded dynamically
        props: { submissionId: 'string' },
        lazy: false,
      }
    ],
    routes: [
      {
        path: '/judge/status',
        component: null as any,
        title: 'Judge Status',
        requiresAuth: true,
      }
    ],
  },
  {
    id: 'icpc-contest',
    name: 'ICPC Contest',
    version: '1.0.0',
    author: 'Judicia Platform Team',
    description: 'ICPC-style contest management plugin',
    dependencies: [],
    capabilities: ['read_contests', 'write_contests'],
    components: [
      {
        name: 'ScoreBoard',
        component: null as any,
        props: { contestId: 'string' },
        lazy: true,
      }
    ],
    routes: [
      {
        path: '/contest',
        component: null as any,
        title: 'Contests',
        requiresAuth: true,
      }
    ],
  }
];

const MOCK_MICROFRONTENDS = {
  'standard-judge': {
    name: 'standard-judge',
    url: 'http://localhost:5001/assets/remoteEntry.js',
    scope: 'standardJudge',
    module: './Plugin',
    type: 'module' as const,
    props: {},
    css: [],
  },
  'icpc-contest': {
    name: 'icpc-contest',
    url: 'http://localhost:5002/assets/remoteEntry.js',
    scope: 'icpcContest',
    module: './Plugin',
    type: 'module' as const,
    props: {},
    css: ['http://localhost:5002/assets/contest.css'],
  }
};

const PluginDemoContent: React.FC = () => {
  const registry = usePluginRegistry();
  const [selectedPlugin, setSelectedPlugin] = useState<string | null>(null);
  const [showRemoteLoader, setShowRemoteLoader] = useState(false);

  // Load mock plugins for demonstration
  useEffect(() => {
    const loadMockPlugins = async () => {
      for (const plugin of MOCK_PLUGINS) {
        const microfrontend = MOCK_MICROFRONTENDS[plugin.id as keyof typeof MOCK_MICROFRONTENDS];
        if (microfrontend) {
          try {
            await registry.registerPlugin(plugin, microfrontend);
            console.log(`Registered plugin: ${plugin.name}`);
          } catch (error) {
            console.error(`Failed to register plugin ${plugin.name}:`, error);
          }
        }
      }
    };

    loadMockPlugins();
  }, [registry]);

  const plugins = registry.getAllPlugins();
  const components = registry.getAllComponents();
  const routes = registry.getAllRoutes();

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto px-4 py-8">
        <h1 className="text-3xl font-bold mb-8">Micro Frontend Plugin Demo</h1>
        
        {/* Plugin Status Overview */}
        <div className="bg-white rounded-lg shadow-sm border p-6 mb-8">
          <h2 className="text-xl font-semibold mb-4">Plugin Registry Status</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-blue-50 p-4 rounded">
              <div className="text-2xl font-bold text-blue-600">{plugins.length}</div>
              <div className="text-blue-800">Registered Plugins</div>
            </div>
            <div className="bg-green-50 p-4 rounded">
              <div className="text-2xl font-bold text-green-600">{components.length}</div>
              <div className="text-green-800">Available Components</div>
            </div>
            <div className="bg-purple-50 p-4 rounded">
              <div className="text-2xl font-bold text-purple-600">{routes.length}</div>
              <div className="text-purple-800">Dynamic Routes</div>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Plugin List */}
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h2 className="text-xl font-semibold mb-4">Registered Plugins</h2>
            <div className="space-y-3">
              {plugins.map(plugin => (
                <div 
                  key={plugin.id}
                  className={`p-4 border rounded-lg cursor-pointer transition-colors ${
                    selectedPlugin === plugin.id 
                      ? 'border-blue-500 bg-blue-50' 
                      : 'border-gray-200 hover:border-gray-300'
                  }`}
                  onClick={() => setSelectedPlugin(plugin.id)}
                >
                  <div className="flex justify-between items-start">
                    <div>
                      <h3 className="font-medium">{plugin.name}</h3>
                      <p className="text-sm text-gray-600">{plugin.description}</p>
                      <div className="flex items-center mt-2 space-x-4">
                        <span className="text-xs bg-gray-100 px-2 py-1 rounded">
                          v{plugin.version}
                        </span>
                        <span className={`text-xs px-2 py-1 rounded ${
                          registry.isPluginLoaded(plugin.id)
                            ? 'bg-green-100 text-green-800'
                            : 'bg-yellow-100 text-yellow-800'
                        }`}>
                          {registry.isPluginLoaded(plugin.id) ? 'Loaded' : 'Loading'}
                        </span>
                      </div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        registry.unregisterPlugin(plugin.id);
                      }}
                      className="text-red-500 hover:text-red-700 text-sm"
                    >
                      Remove
                    </button>
                  </div>
                </div>
              ))}
              {plugins.length === 0 && (
                <div className="text-center py-8 text-gray-500">
                  No plugins registered. Plugins will auto-load from the discovery endpoint.
                </div>
              )}
            </div>
          </div>

          {/* Plugin Details */}
          <div className="bg-white rounded-lg shadow-sm border p-6">
            <h2 className="text-xl font-semibold mb-4">Plugin Details</h2>
            {selectedPlugin ? (
              <PluginDetails pluginId={selectedPlugin} />
            ) : (
              <div className="text-center py-8 text-gray-500">
                Select a plugin to view details
              </div>
            )}
          </div>
        </div>

        {/* Component Demo */}
        <div className="bg-white rounded-lg shadow-sm border p-6 mt-8">
          <h2 className="text-xl font-semibold mb-4">Dynamic Component Loading</h2>
          <div className="flex flex-wrap gap-2 mb-4">
            {components.map(comp => (
              <button
                key={comp.name}
                onClick={() => setShowRemoteLoader(comp.name)}
                className="px-3 py-1 bg-blue-100 text-blue-800 rounded hover:bg-blue-200"
              >
                Load {comp.name}
              </button>
            ))}
          </div>
          
          {showRemoteLoader && (
            <div className="border-t pt-4">
              <h3 className="font-medium mb-2">Component: {showRemoteLoader}</h3>
              <DynamicComponent
                name={showRemoteLoader}
                props={{
                  submissionId: '123',
                  contestId: 'demo-contest',
                }}
                fallback={() => (
                  <div className="bg-yellow-50 border border-yellow-200 rounded p-4">
                    <p>Component fallback - Plugin not available</p>
                  </div>
                )}
                onError={(error) => console.error('Component error:', error)}
              />
            </div>
          )}
        </div>

        {/* Navigation Demo */}
        <div className="bg-white rounded-lg shadow-sm border p-6 mt-8">
          <h2 className="text-xl font-semibold mb-4">Dynamic Navigation</h2>
          <PluginNavigation 
            className="max-w-md"
            onItemClick={(route) => {
              console.log('Navigation item clicked:', route);
            }}
          />
        </div>
      </div>
    </div>
  );
};

const PluginDetails: React.FC<{ pluginId: string }> = ({ pluginId }) => {
  const registry = usePluginRegistry();
  const plugin = registry.getPlugin(pluginId);
  const microfrontend = registry.getMicrofrontend(pluginId);
  const error = registry.getPluginError(pluginId);

  if (!plugin) return <div>Plugin not found</div>;

  return (
    <div className="space-y-4">
      {error && (
        <div className="bg-red-50 border border-red-200 rounded p-3">
          <h4 className="text-red-800 font-medium">Error</h4>
          <p className="text-red-600 text-sm">{error.message}</p>
        </div>
      )}
      
      <div>
        <h3 className="font-medium mb-2">Basic Info</h3>
        <dl className="text-sm space-y-1">
          <div><dt className="inline font-medium">Author:</dt> <dd className="inline">{plugin.author}</dd></div>
          <div><dt className="inline font-medium">Version:</dt> <dd className="inline">{plugin.version}</dd></div>
          <div><dt className="inline font-medium">Capabilities:</dt> <dd className="inline">{plugin.capabilities.join(', ')}</dd></div>
        </dl>
      </div>

      {microfrontend && (
        <div>
          <h3 className="font-medium mb-2">Microfrontend Config</h3>
          <pre className="bg-gray-50 p-3 rounded text-xs overflow-auto">
            {JSON.stringify(microfrontend, null, 2)}
          </pre>
        </div>
      )}

      <div>
        <h3 className="font-medium mb-2">Components ({plugin.components.length})</h3>
        <div className="space-y-1">
          {plugin.components.map(comp => (
            <div key={comp.name} className="text-sm bg-gray-50 p-2 rounded">
              <span className="font-medium">{comp.name}</span>
              {comp.lazy && <span className="ml-2 text-xs bg-yellow-100 text-yellow-800 px-1 rounded">Lazy</span>}
            </div>
          ))}
        </div>
      </div>

      <div>
        <h3 className="font-medium mb-2">Routes ({plugin.routes.length})</h3>
        <div className="space-y-1">
          {plugin.routes.map(route => (
            <div key={route.path} className="text-sm bg-gray-50 p-2 rounded">
              <span className="font-medium">{route.path}</span>
              {route.title && <span className="ml-2 text-gray-600">- {route.title}</span>}
              {route.requiresAuth && <span className="ml-2 text-xs bg-red-100 text-red-800 px-1 rounded">Auth Required</span>}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const PluginDemo: React.FC = () => {
  return (
    <PluginRegistryProvider
      autoDiscovery={true}
      discoveryEndpoint="/api/plugins/discovery"
    >
      <PluginDemoContent />
    </PluginRegistryProvider>
  );
};

export default PluginDemo;