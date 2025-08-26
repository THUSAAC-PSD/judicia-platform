use crate::{JPPParser, JPPIntegration};

#[cfg(test)]
mod tests {
    use super::*;
    
    const EXAMPLE_STANDARD_PROBLEM: &str = r#"
id: "A001"
title: "Simple Addition"
time_limit_ms: 1000
memory_limit_kb: 262144

judging:
  type: "judicia/standard@1.0"

statement:
  format: markdown
  source: "statement.md"
  language: "en"

test_cases:
  input_pattern: "tests/*.in"
  output_pattern: "tests/*.out"

metadata:
  difficulty: "easy"
  tags: ["math", "implementation"]
"#;

    const EXAMPLE_INTERACTIVE_PROBLEM: &str = r#"
id: "B001"
title: "Guess the Number"
time_limit_ms: 2000
memory_limit_kb: 262144

judging:
  type: "judicia/interactive@1.0"
  interactor:
    source: "judge/interactor.cpp"
    language: "cpp.g++17"

metadata:
  difficulty: "medium"
  tags: ["interactive", "binary-search"]
"#;
    
    #[test]
    fn test_parse_standard_problem() {
        let problem = JPPParser::parse_from_string(EXAMPLE_STANDARD_PROBLEM).unwrap();
        assert_eq!(problem.id, "A001");
        assert_eq!(problem.title, "Simple Addition");
        assert_eq!(problem.time_limit_ms, 1000);
        assert_eq!(problem.judging.plugin_type, "judicia/standard@1.0");
    }
    
    #[test]
    fn test_parse_interactive_problem() {
        let problem = JPPParser::parse_from_string(EXAMPLE_INTERACTIVE_PROBLEM).unwrap();
        assert_eq!(problem.id, "B001");
        assert_eq!(problem.judging.plugin_type, "judicia/interactive@1.0");
        assert!(problem.judging.config.contains_key("interactor"));
    }
    
    #[test]
    fn test_plugin_type_parsing() {
        let (name, version) = JPPParser::parse_plugin_type("judicia/standard@1.0").unwrap();
        assert_eq!(name, "judicia/standard");
        assert_eq!(version, "1.0");
        
        let (name, version) = JPPParser::parse_plugin_type("judicia/interactive").unwrap();
        assert_eq!(name, "judicia/interactive");
        assert_eq!(version, "latest");
    }
    
    #[test] 
    fn test_plugin_dependencies() {
        let problem = JPPParser::parse_from_string(EXAMPLE_STANDARD_PROBLEM).unwrap();
        let deps = JPPIntegration::get_plugin_dependencies(&problem).unwrap();
        assert!(deps.contains(&"judicia/standard@1.0".to_string()));
    }
    
    #[test]
    fn test_plugin_compatibility() {
        let problem = JPPParser::parse_from_string(EXAMPLE_STANDARD_PROBLEM).unwrap();
        let available_plugins = vec![
            "judicia/standard@1.0".to_string(),
            "judicia/interactive@1.0".to_string(),
        ];
        
        assert!(JPPIntegration::check_plugin_compatibility(&problem, &available_plugins).unwrap());
        
        let available_plugins = vec!["judicia/interactive@1.0".to_string()];
        assert!(!JPPIntegration::check_plugin_compatibility(&problem, &available_plugins).unwrap());
    }
    
    #[test]
    fn test_evaluation_params_conversion() {
        let problem = JPPParser::parse_from_string(EXAMPLE_STANDARD_PROBLEM).unwrap();
        let params = JPPIntegration::to_evaluation_params(&problem, "print('hello')", "python3").unwrap();
        
        assert_eq!(params["problem_id"], "A001");
        assert_eq!(params["plugin_type"], "judicia/standard");
        assert_eq!(params["plugin_version"], "1.0");
        assert_eq!(params["language_id"], "python3");
        assert_eq!(params["source_code"], "print('hello')");
    }
}