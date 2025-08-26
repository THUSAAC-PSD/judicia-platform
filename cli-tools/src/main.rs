use anyhow::Result;
use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::fs;
use std::path::Path;

mod template;

fn main() -> Result<()> {
    let app = Command::new("create-judicia-plugin")
        .version("0.1.0")
        .author("Judicia Team")
        .about("Create a new Judicia plugin")
        .arg(
            Arg::new("name")
                .help("Plugin name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("type")
                .help("Plugin type")
                .long("type")
                .short('t')
                .value_parser(["contest", "problem", "utility"])
                .default_value("utility"),
        )
        .arg(
            Arg::new("template")
                .help("Template to use")
                .long("template")
                .value_parser(["basic", "full", "interactive"])
                .default_value("basic"),
        )
        .arg(
            Arg::new("output")
                .help("Output directory")
                .long("output")
                .short('o')
                .default_value("."),
        );

    let matches = app.get_matches();
    let plugin_name = matches.get_one::<String>("name").unwrap();
    let plugin_type = matches.get_one::<String>("type").unwrap();
    let template_type = matches.get_one::<String>("template").unwrap();
    let output_dir = matches.get_one::<String>("output").unwrap();

    println!("🚀 Creating Judicia plugin: {}", plugin_name);
    println!("   Type: {}", plugin_type);
    println!("   Template: {}", template_type);

    let pb = ProgressBar::new(6);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>2}/{len:2} {msg}")?,
    );

    pb.set_message("Creating project structure...");
    create_project_structure(output_dir, plugin_name)?;
    pb.inc(1);

    pb.set_message("Generating plugin.toml...");
    generate_plugin_toml(output_dir, plugin_name, plugin_name, plugin_type)?;
    pb.inc(1);

    pb.set_message("Generating Cargo.toml...");
    generate_cargo_toml(output_dir, plugin_name)?;
    pb.inc(1);

    pb.set_message("Generating backend code...");
    generate_backend_code(output_dir, plugin_name, template_type)?;
    pb.inc(1);

    pb.set_message("Generating frontend code...");
    generate_frontend_code(output_dir, plugin_name)?;
    pb.inc(1);

    pb.set_message("Generating README...");
    generate_readme(output_dir, plugin_name, plugin_type)?;
    pb.inc(1);

    pb.finish_with_message("✅ Plugin created successfully!");

    println!();
    println!("Next steps:");
    println!("  1. cd {}/{}", output_dir, plugin_name);
    println!("  2. cargo build");
    println!("  3. Edit src/lib.rs to implement your plugin logic");
    println!("  4. Edit frontend/src/index.tsx for UI components");
    
    Ok(())
}

fn create_project_structure(output_dir: &str, plugin_name: &str) -> Result<()> {
    let project_path = Path::new(output_dir).join(plugin_name);
    
    fs::create_dir_all(&project_path)?;
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("frontend/src"))?;
    fs::create_dir_all(project_path.join("tests"))?;
    
    Ok(())
}

fn generate_plugin_toml(output_dir: &str, plugin_name: &str, display_name: &str, plugin_type: &str) -> Result<()> {
    let project_path = Path::new(output_dir).join(plugin_name);
    let toml_content = template::generate_plugin_toml(display_name, plugin_type);
    
    fs::write(project_path.join("plugin.toml"), toml_content)?;
    Ok(())
}

fn generate_cargo_toml(output_dir: &str, plugin_name: &str) -> Result<()> {
    let project_path = Path::new(output_dir).join(plugin_name);
    let cargo_content = template::generate_cargo_toml(plugin_name);
    
    fs::write(project_path.join("Cargo.toml"), cargo_content)?;
    Ok(())
}

fn generate_backend_code(output_dir: &str, plugin_name: &str, template_type: &str) -> Result<()> {
    let project_path = Path::new(output_dir).join(plugin_name);
    let backend_code = template::generate_backend_code(plugin_name, template_type);
    
    fs::write(project_path.join("src/lib.rs"), backend_code)?;
    Ok(())
}

fn generate_frontend_code(output_dir: &str, plugin_name: &str) -> Result<()> {
    let project_path = Path::new(output_dir).join(plugin_name);
    
    // package.json
    let package_json = json!({
        "name": format!("{}-frontend", plugin_name),
        "version": "1.0.0",
        "private": true,
        "scripts": {
            "build": "webpack --mode=production",
            "dev": "webpack --mode=development --watch"
        },
        "dependencies": {
            "react": "^18.2.0",
            "react-dom": "^18.2.0",
            "@judicia/frontend-sdk": "^0.1.0"
        },
        "devDependencies": {
            "@types/react": "^18.2.0",
            "@types/react-dom": "^18.2.0",
            "typescript": "^5.0.0",
            "webpack": "^5.88.0",
            "webpack-cli": "^5.1.0",
            "ts-loader": "^9.4.0"
        }
    });
    
    fs::write(
        project_path.join("frontend/package.json"),
        serde_json::to_string_pretty(&package_json)?,
    )?;
    
    // Frontend React component
    let frontend_code = template::generate_frontend_code(plugin_name);
    fs::write(project_path.join("frontend/src/index.tsx"), frontend_code)?;
    
    // Webpack config
    let webpack_config = template::generate_webpack_config(plugin_name);
    fs::write(project_path.join("frontend/webpack.config.js"), webpack_config)?;
    
    Ok(())
}

fn generate_readme(output_dir: &str, plugin_name: &str, plugin_type: &str) -> Result<()> {
    let project_path = Path::new(output_dir).join(plugin_name);
    let readme_content = template::generate_readme(plugin_name, plugin_type);
    
    fs::write(project_path.join("README.md"), readme_content)?;
    Ok(())
}