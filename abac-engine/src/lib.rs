use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod policy;
pub mod evaluation;
pub mod attributes;

/// Attribute-Based Access Control (ABAC) Engine
/// Replaces simple role-based authorization with fine-grained policies

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    pub subject: Subject,
    pub action: Action,
    pub resource: Resource,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub user_id: Uuid,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub type_name: String,
    pub id: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<AttributeValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub description: String,
    pub effect: Effect,
    pub target: Target,
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Permit,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub subjects: Option<AttributeMatcher>,
    pub actions: Option<AttributeMatcher>,
    pub resources: Option<AttributeMatcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeMatcher {
    pub attribute: String,
    pub operator: MatchOperator,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchOperator {
    Equals,
    NotEquals,
    Contains,
    In,
    GreaterThan,
    LessThan,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
    AttributeComparison(AttributeComparison),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeComparison {
    pub left: AttributePath,
    pub operator: ComparisonOperator,
    pub right: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributePath {
    Subject(String),
    Action(String),
    Resource(String),
    Environment(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Greater,
    Less,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    Permit,
    Deny,
    NotApplicable,
}

pub struct ABACEngine {
    policies: Arc<DashMap<String, Policy>>,
}

impl ABACEngine {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(DashMap::new()),
        }
    }
    
    pub fn add_policy(&self, policy: Policy) {
        self.policies.insert(policy.id.clone(), policy);
    }
    
    pub fn remove_policy(&self, policy_id: &str) {
        self.policies.remove(policy_id);
    }
    
    pub fn evaluate(&self, request: &AccessRequest) -> Result<Decision> {
        let mut applicable_policies = Vec::new();
        
        // Find applicable policies
        for policy_ref in self.policies.iter() {
            let policy = policy_ref.value();
            if self.is_policy_applicable(policy, request)? {
                applicable_policies.push(policy.clone());
            }
        }
        
        if applicable_policies.is_empty() {
            return Ok(Decision::NotApplicable);
        }
        
        // Evaluate policies (deny-overrides combining algorithm)
        for policy in &applicable_policies {
            if let Some(condition) = &policy.condition {
                if !self.evaluate_condition(condition, request)? {
                    continue;
                }
            }
            
            match policy.effect {
                Effect::Deny => return Ok(Decision::Deny),
                Effect::Permit => continue, // Keep checking for deny
            }
        }
        
        // If no deny found, check for permits
        for policy in &applicable_policies {
            if matches!(policy.effect, Effect::Permit) {
                return Ok(Decision::Permit);
            }
        }
        
        Ok(Decision::NotApplicable)
    }
    
    fn is_policy_applicable(&self, policy: &Policy, request: &AccessRequest) -> Result<bool> {
        // Check if the policy target matches the request
        if let Some(subject_matcher) = &policy.target.subjects {
            if !self.matches_attribute(&request.subject.attributes, subject_matcher)? {
                return Ok(false);
            }
        }
        
        if let Some(action_matcher) = &policy.target.actions {
            if !self.matches_attribute(&request.action.attributes, action_matcher)? {
                return Ok(false);
            }
        }
        
        if let Some(resource_matcher) = &policy.target.resources {
            if !self.matches_attribute(&request.resource.attributes, resource_matcher)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    fn matches_attribute(
        &self,
        attributes: &HashMap<String, AttributeValue>,
        matcher: &AttributeMatcher,
    ) -> Result<bool> {
        let attribute_value = attributes.get(&matcher.attribute);
        
        match attribute_value {
            Some(value) => self.compare_values(value, &matcher.operator, &matcher.value),
            None => Ok(false),
        }
    }
    
    fn compare_values(
        &self,
        left: &AttributeValue,
        operator: &MatchOperator,
        right: &AttributeValue,
    ) -> Result<bool> {
        match operator {
            MatchOperator::Equals => Ok(left == right),
            MatchOperator::NotEquals => Ok(left != right),
            MatchOperator::Contains => {
                match (left, right) {
                    (AttributeValue::String(l), AttributeValue::String(r)) => Ok(l.contains(r)),
                    (AttributeValue::Array(l), r) => Ok(l.contains(r)),
                    _ => Ok(false),
                }
            }
            MatchOperator::In => {
                match (left, right) {
                    (l, AttributeValue::Array(r)) => Ok(r.contains(l)),
                    _ => Ok(false),
                }
            }
            MatchOperator::GreaterThan => {
                match (left, right) {
                    (AttributeValue::Number(l), AttributeValue::Number(r)) => Ok(l > r),
                    _ => Ok(false),
                }
            }
            MatchOperator::LessThan => {
                match (left, right) {
                    (AttributeValue::Number(l), AttributeValue::Number(r)) => Ok(l < r),
                    _ => Ok(false),
                }
            }
            MatchOperator::Regex => {
                match (left, right) {
                    (AttributeValue::String(l), AttributeValue::String(r)) => {
                        match regex::Regex::new(r) {
                            Ok(re) => Ok(re.is_match(l)),
                            Err(_) => Ok(false),
                        }
                    }
                    _ => Ok(false),
                }
            }
        }
    }
    
    fn evaluate_condition(&self, condition: &Condition, request: &AccessRequest) -> Result<bool> {
        match condition {
            Condition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond, request)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, request)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not(condition) => {
                Ok(!self.evaluate_condition(condition, request)?)
            }
            Condition::AttributeComparison(comparison) => {
                let left_value = self.get_attribute_value(&comparison.left, request)?;
                match left_value {
                    Some(value) => self.compare_attribute_values(&value, &comparison.operator, &comparison.right),
                    None => Ok(false),
                }
            }
        }
    }
    
    fn get_attribute_value(&self, path: &AttributePath, request: &AccessRequest) -> Result<Option<AttributeValue>> {
        match path {
            AttributePath::Subject(attr) => Ok(request.subject.attributes.get(attr).cloned()),
            AttributePath::Action(attr) => Ok(request.action.attributes.get(attr).cloned()),
            AttributePath::Resource(attr) => Ok(request.resource.attributes.get(attr).cloned()),
            AttributePath::Environment(attr) => Ok(request.environment.attributes.get(attr).cloned()),
        }
    }
    
    fn compare_attribute_values(
        &self,
        left: &AttributeValue,
        operator: &ComparisonOperator,
        right: &AttributeValue,
    ) -> Result<bool> {
        match operator {
            ComparisonOperator::Equal => Ok(left == right),
            ComparisonOperator::NotEqual => Ok(left != right),
            ComparisonOperator::Greater => {
                match (left, right) {
                    (AttributeValue::Number(l), AttributeValue::Number(r)) => Ok(l > r),
                    _ => Ok(false),
                }
            }
            ComparisonOperator::Less => {
                match (left, right) {
                    (AttributeValue::Number(l), AttributeValue::Number(r)) => Ok(l < r),
                    _ => Ok(false),
                }
            }
            ComparisonOperator::Contains => {
                match (left, right) {
                    (AttributeValue::String(l), AttributeValue::String(r)) => Ok(l.contains(r)),
                    (AttributeValue::Array(l), r) => Ok(l.contains(r)),
                    _ => Ok(false),
                }
            }
            ComparisonOperator::StartsWith => {
                match (left, right) {
                    (AttributeValue::String(l), AttributeValue::String(r)) => Ok(l.starts_with(r)),
                    _ => Ok(false),
                }
            }
            ComparisonOperator::EndsWith => {
                match (left, right) {
                    (AttributeValue::String(l), AttributeValue::String(r)) => Ok(l.ends_with(r)),
                    _ => Ok(false),
                }
            }
        }
    }
}

/// Convenience methods for creating common policies
impl ABACEngine {
    /// Create a policy that allows users with specific roles to access resources
    pub fn create_role_based_policy(
        policy_id: &str,
        description: &str,
        allowed_roles: Vec<String>,
        resource_type: &str,
        action: &str,
    ) -> Policy {
        Policy {
            id: policy_id.to_string(),
            description: description.to_string(),
            effect: Effect::Permit,
            target: Target {
                subjects: Some(AttributeMatcher {
                    attribute: "roles".to_string(),
                    operator: MatchOperator::Contains,
                    value: AttributeValue::Array(
                        allowed_roles.into_iter().map(AttributeValue::String).collect()
                    ),
                }),
                actions: Some(AttributeMatcher {
                    attribute: "name".to_string(),
                    operator: MatchOperator::Equals,
                    value: AttributeValue::String(action.to_string()),
                }),
                resources: Some(AttributeMatcher {
                    attribute: "type".to_string(),
                    operator: MatchOperator::Equals,
                    value: AttributeValue::String(resource_type.to_string()),
                }),
            },
            condition: None,
        }
    }
    
    /// Create a time-based policy that only allows access during certain hours
    pub fn create_time_based_policy(
        policy_id: &str,
        description: &str,
        start_hour: u32,
        end_hour: u32,
    ) -> Policy {
        Policy {
            id: policy_id.to_string(),
            description: description.to_string(),
            effect: Effect::Permit,
            target: Target {
                subjects: None,
                actions: None,
                resources: None,
            },
            condition: Some(Condition::And(vec![
                Condition::AttributeComparison(AttributeComparison {
                    left: AttributePath::Environment("hour".to_string()),
                    operator: ComparisonOperator::Greater,
                    right: AttributeValue::Number(start_hour as f64),
                }),
                Condition::AttributeComparison(AttributeComparison {
                    left: AttributePath::Environment("hour".to_string()),
                    operator: ComparisonOperator::Less,
                    right: AttributeValue::Number(end_hour as f64),
                }),
            ])),
        }
    }
    
    /// Create a plugin-specific policy
    pub fn create_plugin_policy(
        policy_id: &str,
        description: &str,
        plugin_id: &str,
        required_permission: &str,
    ) -> Policy {
        Policy {
            id: policy_id.to_string(),
            description: description.to_string(),
            effect: Effect::Permit,
            target: Target {
                subjects: Some(AttributeMatcher {
                    attribute: "permissions".to_string(),
                    operator: MatchOperator::Contains,
                    value: AttributeValue::String(required_permission.to_string()),
                }),
                actions: None,
                resources: Some(AttributeMatcher {
                    attribute: "plugin_id".to_string(),
                    operator: MatchOperator::Equals,
                    value: AttributeValue::String(plugin_id.to_string()),
                }),
            },
            condition: None,
        }
    }
}