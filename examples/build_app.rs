//! examples/build_app.rs
use helios_client::dsl::PipelineBuilder;
use helios_client::{HeliosClientError, HeliosGraphQLClient};

#[tokio::main]
async fn main() -> Result<(), HeliosClientError> {
    // This example demonstrates building a pipeline and executing it against
    // a running Helios GraphQL server.
    //
    // To run this, you need a Helios server accessible at the specified endpoint.

    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://localhost:8000/api/graphql";

    println!("Building pipeline to send to {}", graphql_endpoint);

    // 2. Build the pipeline using the new DSL.
    let pipeline = PipelineBuilder::new("real-execution-pipeline")
        .script("local_echo", "echo 'This is a local script step!'")
        .then()
        .output("local_echo")
        .build();

    // 3. Create the GraphQL Client.
    let client = HeliosGraphQLClient::new(graphql_endpoint.to_string());
    println!("Client created. Sending pipeline to server...");

    // 4. Execute the GraphQL Mutation and handle the response.
    match client.run_pipeline(pipeline).await {
        Ok(response_data) => {
            println!("\n✅ Pipeline executed successfully!");
            println!("Server Message: {}", response_data.message);

            if response_data.outputs.is_empty() {
                println!("No outputs were returned from the pipeline.");
            } else {
                println!("\nRaw Pipeline Outputs:");
                for output in &response_data.outputs {
                    println!("  - Step ID: {}", output.step_id);
                    println!("    Result: {}", output.result_json);
                    if let Some(err) = &output.error {
                        eprintln!("    Error: {}", err);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ Error running pipeline: {}", e);
        }
    }

    Ok(())
}
