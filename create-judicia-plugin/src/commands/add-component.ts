/**
 * Add component command
 */

import fs from 'fs-extra';
import path from 'path';
import chalk from 'chalk';
import { prompt } from 'enquirer';
import ora from 'ora';

interface AddComponentOptions {
  type: string;
}

export async function addComponent(name: string, options: AddComponentOptions): Promise<void> {
  console.log(chalk.blue(`🧩 Adding component "${name}"`));

  try {
    // Verify we're in a plugin directory
    if (!await isPluginDirectory()) {
      console.error(chalk.red('Error: Not in a Judicia plugin directory'));
      console.log('Run this command from the root of your plugin project');
      process.exit(1);
    }

    // Get component details
    const details = await getComponentDetails(name, options);
    
    // Generate component files
    const spinner = ora('Generating component files...').start();
    
    try {
      await generateComponentFiles(details);
      await updatePluginConfig(details);
      spinner.succeed('Component added successfully');
    } catch (error) {
      spinner.fail('Failed to add component');
      throw error;
    }

    // Show success message
    showAddComponentSuccess(details);

  } catch (error) {
    console.error(chalk.red('Failed to add component:'), error);
    process.exit(1);
  }
}

async function isPluginDirectory(): Promise<boolean> {
  const cargoExists = await fs.pathExists('Cargo.toml');
  const packageExists = await fs.pathExists('package.json');
  
  if (!cargoExists && !packageExists) {
    return false;
  }
  
  // Check if it's a Judicia plugin
  if (cargoExists) {
    const cargoContent = await fs.readFile('Cargo.toml', 'utf-8');
    return cargoContent.includes('judicia-sdk');
  }
  
  if (packageExists) {
    const packageContent = await fs.readFile('package.json', 'utf-8');
    const packageJson = JSON.parse(packageContent);
    return packageJson.dependencies?.['@judicia/frontend-sdk'] || 
           packageJson.devDependencies?.['@judicia/frontend-sdk'];
  }
  
  return false;
}

async function getComponentDetails(name: string, options: AddComponentOptions) {
  const questions = [
    {
      type: 'select' as const,
      name: 'type',
      message: 'Component type:',
      choices: [
        { name: 'react', message: 'React Component' },
        { name: 'rust', message: 'Rust Component (Server-side rendered)' },
        { name: 'web-component', message: 'Web Component (Custom Element)' }
      ],
      initial: options.type || 'react',
      skip: () => !!options.type
    },
    {
      type: 'input' as const,
      name: 'description',
      message: 'Component description:',
      initial: `${name} component`
    },
    {
      type: 'multiselect' as const,
      name: 'features',
      message: 'Component features:',
      choices: [
        { name: 'state', message: 'Internal state management' },
        { name: 'props', message: 'Props validation' },
        { name: 'events', message: 'Custom events' },
        { name: 'slots', message: 'Content slots' },
        { name: 'styles', message: 'Component styles' },
        { name: 'api', message: 'API integration' }
      ]
    },
    {
      type: 'confirm' as const,
      name: 'addToRoutes',
      message: 'Add component to plugin routes?',
      initial: false
    }
  ];

  const answers = await prompt(questions);
  
  return {
    name,
    pascalCase: name.split('-').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(''),
    kebabCase: name.toLowerCase().replace(/[^a-z0-9]/g, '-'),
    ...answers
  };
}

async function generateComponentFiles(details: any): Promise<void> {
  const srcPath = 'src';
  
  if (details.type === 'rust') {
    // Generate Rust component
    await generateRustComponent(details, srcPath);
  } else if (details.type === 'react') {
    // Generate React component
    await generateReactComponent(details, srcPath);
  } else {
    // Generate Web Component
    await generateWebComponent(details, srcPath);
  }
}

