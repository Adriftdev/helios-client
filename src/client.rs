// --- graphql_client Query Definition ---
use graphql_client::GraphQLQuery;

// This uses the structs defined above (PipelineInput, etc.)
// Make sure the paths in schema_path and query_path are correct
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql", // Path to your GraphQL schema file (SDL format)
    query_path = "src/run_pipeline.graphql", // Path to the mutation file
    response_derives = "Debug, Clone, Deserialize", // Derives for generated response struct
    variables_derives = "Debug, Clone" // Derives for generated variables struct
)]

pub struct RunPipeline;

// Type alias for the generated variables struct
pub type PipelineInputVariables = run_pipeline::Variables;
// Type alias for the specific data structure within the generated response
// (maps to the 'runPipeline' field in the mutation response)
pub type RunPipelineResponseData = run_pipeline::ResponseData;
