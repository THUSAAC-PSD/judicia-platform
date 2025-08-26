use crate::{JPProblem, JudgingConfig, StatementConfig, TestCasesConfig};
use anyhow::Result;
use std::collections::HashMap;

/// Example JPP problem definitions for different problem types

impl JPProblem {
    /// Create a standard IOI-style problem
    pub fn new_standard(id: String, title: String) -> Self {
        Self {
            id,
            title,
            time_limit_ms: 1000,
            memory_limit_kb: 256 * 1024,
            judging: JudgingConfig {
                plugin_type: "judicia/standard@1.0".to_string(),
                config: HashMap::new(),
            },
            statement: None,
            test_cases: None,
            metadata: None,
        }
    }
    
    /// Create an interactive problem
    pub fn new_interactive(id: String, title: String, interactor_source: String) -> Self {
        let mut judging_config = HashMap::new();
        judging_config.insert(
            "interactor".to_string(),
            serde_json::json!({
                "source": interactor_source,
                "language": "cpp.g++17"
            })
        );
        
        Self {
            id,
            title,
            time_limit_ms: 1000,
            memory_limit_kb: 256 * 1024,
            judging: JudgingConfig {
                plugin_type: "judicia/interactive@1.0".to_string(),
                config: judging_config,
            },
            statement: None,
            test_cases: None,
            metadata: None,
        }
    }
    
    /// Create a special judge problem
    pub fn new_special_judge(id: String, title: String, checker_source: String) -> Self {
        let mut judging_config = HashMap::new();
        judging_config.insert(
            "checker".to_string(),
            serde_json::json!({
                "source": checker_source,
                "language": "cpp.g++17"
            })
        );
        
        Self {
            id,
            title,
            time_limit_ms: 1000,
            memory_limit_kb: 256 * 1024,
            judging: JudgingConfig {
                plugin_type: "judicia/special-judge@1.0".to_string(),
                config: judging_config,
            },
            statement: None,
            test_cases: None,
            metadata: None,
        }
    }
    
    pub fn with_statement(mut self, source: String) -> Self {
        self.statement = Some(StatementConfig {
            format: crate::StatementFormat::Markdown,
            source,
            language: Some("en".to_string()),
        });
        self
    }
    
    pub fn with_test_cases(mut self, input_pattern: String, output_pattern: String) -> Self {
        self.test_cases = Some(TestCasesConfig {
            input_pattern: Some(input_pattern),
            output_pattern: Some(output_pattern),
            generator: None,
        });
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        self.metadata.as_mut().unwrap().insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_standard_problem() {
        let problem = JPProblem::new_standard("A001".to_string(), "Test Problem".to_string());
        assert_eq!(problem.id, "A001");
        assert_eq!(problem.title, "Test Problem");
        assert_eq!(problem.judging.plugin_type, "judicia/standard@1.0");
    }
    
    #[test]
    fn test_create_interactive_problem() {
        let problem = JPProblem::new_interactive(
            "B001".to_string(),
            "Interactive Problem".to_string(),
            "judge/interactor.cpp".to_string(),
        );
        assert_eq!(problem.judging.plugin_type, "judicia/interactive@1.0");
        assert!(problem.judging.config.contains_key("interactor"));
    }
}