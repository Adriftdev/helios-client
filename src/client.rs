// --- graphql_client Query Definition ---

pub type Json = serde_json::Value;
// Use generated modules instead of proc macros

pub type RunPipelineVariables = crate::run_pipeline::run_pipeline::Variables;
pub type RunPipelineResponseData = crate::run_pipeline::run_pipeline::ResponseData;
pub type RunPipelineRunPipeline = crate::run_pipeline::run_pipeline::RunPipelineRunPipeline;

pub type CreatePipelineVariables = crate::create_pipeline::create_pipeline::Variables;
pub type CreatePipelineResponseData = crate::create_pipeline::create_pipeline::ResponseData;
pub type CreatePipelineCreatePipeline =
    crate::create_pipeline::create_pipeline::CreatePipelineCreatePipeline;
