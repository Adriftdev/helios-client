#![allow(clippy::all, warnings)]
pub type JSON = serde_json::Value;
pub struct RunPipeline;
pub mod run_pipeline {
    #![allow(dead_code)]
    use std::result::Result;

    pub const OPERATION_NAME: &str = "RunPipeline";
    pub const QUERY: &str = r#"
    mutation RunPipeline($pipeline: RunPipelineInput!) {
        runPipeline(pipeline: $pipeline) {
            success
            message
            outputs {
                stepId
                resultJson
                error
            }
        }
    }"#;

    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    type JSON = super::JSON;

    #[derive()]
    pub enum ActionType {
        ACTION_TYPE_UNSPECIFIED,
        WEBHOOK,
        LLM_WORKFLOW,
        EVENT,
        NIMBUS_CONTAINER,
        SCRIPT,
        Other(String),
    }

    impl ::serde::Serialize for ActionType {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                ActionType::ACTION_TYPE_UNSPECIFIED => "ACTION_TYPE_UNSPECIFIED",
                ActionType::WEBHOOK => "WEBHOOK",
                ActionType::LLM_WORKFLOW => "LLM_WORKFLOW",
                ActionType::EVENT => "EVENT",
                ActionType::NIMBUS_CONTAINER => "NIMBUS_CONTAINER",
                ActionType::SCRIPT => "SCRIPT",
                ActionType::Other(ref s) => &s,
            })
        }
    }

    impl<'de> ::serde::Deserialize<'de> for ActionType {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "ACTION_TYPE_UNSPECIFIED" => Ok(ActionType::ACTION_TYPE_UNSPECIFIED),
                "WEBHOOK" => Ok(ActionType::WEBHOOK),
                "LLM_WORKFLOW" => Ok(ActionType::LLM_WORKFLOW),
                "EVENT" => Ok(ActionType::EVENT),
                "NIMBUS_CONTAINER" => Ok(ActionType::NIMBUS_CONTAINER),
                "SCRIPT" => Ok(ActionType::SCRIPT),
                _ => Ok(ActionType::Other(s)),
            }
        }
    }

    #[derive(Serialize)]
    pub struct RunPipelineInput {
        pub name: String,
        pub steps: Vec<StepInput>,
        pub outputs: Vec<String>,
    }

    #[derive(Serialize)]
    pub struct StepInput {
        pub id: String,
        pub name: String,
        pub key: String,
        #[serde(rename = "type")]
        pub type_: ActionType,
        pub data: JSON,
        pub target: Option<String>,
        #[serde(rename = "dependsOn")]
        pub depends_on: Option<String>,
        pub expression: Option<String>,
    }

    #[derive(Serialize)]
    pub struct Variables {
        pub pipeline: RunPipelineInput,
    }

    impl Variables {}

    #[derive(Deserialize)]
    pub struct ResponseData {
        #[serde(rename = "runPipeline")]
        pub run_pipeline: RunPipelineRunPipeline,
    }

    #[derive(Deserialize)]
    pub struct RunPipelineRunPipeline {
        pub success: Boolean,
        pub message: String,
        pub outputs: Vec<RunPipelineRunPipelineOutputs>,
    }

    #[derive(Deserialize)]
    pub struct RunPipelineRunPipelineOutputs {
        #[serde(rename = "stepId")]
        pub step_id: String,
        #[serde(rename = "resultJson")]
        pub result_json: JSON,
        pub error: Option<String>,
    }
}

impl graphql_client::GraphQLQuery for RunPipeline {
    type Variables = run_pipeline::Variables;
    type ResponseData = run_pipeline::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: run_pipeline::QUERY,
            operation_name: run_pipeline::OPERATION_NAME,
        }
    }
}
