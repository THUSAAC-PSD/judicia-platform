/**
 * Available plugin templates
 */

import path from 'path';

export interface Template {
  name: string;
  description: string;
  path: string;
  language: 'rust' | 'typescript';
  features: string[];
}

export const TEMPLATES: Record<string, Template> = {
  basic: {
    name: 'Basic Plugin',
    description: 'A minimal plugin with basic structure',
    path: path.join(__dirname, '../../templates/basic'),
    language: 'rust',
    features: ['basic-structure', 'hello-world-component']
  },
  
  'contest-plugin': {
    name: 'Contest Plugin',
    description: 'Plugin for managing contests and competitions',
    path: path.join(__dirname, '../../templates/contest-plugin'),
    language: 'rust',
    features: ['contest-management', 'leaderboard', 'team-registration']
  },
  
  'problem-solver': {
    name: 'Problem Solver',
    description: 'Plugin for problem-solving interfaces and tools',
    path: path.join(__dirname, '../../templates/problem-solver'),
    language: 'rust',
    features: ['code-editor', 'submission-handler', 'test-runner']
  },
  
  'judge-system': {
    name: 'Judge System',
    description: 'Custom judging system with evaluation logic',
    path: path.join(__dirname, '../../templates/judge-system'),
    language: 'rust',
    features: ['custom-evaluation', 'test-case-management', 'scoring']
  },
  
  'frontend-only': {
    name: 'Frontend Only',
    description: 'TypeScript frontend-only plugin',
    path: path.join(__dirname, '../../templates/frontend-only'),
    language: 'typescript',
    features: ['react-components', 'routing', 'state-management']
  },
  
  'admin-dashboard': {
    name: 'Admin Dashboard',
    description: 'Administrative dashboard and tools',
    path: path.join(__dirname, '../../templates/admin-dashboard'),
    language: 'typescript',
    features: ['user-management', 'system-monitoring', 'configuration']
  },
  
  'notification-system': {
    name: 'Notification System',
    description: 'Plugin for handling notifications and alerts',
    path: path.join(__dirname, '../../templates/notification-system'),
    language: 'rust',
    features: ['real-time-notifications', 'email-integration', 'push-notifications']
  },
  
  'analytics': {
    name: 'Analytics Plugin',
    description: 'Analytics and reporting functionality',
    path: path.join(__dirname, '../../templates/analytics'),
    language: 'typescript',
    features: ['data-visualization', 'reporting', 'metrics-collection']
  }
};