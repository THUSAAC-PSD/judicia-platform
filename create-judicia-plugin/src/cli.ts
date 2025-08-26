#!/usr/bin/env node

/**
 * CLI for creating Judicia plugins
 */

import { Command } from 'commander';
import chalk from 'chalk';
import { createPlugin } from './commands/create';
import { addComponent } from './commands/add-component';
import { addRoute } from './commands/add-route';
import { buildPlugin } from './commands/build';
import { servePlugin } from './commands/serve';

const program = new Command();

program
  .name('create-judicia-plugin')
  .description('CLI tool for scaffolding Judicia Platform plugins')
  .version('1.0.0');

program
  .command('create')
  .alias('c')
  .description('Create a new Judicia plugin')
  .argument('[name]', 'plugin name')
  .option('-t, --template <template>', 'plugin template', 'basic')
  .option('-d, --directory <directory>', 'output directory', '.')
  .option('--no-install', 'skip npm install')
  .option('--no-git', 'skip git initialization')
  .action(createPlugin);

program
  .command('add-component')
  .alias('ac')
  .description('Add a new component to the current plugin')
  .argument('<name>', 'component name')
  .option('-t, --type <type>', 'component type', 'react')
  .action(addComponent);

program
  .command('add-route')
  .alias('ar')
  .description('Add a new route to the current plugin')
  .argument('<path>', 'route path')
  .option('-c, --component <component>', 'component name')
  .action(addRoute);

program
  .command('build')
  .alias('b')
  .description('Build the plugin for production')
  .option('-w, --watch', 'watch for changes')
  .option('-m, --mode <mode>', 'build mode', 'production')
  .action(buildPlugin);

program
  .command('dev')
  .alias('d')
  .description('Start development server')
  .option('-p, --port <port>', 'development server port', '3000')
  .option('-h, --host <host>', 'development server host', 'localhost')
  .action(servePlugin);

// Global error handler
program.on('command:*', () => {
  console.error(chalk.red(`Invalid command: ${program.args.join(' ')}`));
  console.log('See --help for a list of available commands.');
  process.exit(1);
});

// Handle uncaught errors
process.on('uncaughtException', (error) => {
  console.error(chalk.red('Uncaught Exception:'), error);
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error(chalk.red('Unhandled Rejection at:'), promise, chalk.red('reason:'), reason);
  process.exit(1);
});

// Parse command line arguments
program.parse();

// Show help if no command provided
if (!process.argv.slice(2).length) {
  program.outputHelp();
}