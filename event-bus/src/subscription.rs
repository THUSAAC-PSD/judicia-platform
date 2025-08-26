use anyhow::Result;

/// Pattern matching utilities for event subscriptions
pub struct EventPatternMatcher;

impl EventPatternMatcher {
    /// Check if an event type matches a subscription pattern
    /// Supports wildcards like "submission.*" or "contest.problem.*"
    pub fn matches(pattern: &str, event_type: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let event_parts: Vec<&str> = event_type.split('.').collect();
        
        Self::match_parts(&pattern_parts, &event_parts)
    }
    
    fn match_parts(pattern_parts: &[&str], event_parts: &[&str]) -> bool {
        let mut p_idx = 0;
        let mut e_idx = 0;
        
        while p_idx < pattern_parts.len() && e_idx < event_parts.len() {
            let pattern_part = pattern_parts[p_idx];
            let event_part = event_parts[e_idx];
            
            match pattern_part {
                "*" => {
                    // Single wildcard matches exactly one part
                    p_idx += 1;
                    e_idx += 1;
                }
                "**" => {
                    // Double wildcard matches zero or more parts
                    if p_idx == pattern_parts.len() - 1 {
                        // ** at the end matches everything remaining
                        return true;
                    }
                    
                    // Try to match the next pattern part
                    let next_pattern = pattern_parts[p_idx + 1];
                    while e_idx < event_parts.len() {
                        if event_parts[e_idx] == next_pattern {
                            p_idx += 2; // Skip ** and the matched part
                            e_idx += 1;
                            break;
                        }
                        e_idx += 1;
                    }
                    
                    if e_idx == event_parts.len() && event_parts[e_idx - 1] != next_pattern {
                        return false;
                    }
                }
                part if part == event_part => {
                    // Exact match
                    p_idx += 1;
                    e_idx += 1;
                }
                _ => {
                    // No match
                    return false;
                }
            }
        }
        
        // Check if we've consumed all parts
        p_idx == pattern_parts.len() && e_idx == event_parts.len()
    }
    
    pub fn validate_pattern(pattern: &str) -> Result<()> {
        if pattern.is_empty() {
            return Err(anyhow::anyhow!("Pattern cannot be empty"));
        }
        
        let parts: Vec<&str> = pattern.split('.').collect();
        
        for part in parts {
            if part.is_empty() {
                return Err(anyhow::anyhow!("Pattern parts cannot be empty"));
            }
            
            // Only allow alphanumeric, underscore, and wildcards
            if !part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '*') {
                return Err(anyhow::anyhow!("Invalid characters in pattern: {}", part));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exact_match() {
        assert!(EventPatternMatcher::matches("submission.judged", "submission.judged"));
        assert!(!EventPatternMatcher::matches("submission.judged", "submission.created"));
    }
    
    #[test]
    fn test_single_wildcard() {
        assert!(EventPatternMatcher::matches("submission.*", "submission.judged"));
        assert!(EventPatternMatcher::matches("submission.*", "submission.created"));
        assert!(!EventPatternMatcher::matches("submission.*", "contest.started"));
    }
    
    #[test]
    fn test_global_wildcard() {
        assert!(EventPatternMatcher::matches("*", "submission.judged"));
        assert!(EventPatternMatcher::matches("*", "contest.started"));
    }
    
    #[test]
    fn test_pattern_validation() {
        assert!(EventPatternMatcher::validate_pattern("submission.*").is_ok());
        assert!(EventPatternMatcher::validate_pattern("contest.problem.first_blood").is_ok());
        assert!(EventPatternMatcher::validate_pattern("").is_err());
        assert!(EventPatternMatcher::validate_pattern("invalid.pattern.with-dash").is_err());
    }
}