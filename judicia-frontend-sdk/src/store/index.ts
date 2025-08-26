/**
 * State management using Zustand
 */

import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { GlobalState, JudiciaPlatformContext } from '../types';

// Initial state
const initialState: GlobalState = {
  platform: {
    user: null,
    theme: 'auto',
    locale: 'en-US',
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    isAdmin: false,
    isModerator: false,
    features: [],
  },
  plugins: {},
  ui: {
    sidebarCollapsed: false,
    modalStack: [],
    notifications: [],
    loading: {},
  },
  cache: {
    problems: {},
    submissions: {},
    contests: {},
    users: {},
  },
};

// Create store
export const useJudiciaStore = create<GlobalState>()(
  subscribeWithSelector(() => initialState)
);

// Store actions
export const judiciaStore = {
  setState: useJudiciaStore.setState,
  getState: useJudiciaStore.getState,
  subscribe: useJudiciaStore.subscribe,

  // Platform actions
  updatePlatform: (updates: Partial<JudiciaPlatformContext>) => {
    useJudiciaStore.setState(state => ({
      ...state,
      platform: { ...state.platform, ...updates }
    }));
  },

  // UI actions
  setSidebarCollapsed: (collapsed: boolean) => {
    useJudiciaStore.setState(state => ({
      ...state,
      ui: { ...state.ui, sidebarCollapsed: collapsed }
    }));
  },

  pushModal: (modalId: string) => {
    useJudiciaStore.setState(state => ({
      ...state,
      ui: { 
        ...state.ui, 
        modalStack: [...state.ui.modalStack, modalId] 
      }
    }));
  },

  popModal: () => {
    useJudiciaStore.setState(state => ({
      ...state,
      ui: { 
        ...state.ui, 
        modalStack: state.ui.modalStack.slice(0, -1)
      }
    }));
  },

  addNotification: (notification: any) => {
    useJudiciaStore.setState(state => ({
      ...state,
      ui: { 
        ...state.ui, 
        notifications: [...state.ui.notifications, notification]
      }
    }));
  },

  removeNotification: (notificationId: string) => {
    useJudiciaStore.setState(state => ({
      ...state,
      ui: { 
        ...state.ui, 
        notifications: state.ui.notifications.filter(n => n.id !== notificationId)
      }
    }));
  },

  setLoading: (key: string, loading: boolean) => {
    useJudiciaStore.setState(state => ({
      ...state,
      ui: { 
        ...state.ui, 
        loading: { ...state.ui.loading, [key]: loading }
      }
    }));
  },

  // Cache actions
  setCacheData: (type: keyof GlobalState['cache'], id: string, data: any) => {
    useJudiciaStore.setState(state => ({
      ...state,
      cache: {
        ...state.cache,
        [type]: { ...state.cache[type], [id]: data }
      }
    }));
  },

  clearCache: (type?: keyof GlobalState['cache']) => {
    if (type) {
      useJudiciaStore.setState(state => ({
        ...state,
        cache: { ...state.cache, [type]: {} }
      }));
    } else {
      useJudiciaStore.setState(state => ({
        ...state,
        cache: {
          problems: {},
          submissions: {},
          contests: {},
          users: {},
        }
      }));
    }
  },
};

export const createStore = () => useJudiciaStore;

export default useJudiciaStore;