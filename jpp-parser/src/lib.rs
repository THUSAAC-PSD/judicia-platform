use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod problem;
pub mod validation;

#[cfg(test)]
mod tests;

/// Judicia Problem Package (JPP) format
/// This replaces the traditional database-driven problem definition
/// with a YAML-based, plugin-extensible format

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JPProblem {
    pub id: String,
    pub title: String,
    pub time_limit_ms: u32,
    pub memory_limit_kb: u32,
    pub judging: JudgingConfig,
    pub statement: Option<StatementConfig>,
    pub test_cases: Option<TestCasesConfig>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgingConfig {
    #[serde(rename = "type")]
    pub plugin_type: String, // e.g., "judicia/standard@1.0", "judicia/interactive@1.0"
    #[serde(flatten)]
    pub config: HashMap<String, serde_json::Value>, // Plugin-specific configuration
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementConfig {
    pub format: StatementFormat,
    pub source: String, // File path or inline content
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatementFormat {
    Markdown,
    Html,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCasesConfig {
    pub input_pattern: Option<String>,  // e.g., "tests/*.in"
    pub output_pattern: Option<String>, // e.g., "tests/*.out"
    pub generator: Option<GeneratorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    pub source: String,
    pub language: String,
    pub args: Option<Vec<String>>,
}

pub struct JPPParser;

impl JPPParser {
    pub fn parse_from_path(problem_yaml_path: &Path) -> Result<JPProblem> {
        let content = std::fs::read_to_string(problem_yaml_path)?;
        Self::parse_from_string(&content)
    }
    
    pub fn parse_from_string(yaml_content: &str) -> Result<JPProblem> {
        let problem: JPProblem = serde_yaml::from_str(yaml_content)?;
        validation::validate_problem(&problem)?;
        Ok(problem)
    }
    
    pub fn serialize_to_string(problem: &JPProblem) -> Result<String> {
        Ok(serde_yaml::to_string(problem)?)
    }
    
    /// Extract plugin type and version from plugin_type string
    /// e.g., "judicia/standard@1.0" -> ("judicia/standard", "1.0")
    pub fn parse_plugin_type(plugin_type: &str) -> Result<(String, String)> {
        if let Some(at_pos) = plugin_type.rfind('@') {
            let name = plugin_type[..at_pos].to_string();
            let version = plugin_type[at_pos + 1..].to_string();
            Ok((name, version))
        } else {
            Ok((plugin_type.to_string(), "latest".to_string()))
        }
    }
    
    /// Get plugin-specific configuration for the judging type
    pub fn get_judging_config(problem: &JPProblem) -> &HashMap<String, serde_json::Value> {
        &problem.judging.config
    }
    
    /// Resolve test case paths from patterns
    pub fn resolve_test_cases(problem: &JPProblem, base_path: &Path) -> Result<Vec<(String, String)>> {
        let test_cases_config = problem.test_cases.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No test cases configuration found"))?;
        
        let input_pattern = test_cases_config.input_pattern.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No input pattern specified"))?;
        let output_pattern = test_cases_config.output_pattern.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No output pattern specified"))?;
        
        let mut test_cases = Vec::new();
        
        // Find input files using glob pattern
        let input_glob = base_path.join(input_pattern);
        let input_files = glob::glob(&input_glob.to_string_lossy())?;
        
        for input_path in input_files {
            let input_path = input_path?;
            let file_stem = input_path.file_stem()
                .ok_or_else(|| anyhow::anyhow!("Invalid input file: {:?}", input_path))?
                .to_string_lossy();
            
            // Generate corresponding output path
            let output_path = base_path.join(output_pattern.replace("*", &file_stem));
            
            if output_path.exists() {
                test_cases.push((
                    input_path.to_string_lossy().to_string(),
                    output_path.to_string_lossy().to_string(),
                ));
            } else {
                tracing::warn!("Missing output file for input: {:?}", input_path);
            }
        }
        
        test_cases.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(test_cases)
    }
    
    /// Validate plugin configuration against known plugin types
    pub fn validate_plugin_config(problem: &JPProblem) -> Result<()> {
        let (plugin_name, _version) = Self::parse_plugin_type(&problem.judging.plugin_type)?;
        
        match plugin_name.as_str() {
            "judicia/standard" => {
                // Standard problems don't need special configuration
                Ok(())
            }
            "judicia/interactive" => {
                let config = &problem.judging.config;
                if !config.contains_key("interactor") {
                    return Err(anyhow::anyhow!("Interactive problems must specify an interactor"));
                }
                
                let interactor = config.get("interactor").unwrap();
                if !interactor.is_object() || !interactor.get("source").is_some() {
                    return Err(anyhow::anyhow!("Interactor must specify a source file"));
                }
                
                Ok(())
            }
            "judicia/special-judge" => {
                let config = &problem.judging.config;
                if !config.contains_key("checker") {
                    return Err(anyhow::anyhow!("Special judge problems must specify a checker"));
                }
                
                let checker = config.get("checker").unwrap();
                if !checker.is_object() || !checker.get("source").is_some() {
                    return Err(anyhow::anyhow!("Checker must specify a source file"));
                }
                
                Ok(())
            }
            _ => {
                tracing::info!("Unknown plugin type, skipping validation: {}", plugin_name);
                Ok(())
            }
        }
    }
    
    /// Extract problem package to directory with validation
    pub fn extract_package(package_path: &Path, target_dir: &Path) -> Result<JPProblem> {
        if !package_path.exists() {
            return Err(anyhow::anyhow!("Package file not found: {:?}", package_path));
        }
        
        // For now, assume package is a directory with problem.yaml
        // In full implementation, this would handle zip files, tar.gz, etc.
        let problem_yaml = package_path.join("problem.yaml");
        if !problem_yaml.exists() {
            return Err(anyhow::anyhow!("problem.yaml not found in package: {:?}", package_path));
        }
        
        let problem = Self::parse_from_path(&problem_yaml)?;
        
        // Validate plugin configuration
        Self::validate_plugin_config(&problem)?;
        
        // Copy package contents to target directory
        if package_path != target_dir {
            copy_dir_all(package_path, target_dir)?;
        }
        
        Ok(problem)
    }
}

/// Integration utilities for the plugin system
pub struct JPPIntegration;

impl JPPIntegration {
    /// Convert a JPP problem to evaluation job parameters
    pub fn to_evaluation_params(problem: &JPProblem, submission_code: &str, language_id: &str) -> Result<serde_json::Value> {
        let (plugin_name, plugin_version) = JPPParser::parse_plugin_type(&problem.judging.plugin_type)?;
        
        let mut params = serde_json::json!({
            "problem_id": problem.id,
            "title": problem.title,
            "time_limit_ms": problem.time_limit_ms,
            "memory_limit_kb": problem.memory_limit_kb,
            "plugin_type": plugin_name,
            "plugin_version": plugin_version,
            "language_id": language_id,
            "source_code": submission_code,
            "judging_config": problem.judging.config
        });
        
        // Add test case information if available
        if let Some(test_cases) = &problem.test_cases {
            params["test_cases"] = serde_json::json!({
                "input_pattern": test_cases.input_pattern,
                "output_pattern": test_cases.output_pattern,
                "generator": test_cases.generator
            });
        }
        
        // Add metadata
        if let Some(metadata) = &problem.metadata {
            params["metadata"] = serde_json::to_value(metadata)?;
        }
        
        Ok(params)
    }
    
    /// Extract required plugin dependencies from a problem
    pub fn get_plugin_dependencies(problem: &JPProblem) -> Result<Vec<String>> {
        let mut dependencies = Vec::new();
        
        // Main judging plugin
        dependencies.push(problem.judging.plugin_type.clone());
        
        // Check for additional plugin dependencies in config
        if let Some(deps) = problem.judging.config.get("depends_on") {
            if let Some(deps_array) = deps.as_array() {
                for dep in deps_array {
                    if let Some(dep_str) = dep.as_str() {
                        dependencies.push(dep_str.to_string());
                    }
                }
            }
        }
        
        // Check metadata for plugin dependencies
        if let Some(metadata) = &problem.metadata {
            if let Some(plugins) = metadata.get("required_plugins") {
                if let Some(plugins_array) = plugins.as_array() {
                    for plugin in plugins_array {
                        if let Some(plugin_str) = plugin.as_str() {
                            dependencies.push(plugin_str.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Check if a problem is compatible with available plugins
    pub fn check_plugin_compatibility(problem: &JPProblem, available_plugins: &[String]) -> Result<bool> {
        let dependencies = Self::get_plugin_dependencies(problem)?;
        
        for dep in dependencies {
            let (plugin_name, _version) = JPPParser::parse_plugin_type(&dep)?;
            
            // Check if plugin is available (simplified version matching)
            let available = available_plugins.iter().any(|available| {
                let (available_name, _) = JPPParser::parse_plugin_type(available).unwrap_or_default();
                available_name == plugin_name
            });
            
            if !available {
                tracing::warn!("Required plugin not available: {}", plugin_name);
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

/// Helper function to recursively copy directories
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}