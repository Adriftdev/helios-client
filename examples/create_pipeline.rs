//! examples/create_pipeline.rs
//! Demonstrates creating a pipeline with a trigger using the create_pipeline mutation.

use helios_client::dsl::{CreatePipelineBuilder, Pipeline, PipelineBuilder};
use helios_client::{HeliosClientError, HeliosGraphQLClient};

#[tokio::main]
async fn main() -> Result<(), HeliosClientError> {
    // This example demonstrates creating a pipeline with a trigger (e.g., webhook or event).
    // The pipeline will be registered on the server and can be triggered later.
    //
    // NOTE: This example requires a running Helios server instance accessible
    // at the specified GraphQL endpoint.

    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://adriftdev.ddns.net/api/graphql";

    // 2. Build the pipeline with a trigger.
    let pipeline = CreatePipelineBuilder::new("test")
        .trigger("test")
        .webhook("test_webhook", "https://example.com/webhook")
        .then()
        .llm_workflow("test_llm")
        .prompt("What is the meaning of life?")
        .depends_on("test_webhook")
        .workflow_type(&helios_client::dsl::LLMWorkflowTypes::Agent)
        .then()
        .build();

    // 3. Create the GraphQL Client.
    let client = HeliosGraphQLClient::new(graphql_endpoint.to_string());
    println!(
        "Client created. Registering pipeline with trigger at {}...",
        graphql_endpoint
    );

    // 4. Execute the create_pipeline mutation.
    match client.create_pipeline(pipeline).await {
        Ok(response) => {
            println!("\n✅ Pipeline created successfully!");
            println!("Pipeline ID: {}", response.id);
            println!("Trigger: {}", response.trigger);
            println!("Steps field present (not printable as Debug)");
        }
        Err(e) => {
            eprintln!("\n❌ Error creating pipeline: {}", e);
            eprintln!(
                "\nHint: Is the Helios server running at {}?",
                graphql_endpoint
            );
        }
    }

    Ok(())
}
