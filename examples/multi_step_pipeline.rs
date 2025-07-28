//! examples/multi_step_pipeline.rs
use helios_client::dsl::PipelineBuilder;
use helios_client::HeliosGraphQLClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example demonstrates running a multi-step pipeline with dependencies
    // and conditional execution.
    //
    // NOTE: This example requires a running Helios server instance accessible
    // at the specified GraphQL endpoint.

    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://adriftdev.ddns.net/api/graphql";

    // 2. Build the pipeline using the new DSL.
    let pipeline = PipelineBuilder::new("conditional-pipeline")
        .script(
            "generate_message",
            "echo '{\"message\": \"Success from step 1\"}'",
        )
        .then()
        .container("process_message", "alpine:latest")
        .with_args(&[
            "sh",
            "-c",
            "echo 'Received from previous step: {{ generate_message }}'",
        ])
        .depends_on("generate_message")
        .when("script == 'Success from step 1'")
        .then()
        .output("generate_message")
        .output("process_message")
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
