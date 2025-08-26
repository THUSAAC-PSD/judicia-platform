use crate::JPProblem;
use anyhow::Result;

pub fn validate_problem(problem: &JPProblem) -> Result<()> {
    // Validate required fields
    if problem.id.is_empty() {
        return Err(anyhow::anyhow!("Problem ID cannot be empty"));
    }
    
    if problem.title.is_empty() {
        return Err(anyhow::anyhow!("Problem title cannot be empty"));
    }
    
    // Validate limits
    if problem.time_limit_ms == 0 {
        return Err(anyhow::anyhow!("Time limit must be greater than 0"));
    }
    
    if problem.time_limit_ms > 60000 {
        return Err(anyhow::anyhow!("Time limit cannot exceed 60 seconds"));
    }
    
    if problem.memory_limit_kb == 0 {
        return Err(anyhow::anyhow!("Memory limit must be greater than 0"));
    }
    
    if problem.memory_limit_kb > 2048 * 1024 {
        return Err(anyhow::anyhow!("Memory limit cannot exceed 2GB"));
    }
    
    // Validate plugin type format
    validate_plugin_type(&problem.judging.plugin_type)?;
    
    // Validate problem ID format (alphanumeric + underscore only)
    if !problem.id.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow::anyhow!("Problem ID can only contain alphanumeric characters and underscores"));
    }
    
    Ok(())
}

fn validate_plugin_type(plugin_type: &str) -> Result<()> {
    if plugin_type.is_empty() {
        return Err(anyhow::anyhow!("Plugin type cannot be empty"));
    }
    
    // Check format: namespace/name@version or namespace/name
    let parts: Vec<&str> = plugin_type.split('@').collect();
    if parts.len() > 2 {
        return Err(anyhow::anyhow!("Invalid plugin type format: {}", plugin_type));
    }
    
    let name_part = parts[0];
    if !name_part.contains('/') {
        return Err(anyhow::anyhow!("Plugin type must include namespace: {}", plugin_type));
    }
    
    let namespace_name: Vec<&str> = name_part.split('/').collect();
    if namespace_name.len() != 2 {
        return Err(anyhow::anyhow!("Plugin type must be in format 'namespace/name': {}", plugin_type));
    }
    
    let namespace = namespace_name[0];
    let name = namespace_name[1];
    
    if namespace.is_empty() || name.is_empty() {
        return Err(anyhow::anyhow!("Namespace and name cannot be empty: {}", plugin_type));
    }
    
    // Validate version if present
    if parts.len() == 2 {
        let version = parts[1];
        if version.is_empty() {
            return Err(anyhow::anyhow!("Version cannot be empty: {}", plugin_type));
        }
        
        // Basic semver validation
        if !version.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err(anyhow::anyhow!("Invalid version format: {}", version));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JPProblem;
    
    #[test]
    fn test_valid_problem() {
        let problem = JPProblem::new_standard("A001".to_string(), "Valid Problem".to_string());
        assert!(validate_problem(&problem).is_ok());
    }
    
    #[test]
    fn test_empty_id() {
        let mut problem = JPProblem::new_standard("".to_string(), "Problem".to_string());
        assert!(validate_problem(&problem).is_err());
    }
    
    #[test]
    fn test_empty_title() {
        let problem = JPProblem::new_standard("A001".to_string(), "".to_string());
        assert!(validate_problem(&problem).is_err());
    }
    
    #[test]
    fn test_invalid_plugin_type() {
        assert!(validate_plugin_type("invalid").is_err());
        assert!(validate_plugin_type("judicia/").is_err());
        assert!(validate_plugin_type("/standard").is_err());
        assert!(validate_plugin_type("judicia/standard@").is_err());
    }
    
    #[test]
    fn test_valid_plugin_type() {
        assert!(validate_plugin_type("judicia/standard@1.0").is_ok());
        assert!(validate_plugin_type("judicia/standard").is_ok());
        assert!(validate_plugin_type("custom/interactive@2.1.0").is_ok());
    }
}