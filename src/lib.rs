pub mod client;
pub mod dsl;

pub mod create_pipeline;
pub mod run_pipeline;
pub type JSON = serde_json::Value;
use graphql_client::GraphQLQuery;

use crate::create_pipeline::create_pipeline::{
    CreatePipelineCreatePipeline, Variables as CreatePipelineVariables,
};
use crate::run_pipeline::run_pipeline::{
    RunPipelineRunPipeline, Variables as RunPipelineVariables,
};
use client::{CreatePipelineResponseData, RunPipelineResponseData};
use dsl::Pipeline;
use reqwest::Client;
use thiserror::Error;

// Remove GraphQLQuery import, not needed for generated modules
use graphql_client::Response as GraphQLResponse; // GraphQL types

#[derive(Error, Debug)]
pub enum HeliosClientError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("GraphQL request failed: {0:?}")]
    GraphQL(Vec<graphql_client::Error>),
    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("No data received from GraphQL response")]
    NoData,
    #[error("Invalid response format")]
    InvalidResponseFormat,
    #[error("Invalid pipeline input: {0}")]
    GenericError(String),
}

#[derive(Clone)] // Add clone if needed
pub struct HeliosGraphQLClient {
    endpoint: String,
    http_client: Client,
}

impl HeliosGraphQLClient {
    /// Creates a new client instance.
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            http_client: Client::new(), // Create a reqwest client
        }
    }

    /// Connect (alternative constructor, maybe less needed now)
    pub async fn connect(endpoint: &str) -> Result<Self, HeliosClientError> {
        // Basic connect, no TLS specific config here, reqwest handles defaults
        Ok(Self::new(endpoint.to_string()))
    }

    /// Runs the pipeline via GraphQL mutation.
    pub async fn run_pipeline(
        &self,
        pipeline: Pipeline,
    ) -> Result<RunPipelineRunPipeline, HeliosClientError> {
        if pipeline.trigger.is_some() {
            return Err(HeliosClientError::GenericError(
                "Pipelines with triggers must be created, not run directly.".to_string(),
            ));
        }

        let variables = RunPipelineVariables {
            pipeline: pipeline.into(),
        };

        let request_body = crate::run_pipeline::RunPipeline::build_query(variables);

        let res = self
            .http_client
            .post(&self.endpoint)
            .json(&request_body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_body = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            return Err(HeliosClientError::GenericError(format!(
                "HTTP error: {} - {}",
                status, error_body
            )));
        }

        let response_body: GraphQLResponse<RunPipelineResponseData> = res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(HeliosClientError::GraphQL(errors));
        }

        let response_data = response_body.data.ok_or(HeliosClientError::NoData)?;
        let run_pipeline_data = response_data.run_pipeline;

        Ok(run_pipeline_data)
    }

    pub async fn create_pipeline(
        &self,
        pipeline: Pipeline,
    ) -> Result<CreatePipelineCreatePipeline, HeliosClientError> {
        if pipeline.trigger.is_none() {
            return Err(HeliosClientError::GenericError(
                "Pipelines without triggers must be run directly, not created.".to_string(),
            ));
        }

        let variables = CreatePipelineVariables {
            pipeline: pipeline.into(),
        };

        let request_body = crate::create_pipeline::CreatePipeline::build_query(variables);

        let res = self
            .http_client
            .post(&self.endpoint)
            .json(&request_body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_body = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            return Err(HeliosClientError::GenericError(format!(
                "HTTP error: {} - {}",
                status, error_body
            )));
        }

        let response_body: GraphQLResponse<CreatePipelineResponseData> = res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(HeliosClientError::GraphQL(errors));
        }

        let response_data = response_body.data.ok_or(HeliosClientError::NoData)?;
        let create_pipeline_data = response_data.create_pipeline;

        Ok(create_pipeline_data)
    }
}
