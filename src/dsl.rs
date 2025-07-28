use serde_json::Value;

// --- ActionType and Step ---

#[derive(Debug, Clone)]
pub enum ActionType {
    Webhook,
    LlmWorkflow,
    Event,
    NimbusContainer,
    Script,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub id: Option<String>,
    pub name: String,
    pub key: String,
    pub type_: ActionType,
    pub data: Value,
    pub target: Option<String>,
    pub depends_on: Option<String>,
    pub expression: Option<String>,
}

// --- PipelineBuilder ---

#[derive(Default)]
pub struct PipelineBuilder {
    name: String,
    trigger: Option<String>,
    steps: Vec<Step>,
    outputs: Vec<String>,
    current_step: Option<Step>,
}

impl PipelineBuilder {
    pub fn new(name: &str) -> Self {
        PipelineBuilder {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn webhook(self, id: &str, url: &str) -> WebhookStepBuilder {
        WebhookStepBuilder::new(self, id, url)
    }

    pub fn script(self, id: &str, cmd: &str) -> ScriptStepBuilder {
        ScriptStepBuilder::new(self, id, cmd)
    }

    pub fn container(self, id: &str, image: &str) -> ContainerStepBuilder {
        ContainerStepBuilder::new(self, id, image)
    }

    pub fn llm_workflow(self, id: &str) -> LlmWorkflowStepBuilder {
        LlmWorkflowStepBuilder::new(self, id)
    }

    pub fn output(mut self, id: &str) -> Self {
        self.outputs.push(id.to_string());
        self
    }

    pub fn build(mut self) -> Pipeline {
        self.push_current();
        Pipeline {
            name: self.name,
            trigger: self.trigger,
            steps: self.steps,
            outputs: self.outputs,
        }
    }

    fn push_current(&mut self) {
        if let Some(step) = self.current_step.take() {
            self.steps.push(step);
        }
    }
}

// --- Pipeline struct for build() output ---

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub trigger: Option<String>,
    pub steps: Vec<Step>,
    pub outputs: Vec<String>,
}

// --- Conversion to GraphQL input types ---

use crate::create_pipeline;
use crate::run_pipeline;

impl From<ActionType> for create_pipeline::create_pipeline::ActionType {
    fn from(a: ActionType) -> Self {
        use create_pipeline::create_pipeline::ActionType as Gql;
        match a {
            ActionType::Webhook => Gql::WEBHOOK,
            ActionType::LlmWorkflow => Gql::LLM_WORKFLOW,
            ActionType::Event => Gql::EVENT,
            ActionType::NimbusContainer => Gql::NIMBUS_CONTAINER,
            ActionType::Script => Gql::SCRIPT,
        }
    }
}

impl From<ActionType> for run_pipeline::run_pipeline::ActionType {
    fn from(a: ActionType) -> Self {
        use run_pipeline::run_pipeline::ActionType as Gql;
        match a {
            ActionType::Webhook => Gql::WEBHOOK,
            ActionType::LlmWorkflow => Gql::LLM_WORKFLOW,
            ActionType::Event => Gql::EVENT,
            ActionType::NimbusContainer => Gql::NIMBUS_CONTAINER,
            ActionType::Script => Gql::SCRIPT,
        }
    }
}

impl From<Step> for create_pipeline::create_pipeline::StepInput {
    fn from(s: Step) -> Self {
        create_pipeline::create_pipeline::StepInput {
            name: s.name,
            key: s.key,
            type_: s.type_.into(),
            data: s.data,
            target: s.target,
            depends_on: s.depends_on,
            expression: s.expression,
        }
    }
}

impl From<Step> for run_pipeline::run_pipeline::StepInput {
    fn from(s: Step) -> Self {
        run_pipeline::run_pipeline::StepInput {
            id: s.id.unwrap_or_else(|| s.name.clone()),
            name: s.name,
            key: s.key,
            type_: s.type_.into(),
            data: s.data,
            target: s.target,
            depends_on: s.depends_on,
            expression: s.expression,
        }
    }
}

impl From<Pipeline> for create_pipeline::create_pipeline::CreatePipelineInput {
    fn from(p: Pipeline) -> Self {
        create_pipeline::create_pipeline::CreatePipelineInput {
            name: p.name,
            trigger: p.trigger.unwrap_or_else(|| "manual".to_string()),
            steps: p.steps.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Pipeline> for run_pipeline::run_pipeline::RunPipelineInput {
    fn from(p: Pipeline) -> Self {
        run_pipeline::run_pipeline::RunPipelineInput {
            name: p.name,
            steps: p.steps.into_iter().map(Into::into).collect(),
            outputs: p.outputs,
        }
    }
}

// --- WebhookStepBuilder ---

pub struct WebhookStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
}

impl WebhookStepBuilder {
    pub fn new(mut pipeline: PipelineBuilder, id: &str, url: &str) -> Self {
        pipeline.push_current();
        let step = Step {
            id: Some(id.to_string()),
            name: id.to_string(),
            key: id.to_string(),
            type_: ActionType::Webhook,
            data: serde_json::json!({ "url": url, "method": "GET" }),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.current_step = Some(step);
        Self {
            pipeline,
            step_id: id.to_string(),
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("method".to_string(), serde_json::json!(method));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            let headers = data
                .entry("headers")
                .or_insert_with(|| serde_json::json!({}));
            if let Some(headers_map) = headers.as_object_mut() {
                headers_map.insert(key.to_string(), serde_json::json!(value));
            }
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn body<T: serde::Serialize>(mut self, body: &T) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert(
                "body".to_string(),
                serde_json::to_value(body).unwrap_or(serde_json::json!(null)),
            );
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(mut self) -> PipelineBuilder {
        self.pipeline.push_current();
        self.pipeline
    }
}

// --- ScriptStepBuilder ---

pub struct ScriptStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
}

impl ScriptStepBuilder {
    pub fn new(mut pipeline: PipelineBuilder, id: &str, script: &str) -> Self {
        pipeline.push_current();
        let step = Step {
            id: Some(id.to_string()),
            name: id.to_string(),
            key: id.to_string(),
            type_: ActionType::Script,
            data: serde_json::json!({ "script": script }),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.current_step = Some(step);
        Self {
            pipeline,
            step_id: id.to_string(),
        }
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(mut self) -> PipelineBuilder {
        self.pipeline.push_current();
        self.pipeline
    }
}

// --- ContainerStepBuilder ---

pub struct ContainerStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
}

impl ContainerStepBuilder {
    pub fn new(mut pipeline: PipelineBuilder, id: &str, image: &str) -> Self {
        pipeline.push_current();
        let step = Step {
            id: Some(id.to_string()),
            name: id.to_string(),
            key: id.to_string(),
            type_: ActionType::NimbusContainer,
            data: serde_json::json!({ "image": image, "step_id": id }),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.current_step = Some(step);
        Self {
            pipeline,
            step_id: id.to_string(),
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("command".to_string(), serde_json::json!(args));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn with_volume(mut self, host_path: &str, container_path: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            let volumes = data
                .entry("volumes")
                .or_insert_with(|| serde_json::json!([]));
            if let Some(vols) = volumes.as_array_mut() {
                vols.push(serde_json::json!({
                    "host_path": host_path,
                    "container_path": container_path,
                    "read_only": false
                }));
            }
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn platform(mut self, platform: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("platform".to_string(), serde_json::json!(platform));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("timeout".to_string(), serde_json::json!(seconds));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn working_dir(mut self, dir: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("working_dir".to_string(), serde_json::json!(dir));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(mut self) -> PipelineBuilder {
        self.pipeline.push_current();
        self.pipeline
    }
}

pub enum LLMWorkflowTypes {
    Generic,
    PromptChain,
    Routing,
    Orchestrator,
    Agent,
    EvaluationOptimization,
}

impl std::fmt::Display for LLMWorkflowTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMWorkflowTypes::Generic => write!(f, "generic"),
            LLMWorkflowTypes::PromptChain => write!(f, "prompt_chain"),
            LLMWorkflowTypes::Routing => write!(f, "routing"),
            LLMWorkflowTypes::Orchestrator => write!(f, "orchestrator"),
            LLMWorkflowTypes::Agent => write!(f, "agent_executor"),
            LLMWorkflowTypes::EvaluationOptimization => write!(f, "evaluator_optimizer"),
        }
    }
}

pub struct LlmWorkflowStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
}
impl LlmWorkflowStepBuilder {
    pub fn new(mut pipeline: PipelineBuilder, id: &str) -> Self {
        pipeline.push_current();
        let step = Step {
            id: Some(id.to_string()),
            name: id.to_string(),
            key: id.to_string(),
            type_: ActionType::LlmWorkflow,
            data: serde_json::json!({}),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.current_step = Some(step);
        Self {
            pipeline,
            step_id: id.to_string(),
        }
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(mut self) -> PipelineBuilder {
        self.pipeline.push_current();
        self.pipeline
    }

    pub fn prompt(mut self, prompt: &str) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("prompt".to_string(), serde_json::json!(prompt));
            step.data = serde_json::json!(data);
        }
        self
    }

    // stores the workfow type on step.target as a string
    pub fn workflow_type(mut self, workflow_type: &LLMWorkflowTypes) -> Self {
        if let Some(ref mut step) = self.pipeline.current_step {
            step.target = Some(workflow_type.to_string());
        }
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct CreatePipelineBuilder {
    pub name: String,
    pub trigger: Option<String>,
    pub steps: Vec<Step>,
    pub outputs: Vec<String>,
    pub current_step: Option<Step>,
}

impl CreatePipelineBuilder {
    pub fn new(name: &str) -> Self {
        CreatePipelineBuilder {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn trigger(mut self, trigger: &str) -> Self {
        self.trigger = Some(trigger.to_string());
        self
    }

    pub fn webhook(self, name: &str, url: &str) -> CreateWebhookStepBuilder {
        CreateWebhookStepBuilder::new(self, name, url)
    }

    pub fn script(self, name: &str, cmd: &str) -> CreateScriptStepBuilder {
        CreateScriptStepBuilder::new(self, name, cmd)
    }

    pub fn container(self, name: &str, image: &str) -> CreateContainerStepBuilder {
        CreateContainerStepBuilder::new(self, name, image)
    }

    pub fn llm_workflow(self, name: &str) -> CreateLlmWorkflowStepBuilder {
        CreateLlmWorkflowStepBuilder::new(self, name)
    }

    pub fn output(mut self, id: &str) -> Self {
        self.outputs.push(id.to_string());
        self
    }

    pub fn build(mut self) -> Pipeline {
        self.push_current();
        Pipeline {
            name: self.name,
            trigger: self.trigger,
            steps: self.steps,
            outputs: self.outputs,
        }
    }

    fn push_current(&mut self) {
        if let Some(step) = self.current_step.take() {
            self.steps.push(step);
        }
    }
}

pub struct CreateWebhookStepBuilder {
    pipeline: CreatePipelineBuilder,
    step_name: String,
}

impl CreateWebhookStepBuilder {
    pub fn new(mut pipeline: CreatePipelineBuilder, name: &str, url: &str) -> Self {
        let step = Step {
            id: None,
            name: name.to_string(),
            key: name.to_string(),
            type_: ActionType::Webhook,
            data: serde_json::json!({ "url": url, "method": "GET" }),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.steps.push(step);
        Self {
            pipeline,
            step_name: name.to_string(),
        }
    }
    pub fn method(mut self, method: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("method".to_string(), serde_json::json!(method));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            let headers = data
                .entry("headers")
                .or_insert_with(|| serde_json::json!({}));
            if let Some(headers_map) = headers.as_object_mut() {
                headers_map.insert(key.to_string(), serde_json::json!(value));
            }
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn body<T: serde::Serialize>(mut self, body: &T) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert(
                "body".to_string(),
                serde_json::to_value(body).unwrap_or(serde_json::json!(null)),
            );
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(self) -> CreatePipelineBuilder {
        self.pipeline
    }
}

pub struct CreateScriptStepBuilder {
    pipeline: CreatePipelineBuilder,
    step_name: String,
}

impl CreateScriptStepBuilder {
    pub fn new(mut pipeline: CreatePipelineBuilder, name: &str, script: &str) -> Self {
        let step = Step {
            id: None,
            name: name.to_string(),
            key: name.to_string(),
            type_: ActionType::Script,
            data: serde_json::json!({ "script": script }),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.steps.push(step);
        Self {
            pipeline,
            step_name: name.to_string(),
        }
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(self) -> CreatePipelineBuilder {
        self.pipeline
    }
}

pub struct CreateContainerStepBuilder {
    pipeline: CreatePipelineBuilder,
    step_name: String,
}

impl CreateContainerStepBuilder {
    pub fn new(mut pipeline: CreatePipelineBuilder, name: &str, image: &str) -> Self {
        let step = Step {
            id: None,
            name: name.to_string(),
            key: name.to_string(),
            type_: ActionType::NimbusContainer,
            data: serde_json::json!({ "image": image, "step_id": name }),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.steps.push(step);
        Self {
            pipeline,
            step_name: name.to_string(),
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("command".to_string(), serde_json::json!(args));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn with_volume(mut self, host_path: &str, container_path: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            let volumes = data
                .entry("volumes")
                .or_insert_with(|| serde_json::json!([]));
            if let Some(vols) = volumes.as_array_mut() {
                vols.push(serde_json::json!({
                    "host_path": host_path,
                    "container_path": container_path,
                    "read_only": false
                }));
            }
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn platform(mut self, platform: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("platform".to_string(), serde_json::json!(platform));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("timeout".to_string(), serde_json::json!(seconds));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn working_dir(mut self, dir: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("working_dir".to_string(), serde_json::json!(dir));
            step.data = serde_json::json!(data);
        }
        self
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(self) -> CreatePipelineBuilder {
        self.pipeline
    }
}

pub struct CreateLlmWorkflowStepBuilder {
    pipeline: CreatePipelineBuilder,
    step_name: String,
}

impl CreateLlmWorkflowStepBuilder {
    pub fn new(mut pipeline: CreatePipelineBuilder, name: &str) -> Self {
        let step = Step {
            id: None,
            name: name.to_string(),
            key: name.to_string(),
            type_: ActionType::LlmWorkflow,
            data: serde_json::json!({}),
            target: None,
            depends_on: None,
            expression: None,
        };
        pipeline.steps.push(step);
        Self {
            pipeline,
            step_name: name.to_string(),
        }
    }

    pub fn depends_on(mut self, id: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.depends_on = Some(id.to_string());
        }
        self
    }

    pub fn when(mut self, expression: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.expression = Some(expression.to_string());
        }
        self
    }

    pub fn then(self) -> CreatePipelineBuilder {
        self.pipeline
    }

    pub fn prompt(mut self, prompt: &str) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            let mut data = step.data.as_object().cloned().unwrap_or_default();
            data.insert("prompt".to_string(), serde_json::json!(prompt));
            step.data = serde_json::json!(data);
        }
        self
    }

    // stores the workfow type on step.target as a string
    pub fn workflow_type(mut self, workflow_type: &LLMWorkflowTypes) -> Self {
        if let Some(step) = self.pipeline.steps.last_mut() {
            step.target = Some(workflow_type.to_string());
        }
        self
    }
}
