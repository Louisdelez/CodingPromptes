//! Spec Engine — core logic for Spec-Driven Development.
//! Inspired by Kiro specs and SpecKit methodology.
//!
//! Pipeline: Constitution → Specification → Plan → Tasks → Implementation
//! Each phase builds on the previous, with validation at each step.

pub mod templates;
pub mod validator;
pub mod generator;
pub mod workflow;
pub mod export;
