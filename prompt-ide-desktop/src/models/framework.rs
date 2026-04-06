use serde::{Deserialize, Serialize};
use super::block::{BlockType, PromptBlock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    pub id: String,
    pub name: String,
    pub description: String,
    pub blocks: Vec<FrameworkBlock>,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkBlock {
    pub block_type: BlockType,
    pub content: String,
}

pub fn builtin_frameworks() -> Vec<Framework> {
    vec![
        Framework {
            id: "co-star".into(),
            name: "CO-STAR".into(),
            description: "Context, Objective, Style, Tone, Audience, Response".into(),
            blocks: vec![
                FrameworkBlock { block_type: BlockType::Context, content: "## Contexte\n".into() },
                FrameworkBlock { block_type: BlockType::Task, content: "## Objectif\n".into() },
                FrameworkBlock { block_type: BlockType::Role, content: "## Style\n".into() },
                FrameworkBlock { block_type: BlockType::Constraints, content: "## Ton\n".into() },
                FrameworkBlock { block_type: BlockType::Format, content: "## Audience\n".into() },
                FrameworkBlock { block_type: BlockType::Format, content: "## Format de reponse\n".into() },
            ],
            builtin: true,
        },
        Framework {
            id: "risen".into(),
            name: "RISEN".into(),
            description: "Role, Instructions, Steps, End Goal, Narrowing".into(),
            blocks: vec![
                FrameworkBlock { block_type: BlockType::Role, content: "## Role\n".into() },
                FrameworkBlock { block_type: BlockType::Task, content: "## Instructions\n".into() },
                FrameworkBlock { block_type: BlockType::Task, content: "## Etapes\n1. \n2. \n3. ".into() },
                FrameworkBlock { block_type: BlockType::Format, content: "## Objectif final\n".into() },
                FrameworkBlock { block_type: BlockType::Constraints, content: "## Restrictions\n".into() },
            ],
            builtin: true,
        },
        Framework {
            id: "race".into(),
            name: "RACE".into(),
            description: "Role, Action, Context, Expect".into(),
            blocks: vec![
                FrameworkBlock { block_type: BlockType::Role, content: "## Role\n".into() },
                FrameworkBlock { block_type: BlockType::Task, content: "## Action\n".into() },
                FrameworkBlock { block_type: BlockType::Context, content: "## Contexte\n".into() },
                FrameworkBlock { block_type: BlockType::Format, content: "## Resultat attendu\n".into() },
            ],
            builtin: true,
        },
        Framework {
            id: "ape".into(),
            name: "APE".into(),
            description: "Action, Purpose, Expectation".into(),
            blocks: vec![
                FrameworkBlock { block_type: BlockType::Task, content: "## Action\n".into() },
                FrameworkBlock { block_type: BlockType::Context, content: "## But\n".into() },
                FrameworkBlock { block_type: BlockType::Format, content: "## Resultat attendu\n".into() },
            ],
            builtin: true,
        },
    ]
}

impl Framework {
    pub fn to_blocks(&self) -> Vec<PromptBlock> {
        self.blocks
            .iter()
            .map(|fb| {
                let mut block = PromptBlock::new(fb.block_type);
                block.content = fb.content.clone();
                block
            })
            .collect()
    }
}
