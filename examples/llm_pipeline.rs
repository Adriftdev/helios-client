//! examples/simple_script.rs
use helios_client::dsl::PipelineBuilder;
use helios_client::HeliosGraphQLClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define the GraphQL server endpoint.
    let graphql_endpoint = "http://localhost:8000/api/graphql";

    // 2. Build the pipeline using the new DSL.
    let pipeline = PipelineBuilder::new("llm-pipeline")
        .llm_workflow("current_weather")
        .prompt("What is the weather in london?")
        .workflow_type(&helios_client::dsl::LLMWorkflowTypes::Agent)
        .then()
        .llm_workflow("drone_flight")
        .prompt(
            r#"
        Based on the current weather in london around the central train station: 

        {{nested_json 'current_weather'}}. 

        Should we fly drones today?. 
        "#,
        )
        .workflow_type(&helios_client::dsl::LLMWorkflowTypes::Agent)
        .depends_on("current_weather")
        .then()
        .output("drone_flight")
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
                println!("\nOutput from 'drone_flight':");
                println!(
                    "{}",
                    output
                        .result_json
                        .get("drone_flight")
                        .unwrap_or(&serde_json::Value::Null)
                );
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
