/**
 * Create plugin command
 */

import fs from 'fs-extra';
import path from 'path';
import chalk from 'chalk';
import { prompt } from 'enquirer';
import ora from 'ora';
import validatePackageName from 'validate-npm-package-name';
import { exec } from 'child_process';
import { promisify } from 'util';
import { generateFromTemplate } from '../utils/template';
import { TEMPLATES } from '../templates';

const execAsync = promisify(exec);

interface CreateOptions {
  template: string;
  directory: string;
  install: boolean;
  git: boolean;
}

interface PluginPromptAnswers {
  name: string;
  displayName: string;
  description: string;
  author: string;
  email: string;
  repository: string;
  license: string;
  template: string;
  language: 'rust' | 'typescript';
  capabilities: string[];
  frontend: boolean;
  routes: boolean;
}

export async function createPlugin(name: string | undefined, options: CreateOptions): Promise<void> {
  console.log(chalk.blue('🎯 Welcome to Judicia Plugin Creator!'));
  console.log();

  try {
    // Get plugin information through prompts
    const answers = await getPluginInfo(name, options);
    
    // Validate and create project directory
    const projectPath = path.resolve(options.directory, answers.name);
    await validateAndCreateDirectory(projectPath);

    // Generate plugin from template
    const spinner = ora('Creating plugin from template...').start();
    
    try {
      await generatePlugin(projectPath, answers, options);
      spinner.succeed('Plugin structure created successfully');
    } catch (error) {
      spinner.fail('Failed to create plugin structure');
      throw error;
    }

    // Initialize git repository
    if (options.git) {
      const gitSpinner = ora('Initializing git repository...').start();
      try {
        await initializeGit(projectPath);
        gitSpinner.succeed('Git repository initialized');
      } catch (error) {
        gitSpinner.warn('Failed to initialize git repository');
        console.log(chalk.yellow('You can initialize git manually later'));
      }
    }

    // Install dependencies
    if (options.install) {
      const installSpinner = ora('Installing dependencies...').start();
      try {
        await installDependencies(projectPath, answers.language);
        installSpinner.succeed('Dependencies installed successfully');
      } catch (error) {
        installSpinner.warn('Failed to install dependencies');
        console.log(chalk.yellow('You can install dependencies manually later'));
      }
    }

    // Show success message and next steps
    showSuccessMessage(answers.name, projectPath, options);

  } catch (error) {
    console.error(chalk.red('Failed to create plugin:'), error);
    process.exit(1);
  }
}

async function getPluginInfo(name: string | undefined, options: CreateOptions): Promise<PluginPromptAnswers> {
  const questions = [
    {
      type: 'input' as const,
      name: 'name',
      message: 'Plugin name:',
      initial: name || 'my-judicia-plugin',
      validate: (value: string) => {
        const result = validatePackageName(value);
        if (!result.validForNewPackages) {
          return result.errors?.[0] || result.warnings?.[0] || 'Invalid package name';
        }
        return true;
      }
    },
    {
      type: 'input' as const,
      name: 'displayName',
      message: 'Display name:',
      initial: (answers: any) => answers.name.split('-').map((word: string) => 
        word.charAt(0).toUpperCase() + word.slice(1)
      ).join(' ')
    },
    {
      type: 'input' as const,
      name: 'description',
      message: 'Description:',
      initial: 'A Judicia Platform plugin'
    },
    {
      type: 'input' as const,
      name: 'author',
      message: 'Author name:',
      initial: process.env.USER || process.env.USERNAME || 'Anonymous'
    },
    {
      type: 'input' as const,
      name: 'email',
      message: 'Author email:',
      initial: process.env.EMAIL || ''
    },
    {
      type: 'input' as const,
      name: 'repository',
      message: 'Repository URL:',
      initial: ''
    },
    {
      type: 'select' as const,
      name: 'license',
      message: 'License:',
      choices: ['MIT', 'Apache-2.0', 'GPL-3.0', 'BSD-3-Clause', 'ISC'],
      initial: 'MIT'
    },
    {
      type: 'select' as const,
      name: 'template',
      message: 'Plugin template:',
      choices: Object.keys(TEMPLATES),
      initial: options.template || 'basic',
      skip: () => !!options.template
    },
    {
      type: 'select' as const,
      name: 'language',
      message: 'Implementation language:',
      choices: [
        { name: 'rust', message: 'Rust (WebAssembly)' },
        { name: 'typescript', message: 'TypeScript (Browser)' }
      ],
      initial: 'rust'
    },
    {
      type: 'multiselect' as const,
      name: 'capabilities',
      message: 'Required capabilities:',
      choices: [
        { name: 'read_problems', message: 'Read Problems' },
        { name: 'write_problems', message: 'Write Problems' },
        { name: 'read_submissions', message: 'Read Submissions' },
        { name: 'write_submissions', message: 'Write Submissions' },
        { name: 'read_contests', message: 'Read Contests' },
        { name: 'write_contests', message: 'Write Contests' },
        { name: 'register_components', message: 'Register UI Components' },
        { name: 'register_routes', message: 'Register Routes' },
        { name: 'emit_events', message: 'Emit Events' },
        { name: 'subscribe_events', message: 'Subscribe to Events' },
        { name: 'notifications', message: 'Send Notifications' },
        { name: 'file_storage', message: 'File Storage Access' },
        { name: 'admin_operations', message: 'Admin Operations' }
      ],
      initial: ['read_problems', 'emit_events']
    },
    {
      type: 'confirm' as const,
      name: 'frontend',
      message: 'Include frontend components?',
      initial: true
    },
    {
      type: 'confirm' as const,
      name: 'routes',
      message: 'Include custom routes?',
      initial: false
    }
  ];

  return await prompt(questions) as PluginPromptAnswers;
}

