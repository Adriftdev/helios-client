//! examples/simple_script.rs
use helios_client::dsl::PipelineBuilder;
use helios_client::HeliosGraphQLClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example demonstrates building and running a simple, single-step
    // pipeline that executes a shell command.
    //
    // NOTE: This example requires a running Helios server instance accessible
    // at the specified GraphQL endpoint.

    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://localhost:8000/api/graphql";

    // 2. Build the pipeline using the new DSL.
    let pipeline = PipelineBuilder::new("simple-script-pipeline")
        .script("echo_step", "echo 'Hello from a Helios script!'")
        .then()
        .output("echo_step")
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
            if let Some(output) = response.outputs.first() {
                println!("\nOutput from 'echo_step':");
                println!("{}", output.result_json);
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
