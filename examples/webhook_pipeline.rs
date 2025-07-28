//! examples/webhook_pipeline.rs
use helios_client::dsl::PipelineBuilder;
use helios_client::HeliosGraphQLClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example builds and runs a pipeline that makes a GET request to a public API.
    //
    // NOTE: This example requires a running Helios server instance accessible
    // at the specified GraphQL endpoint.

    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://localhost:8000/api/graphql";

    // 2. Build the pipeline.
    let pipeline = PipelineBuilder::new("public-api-pipeline")
        .webhook("get_public_apis", "https://api.publicapis.org/entries")
        .method("GET")
        .then()
        .output("get_public_apis")
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
                println!("\nOutput from 'get_public_apis':");
                // Attempt to pretty-print the JSON response
                match serde_json::to_string_pretty(&output.result_json) {
                    Ok(json_str) => {
                        println!("{}", json_str);
                    }
                    Err(_) => {
                        // Fallback for non-JSON or malformed JSON
                        println!("{}", output.result_json);
                    }
                }
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
