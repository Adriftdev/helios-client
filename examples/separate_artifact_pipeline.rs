//! examples/separate_artifact_pipeline.rs
//! Demonstrates isolation between steps using separate artifact volumes.

use helios_client::dsl::PipelineBuilder;
use helios_client::HeliosGraphQLClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example demonstrates two steps with separate artifact volumes.
    // Step 1 writes to /app/step1.txt in its own volume.
    // Step 2 mounts a different volume and tries to read /app/step1.txt (should not find it).

    let graphql_endpoint = "http://adriftdev.ddns.net/api/graphql";

    let pipeline = PipelineBuilder::new("separate-artifact-pipeline")
        .container("step1", "alpine:latest")
        .working_dir("/app")
        .with_args(&["sh", "-c", "mkdir -p /app && cd /app && echo '=== PWD ==='; pwd; echo '=== BEFORE ==='; ls -l; echo secret > step1.txt; echo '=== AFTER ==='; ls -l; sleep 2"])
        .with_volume("pvc-step1", "/app")
        .then()
        .container("step2", "alpine:latest")
        .depends_on("step1")
        .working_dir("/app")
        .with_args(&["sh", "-c", "cat step1.txt || echo not found"])
        .with_volume("pvc-step2", "/app")
        .then()
        .output("step1")
        .output("step2")
        .build();

    let client = HeliosGraphQLClient::new(graphql_endpoint.to_string());
    println!(
        "Client created. Sending pipeline to server at {}...",
        graphql_endpoint
    );

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
