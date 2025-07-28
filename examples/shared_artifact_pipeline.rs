//! examples/shared_artifact_pipeline.rs
//! Demonstrates sharing data between steps using the default shared artifact volume.

use helios_client::dsl::PipelineBuilder;
use helios_client::HeliosGraphQLClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example demonstrates two steps sharing a file via the default artifact volume.
    // Step 1 writes "hello world" to /app/shared.txt.
    // Step 2 reads /app/shared.txt and prints its contents.

    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://localhost:8000/api/graphql";

    // 2. Build the pipeline using the builder DSL.
    let pipeline = PipelineBuilder::new("shared-artifact-pipeline")
        .container("step1", "alpine:latest")
        .working_dir("/app")
        .with_args(&["sh", "-c", "mkdir -p /app && cd /app && echo '=== PWD ==='; pwd; echo '=== BEFORE ==='; ls -l; echo secret > step1.txt; echo '=== AFTER ==='; ls -l; sleep 2"])
        .then()
        .container("step2", "alpine:latest")
        .depends_on("step1")
        .working_dir("/app")
        .with_args(&["sh", "-c", "cat step1.txt || echo not found"])
        .then()
        .output("step1")
        .output("step2")
        .build();

    // 3. Create the GraphQL Client.
    let client = HeliosGraphQLClient::new(graphql_endpoint.to_string());
    println!(
        "Client created. Sending pipeline to server at {}...",
        graphql_endpoint
    );

    // 4. Execute the GraphQL Mutation.
    match client.run_pipeline(pipeline).await {
        Ok(response) => {
            println!("\n✅ Pipeline executed successfully!");
            println!("Server Message: {}", response.message);
            println!("\nOutputs:");
            for output in response.outputs {
                println!("  - Step ID: {}", output.step_id);
                println!("    Result: {}", output.result_json);
            }
        }
        Err(e) => {
            eprintln!("\n❌ Error running pipeline: {}", e);
            eprintln!(
                "\nHint: Is the Helios server running at {}?",
                graphql_endpoint
            );
        }
    }

    Ok(())
}