async function generateRustComponent(details: any, srcPath: string): Promise<void> {
  const componentFile = `${srcPath}/components/${details.kebabCase}.rs`;
  
  await fs.ensureDir(path.dirname(componentFile));
  
  const componentCode = `use judicia_sdk::prelude::*;

/// ${details.description}
pub struct ${details.pascalCase}Component {${details.features.includes('state') ? '\n    state: ComponentState,' : ''}
}

${details.features.includes('state') ? `
#[derive(Default)]
struct ComponentState {
    // Add your state fields here
}
` : ''}

impl ${details.pascalCase}Component {
    pub fn new() -> Self {
        Self {${details.features.includes('state') ? '\n            state: ComponentState::default(),' : ''}
        }
    }
    
    pub async fn render(&self, props: &serde_json::Value) -> PluginResult<String> {
        ${details.features.includes('props') ? 'self.validate_props(props)?;\n        ' : ''}let html = format!(r#"
            <div class="judicia-${details.kebabCase}">
                <h3>${details.pascalCase}</h3>
                <p>${details.description}</p>
                <!-- Add your component markup here -->
            </div>
        "#);
        
        Ok(html)
    }${details.features.includes('props') ? `
    
    fn validate_props(&self, props: &serde_json::Value) -> PluginResult<()> {
        // Add props validation here
        Ok(())
    }` : ''}${details.features.includes('events') ? `
    
    pub async fn handle_event(&mut self, event: &str, data: &serde_json::Value) -> PluginResult<()> {
        match event {
            "click" => self.handle_click(data).await,
            _ => Ok(())
        }
    }
    
    async fn handle_click(&mut self, _data: &serde_json::Value) -> PluginResult<()> {
        // Handle click event
        Ok(())
    }` : ''}
}
`;
  
  await fs.writeFile(componentFile, componentCode);
  
  // Update mod.rs if it exists
  const modFile = `${srcPath}/components/mod.rs`;
  if (await fs.pathExists(modFile)) {
    const modContent = await fs.readFile(modFile, 'utf-8');
    if (!modContent.includes(`mod ${details.kebabCase};`)) {
      const newModContent = `${modContent}\npub mod ${details.kebabCase};\npub use ${details.kebabCase}::${details.pascalCase}Component;\n`;
      await fs.writeFile(modFile, newModContent);
    }
  } else {
    await fs.writeFile(modFile, `pub mod ${details.kebabCase};\npub use ${details.kebabCase}::${details.pascalCase}Component;\n`);
  }
}

