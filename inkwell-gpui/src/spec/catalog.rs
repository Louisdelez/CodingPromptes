//! Extension catalog — community extensions registry.
//! Inspired by SpecKit's catalog.community.json.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub ext_type: String,
    pub url: Option<String>,
}

/// Built-in community catalog (subset of SpecKit's 70+ extensions)
pub fn community_catalog() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry {
            id: "bugfix-workflow".into(),
            name: "Bugfix Workflow".into(),
            description: "Structured bug fixing with diagnosis and regression prevention".into(),
            version: "1.0.0".into(), author: "SpecKit".into(),
            ext_type: "process".into(), url: None,
        },
        CatalogEntry {
            id: "cleanup".into(),
            name: "Cleanup".into(),
            description: "Post-implementation quality gate and cleanup".into(),
            version: "1.0.0".into(), author: "SpecKit".into(),
            ext_type: "code".into(), url: None,
        },
        CatalogEntry {
            id: "checkpoint".into(),
            name: "Checkpoint".into(),
            description: "Intermediate commit management during implementation".into(),
            version: "1.0.0".into(), author: "SpecKit".into(),
            ext_type: "integration".into(), url: None,
        },
        CatalogEntry {
            id: "docguard".into(),
            name: "DocGuard".into(),
            description: "Constitution-Driven Development enforcement".into(),
            version: "1.0.0".into(), author: "SpecKit".into(),
            ext_type: "docs".into(), url: None,
        },
        CatalogEntry {
            id: "aide".into(),
            name: "AI-Driven Engineering".into(),
            description: "7-step AI-guided development workflow".into(),
            version: "1.0.0".into(), author: "Community".into(),
            ext_type: "process".into(), url: None,
        },
        CatalogEntry {
            id: "archive".into(),
            name: "Archive".into(),
            description: "Archive merged features for reference".into(),
            version: "1.0.0".into(), author: "SpecKit".into(),
            ext_type: "process".into(), url: None,
        },
        CatalogEntry {
            id: "branch-convention".into(),
            name: "Branch Convention".into(),
            description: "Configurable branch naming patterns".into(),
            version: "1.0.0".into(), author: "SpecKit".into(),
            ext_type: "integration".into(), url: None,
        },
        CatalogEntry {
            id: "conduct".into(),
            name: "Conduct".into(),
            description: "Sub-agent delegation for complex tasks".into(),
            version: "1.0.0".into(), author: "Community".into(),
            ext_type: "process".into(), url: None,
        },
    ]
}

/// Search catalog by query
pub fn search(query: &str) -> Vec<CatalogEntry> {
    let q = query.to_lowercase();
    community_catalog().into_iter()
        .filter(|e| e.name.to_lowercase().contains(&q) ||
                     e.description.to_lowercase().contains(&q) ||
                     e.id.contains(&q))
        .collect()
}
