/**
 * Template generation utilities
 */

import fs from 'fs-extra';
import path from 'path';
import Handlebars from 'handlebars';

export async function generateFromTemplate(
  templatePath: string,
  outputPath: string,
  data: Record<string, any>
): Promise<void> {
  // Register Handlebars helpers
  registerHandlebarsHelpers();

  // Recursively process template directory
  await processDirectory(templatePath, outputPath, data);
}

async function processDirectory(
  templateDir: string,
  outputDir: string,
  data: Record<string, any>
): Promise<void> {
  await fs.ensureDir(outputDir);

  const entries = await fs.readdir(templateDir, { withFileTypes: true });

  for (const entry of entries) {
    const templatePath = path.join(templateDir, entry.name);
    const outputName = processFileName(entry.name, data);
    const outputPath = path.join(outputDir, outputName);

    if (entry.isDirectory()) {
      // Recursively process subdirectory
      await processDirectory(templatePath, outputPath, data);
    } else {
      // Process file
      await processFile(templatePath, outputPath, data);
    }
  }
}

async function processFile(
  templatePath: string,
  outputPath: string,
  data: Record<string, any>
): Promise<void> {
  const templateContent = await fs.readFile(templatePath, 'utf-8');

  if (path.extname(templatePath) === '.hbs') {
    // Process Handlebars template
    const template = Handlebars.compile(templateContent);
    const processedContent = template(data);
    
    // Remove .hbs extension from output file
    const finalOutputPath = outputPath.replace(/\.hbs$/, '');
    await fs.writeFile(finalOutputPath, processedContent);
  } else {
    // Copy binary file as-is
    await fs.copy(templatePath, outputPath);
  }
}

function processFileName(fileName: string, data: Record<string, any>): string {
  // Replace template variables in file names
  let processedName = fileName;
  
  // Replace common template variables
  processedName = processedName.replace(/{{kebabCase}}/g, data.kebabCase || 'my-plugin');
  processedName = processedName.replace(/{{pascalCase}}/g, data.pascalCase || 'MyPlugin');
  processedName = processedName.replace(/{{camelCase}}/g, data.camelCase || 'myPlugin');
  
  return processedName;
}

function registerHandlebarsHelpers(): void {
  // Helper for conditional rendering
  Handlebars.registerHelper('if_eq', function(a, b, options) {
    if (a === b) {
      return options.fn(this);
    }
    return options.inverse(this);
  });

  // Helper for array length check
  Handlebars.registerHelper('if_gt', function(a, b, options) {
    if (a > b) {
      return options.fn(this);
    }
    return options.inverse(this);
  });

  // Helper for capitalizing strings
  Handlebars.registerHelper('capitalize', function(str: string) {
    return str.charAt(0).toUpperCase() + str.slice(1);
  });

  // Helper for joining arrays
  Handlebars.registerHelper('join', function(array: string[], separator: string = ', ') {
    return array.join(separator);
  });

  // Helper for JSON stringify
  Handlebars.registerHelper('json', function(obj: any) {
    return JSON.stringify(obj, null, 2);
  });

  // Helper for date formatting
  Handlebars.registerHelper('date', function(format: string = 'YYYY-MM-DD') {
    const now = new Date();
    if (format === 'YYYY-MM-DD') {
      return now.toISOString().split('T')[0];
    }
    if (format === 'YYYY') {
      return now.getFullYear().toString();
    }
    return now.toISOString();
  });

  // Helper for converting to different cases
  Handlebars.registerHelper('kebabCase', function(str: string) {
    return str.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
  });

  Handlebars.registerHelper('camelCase', function(str: string) {
    return str.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
  });

  Handlebars.registerHelper('pascalCase', function(str: string) {
    return str.split('-').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join('');
  });

  Handlebars.registerHelper('snakeCase', function(str: string) {
    return str.replace(/-/g, '_').toLowerCase();
  });

  Handlebars.registerHelper('constantCase', function(str: string) {
    return str.replace(/-/g, '_').toUpperCase();
  });
}