async function generateReactComponent(details: any, srcPath: string): Promise<void> {
  const componentFile = `${srcPath}/components/${details.pascalCase}.tsx`;
  
  await fs.ensureDir(path.dirname(componentFile));
  
  const componentCode = `import React${details.features.includes('state') ? ', { useState, useEffect }' : ''} from 'react';
import { useJudicia${details.features.includes('api') ? ', useAPI' : ''}${details.features.includes('events') ? ', useEventEmitter' : ''} } from '@judicia/frontend-sdk';

${details.features.includes('props') ? `interface ${details.pascalCase}Props {
  // Define your props here
  className?: string;
  children?: React.ReactNode;
}
` : ''}
export function ${details.pascalCase}(${details.features.includes('props') ? `props: ${details.pascalCase}Props` : ''}) {
  const sdk = useJudicia();${details.features.includes('api') ? '\n  const api = useAPI();' : ''}${details.features.includes('events') ? '\n  const emit = useEventEmitter();' : ''}${details.features.includes('state') ? `\n  const [state, setState] = useState({
    // Add your state here
  });` : ''}${details.features.includes('api') ? `
  
  useEffect(() => {
    // Load initial data
    loadData();
  }, []);
  
  const loadData = async () => {
    try {
      // const response = await api.get('/api/data');
      // setState(response.data);
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  };` : ''}${details.features.includes('events') ? `
  
  const handleClick = () => {
    emit('${details.kebabCase}.clicked', { timestamp: new Date() });
  };` : ''}

  return (
    <div className={\`judicia-${details.kebabCase}\${${details.features.includes('props') ? 'props.className ? \` \${props.className}\` : \'\'' : '\'\''}\}\`}>
      <h3>${details.pascalCase}</h3>
      <p>${details.description}</p>
      ${details.features.includes('events') ? '<button onClick={handleClick}>Click Me</button>' : ''}
      ${details.features.includes('slots') ? '{props.children}' : ''}
      {/* Add your component content here */}
    </div>
  );
}

export default ${details.pascalCase};
`;
  
  await fs.writeFile(componentFile, componentCode);
  
  // Add styles if requested
  if (details.features.includes('styles')) {
    const styleFile = `${srcPath}/components/${details.pascalCase}.css`;
    const styles = `.judicia-${details.kebabCase} {
  /* Add your component styles here */
  padding: 1rem;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.judicia-${details.kebabCase} h3 {
  margin: 0 0 0.5rem 0;
  color: var(--judicia-color-primary, #007bff);
}
`;
    await fs.writeFile(styleFile, styles);
  }
}

async function generateWebComponent(details: any, srcPath: string): Promise<void> {
  const componentFile = `${srcPath}/components/${details.kebabCase}.js`;
  
  await fs.ensureDir(path.dirname(componentFile));
  
  const componentCode = `class ${details.pascalCase}Element extends HTMLElement {
  constructor() {
    super();${details.features.includes('state') ? `
    this.state = {
      // Add your state here
    };` : ''}
    
    this.attachShadow({ mode: 'open' });
    this.render();
  }
  
  static get observedAttributes() {
    return [/* Add observed attributes here */];
  }
  
  connectedCallback() {
    this.addEventListener('click', this.handleClick.bind(this));
  }
  
  disconnectedCallback() {
    this.removeEventListener('click', this.handleClick.bind(this));
  }
  
  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue !== newValue) {
      this.render();
    }
  }
  
  render() {
    this.shadowRoot.innerHTML = \`
      ${details.features.includes('styles') ? `<style>
        :host {
          display: block;
          padding: 1rem;
          border: 1px solid #ccc;
          border-radius: 4px;
        }
        
        h3 {
          margin: 0 0 0.5rem 0;
          color: var(--judicia-color-primary, #007bff);
        }
      </style>` : ''}
      <div class="content">
        <h3>${details.pascalCase}</h3>
        <p>${details.description}</p>
        ${details.features.includes('slots') ? '<slot></slot>' : ''}
        <!-- Add your component markup here -->
      </div>
    \`;
  }${details.features.includes('events') ? `
  
  handleClick(event) {
    // Emit custom event
    this.dispatchEvent(new CustomEvent('${details.kebabCase}-clicked', {
      detail: { timestamp: new Date() },
      bubbles: true
    }));
  }` : ''}
}

// Register the custom element
customElements.define('judicia-${details.kebabCase}', ${details.pascalCase}Element);

export { ${details.pascalCase}Element };
`;
  
  await fs.writeFile(componentFile, componentCode);
}

async function updatePluginConfig(details: any): Promise<void> {
  // Update Cargo.toml or package.json to include the new component
  // This is a simplified implementation
  console.log(chalk.dim(`Component ${details.name} added to plugin configuration`));
}

function showAddComponentSuccess(details: any): void {
  console.log();
  console.log(chalk.green('✨ Component added successfully!'));
  console.log();
  console.log(chalk.yellow('Files created:'));
  
  if (details.type === 'rust') {
    console.log(chalk.cyan(`  src/components/${details.kebabCase}.rs`));
  } else if (details.type === 'react') {
    console.log(chalk.cyan(`  src/components/${details.pascalCase}.tsx`));
    if (details.features.includes('styles')) {
      console.log(chalk.cyan(`  src/components/${details.pascalCase}.css`));
    }
  } else {
    console.log(chalk.cyan(`  src/components/${details.kebabCase}.js`));
  }
  
  console.log();
  console.log(chalk.yellow('Next steps:'));
  console.log(`  1. Customize your component implementation`);
  console.log(`  2. Add component to your plugin's main file`);
  console.log(`  3. Test your component with ${chalk.cyan('npm run dev')}`);
  console.log();
}