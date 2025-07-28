#![allow(clippy::all, warnings)]
pub type JSON = serde_json::Value;
pub struct CreatePipeline;
pub mod create_pipeline {
    #![allow(dead_code)]
    use std::result::Result;

    pub const OPERATION_NAME: &str = "CreatePipeline";
    pub const QUERY: &str = r#"
    mutation CreatePipeline($pipeline: CreatePipelineInput!) {
        createPipeline(pipeline: $pipeline) {
            id
            trigger
            steps {
                name
                type
                target
                dependsOn
                expression
                data
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
    pub struct CreatePipelineInput {
        pub name: String,
        pub trigger: String,
        pub steps: Vec<StepInput>,
    }

    #[derive(Serialize)]
    pub struct StepInput {
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
        pub pipeline: CreatePipelineInput,
    }

    impl Variables {}

    #[derive(Deserialize)]
    pub struct ResponseData {
        #[serde(rename = "createPipeline")]
        pub create_pipeline: CreatePipelineCreatePipeline,
    }

    #[derive(Deserialize)]
    pub struct CreatePipelineCreatePipeline {
        pub id: String,
        pub trigger: String,
        pub steps: Vec<CreatePipelineCreatePipelineSteps>,
    }

    #[derive(Deserialize)]
    pub struct CreatePipelineCreatePipelineSteps {
        #[serde(rename = "type")]
        pub type_: ActionType,
        pub target: Option<String>,
        #[serde(rename = "dependsOn")]
        pub depends_on: Option<String>,
        pub expression: Option<String>,
        pub data: JSON,
    }
}

impl graphql_client::GraphQLQuery for CreatePipeline {
    type Variables = create_pipeline::Variables;
    type ResponseData = create_pipeline::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: create_pipeline::QUERY,
            operation_name: create_pipeline::OPERATION_NAME,
        }
    }
}