async function validateAndCreateDirectory(projectPath: string): Promise<void> {
  if (await fs.pathExists(projectPath)) {
    const files = await fs.readdir(projectPath);
    if (files.length > 0) {
      const { overwrite } = await prompt({
        type: 'confirm' as const,
        name: 'overwrite',
        message: `Directory ${path.basename(projectPath)} is not empty. Overwrite?`,
        initial: false
      }) as { overwrite: boolean };

      if (!overwrite) {
        console.log(chalk.yellow('Plugin creation cancelled'));
        process.exit(0);
      }

      await fs.emptyDir(projectPath);
    }
  } else {
    await fs.ensureDir(projectPath);
  }
}

async function generatePlugin(
  projectPath: string, 
  answers: PluginPromptAnswers, 
  options: CreateOptions
): Promise<void> {
  const template = TEMPLATES[answers.template as keyof typeof TEMPLATES];
  
  if (!template) {
    throw new Error(`Template "${answers.template}" not found`);
  }

  const templateData = {
    ...answers,
    packageName: answers.name,
    kebabCase: answers.name,
    pascalCase: answers.name.split('-').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(''),
    camelCase: answers.name.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()),
    year: new Date().getFullYear(),
    date: new Date().toISOString().split('T')[0],
    hasRepository: !!answers.repository,
    capabilitiesArray: answers.capabilities.map(cap => `"${cap}"`).join(', '),
    capabilitiesList: answers.capabilities.map(cap => `- ${cap}`).join('\n'),
  };

  await generateFromTemplate(template.path, projectPath, templateData);
}

async function initializeGit(projectPath: string): Promise<void> {
  await execAsync('git init', { cwd: projectPath });
  await execAsync('git add .', { cwd: projectPath });
  await execAsync('git commit -m "Initial commit"', { cwd: projectPath });
}

async function installDependencies(projectPath: string, language: 'rust' | 'typescript'): Promise<void> {
  if (language === 'rust') {
    // For Rust plugins, we need cargo
    await execAsync('cargo check', { cwd: projectPath });
  } else {
    // For TypeScript plugins, use npm
    if (await fs.pathExists(path.join(projectPath, 'package.json'))) {
      await execAsync('npm install', { cwd: projectPath });
    }
  }
}

function showSuccessMessage(name: string, projectPath: string, options: CreateOptions): void {
  console.log();
  console.log(chalk.green('🎉 Success! Created plugin', chalk.bold(name)));
  console.log('📁 Plugin created at:', chalk.cyan(projectPath));
  console.log();
  
  console.log(chalk.yellow('Next steps:'));
  console.log(`  1. ${chalk.cyan(`cd ${path.basename(projectPath)}`)}`);
  
  if (!options.install) {
    console.log(`  2. ${chalk.cyan('npm install')} (or ${chalk.cyan('cargo build')} for Rust)`);
  }
  
  console.log(`  ${options.install ? '2' : '3'}. ${chalk.cyan('npm run dev')} (or ${chalk.cyan('cargo run')} for Rust)`);
  console.log(`  ${options.install ? '3' : '4'}. Start developing your plugin!`);
  console.log();
  
  console.log(chalk.gray('Useful commands:'));
  console.log(chalk.gray(`  ${chalk.white('create-judicia-plugin add-component <name>')} - Add a new component`));
  console.log(chalk.gray(`  ${chalk.white('create-judicia-plugin add-route <path>')} - Add a new route`));
  console.log(chalk.gray(`  ${chalk.white('create-judicia-plugin build')} - Build for production`));
  console.log();
  
  console.log(chalk.blue('📚 Documentation: https://docs.judicia.dev'));
  console.log(chalk.blue('💬 Community: https://discord.gg/judicia'));
  console.log();
}