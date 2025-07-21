// src/dsl.rs (or wherever your builder code lives)

// Use the GraphQL types defined above
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::client::run_pipeline::{PipelineInput, StepInput};

/// Represents the type of LLM prompt to use
#[derive(Debug, Clone, Copy)]
pub enum LLMPromptType {
    BasicPrompt,
    ChainPrompt,
    RoutePrompt,
    OrchestratorPrompt,
    EvaluatorPrompt,
}

impl From<LLMPromptType> for String {
    fn from(prompt_type: LLMPromptType) -> Self {
        match prompt_type {
            LLMPromptType::BasicPrompt => "basic".to_string(),
            LLMPromptType::ChainPrompt => "prompt_chain".to_string(),
            LLMPromptType::RoutePrompt => "routing".to_string(),
            LLMPromptType::OrchestratorPrompt => "orchestrator".to_string(),
            LLMPromptType::EvaluatorPrompt => "evaluator_optimizer".to_string(),
        }
    }
}

/// HTTP methods for webhook steps
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl From<HttpMethod> for String {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
        }
    }
}

/// Represents a platform for container execution
#[derive(Clone)]
pub enum Platform {
    LinuxArm64,
    LinuxAmd64,
    LinuxArm,
    LinuxArmV7,
    Windows,
    MacOSIntel,
    MacOS,
    Other(String),
}

impl From<Platform> for String {
    fn from(platform: Platform) -> Self {
        match platform {
            Platform::LinuxArm64 => "linux/arm64".to_string(),
            Platform::LinuxAmd64 => "linux/amd64".to_string(),
            Platform::LinuxArm => "linux/arm/v7".to_string(),
            Platform::LinuxArmV7 => "linux/arm/v7".to_string(),
            Platform::Windows => "windows/amd64".to_string(),
            Platform::MacOS => "linux/amd64".to_string(),
            Platform::MacOSIntel => "linux/arm64/v8".to_string(),
            Platform::Other(s) => s,
        }
    }
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match self {
            Platform::LinuxArm64 => "linux/arm64/v8".to_string(),
            Platform::LinuxAmd64 => "linux/arm64/v8".to_string(),
            Platform::LinuxArm => "linux/arm/v7".to_string(),
            Platform::LinuxArmV7 => "linux/arm/v7".to_string(),
            Platform::Windows => "windows/amd64".to_string(),
            Platform::MacOS => "linux/amd64".to_string(),
            Platform::MacOSIntel => "linux/arm64/v8".to_string(),
            Platform::Other(s) => s.clone(),
        }
    }
}

// Callback type remains the same for now
pub type StepCallback = Box<dyn Fn(Value) + Send + Sync>;

/// Main pipeline builder
pub struct PipelineBuilder {
    id: String,
    steps: Vec<StepInput>, // Changed from helios::Step to GraphQL StepInput
    outputs: Vec<String>,
    callbacks: HashMap<String, StepCallback>,
    current_step_id: Option<String>,
}

impl PipelineBuilder {
    /// Create a new pipeline with the given ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            steps: vec![],
            outputs: vec![],
            callbacks: HashMap::new(),
            current_step_id: None,
        }
    }

    /// Add a webhook step to the pipeline
    pub fn webhook<S: Into<String>>(mut self, id: S, url: &str) -> WebhookStepBuilder {
        let id_str = id.into();
        // Create GraphQL StepInput
        let step = StepInput {
            id: id_str.clone(),
            type_: crate::client::run_pipeline::ActionType::WEBHOOK,
            data: serde_json::json!({
                "url": url,
                "method": "GET" // Default method
            })
            .to_string(),
            target: None,
            depends_on: None,
            expression: None,
        };

        self.steps.push(step);
        self.current_step_id = Some(id_str.clone());

        WebhookStepBuilder {
            pipeline: self,
            step_id: id_str,
            // Keep internal state for builder methods
            url: url.to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add an LLM workflow step to the pipeline
    pub fn llm<S: Into<String>>(mut self, id: S, prompt: &str) -> LLMStepBuilder {
        let id_str = id.into();
        // Create GraphQL StepInput
        let step = StepInput {
            id: id_str.clone(),
            type_: crate::client::run_pipeline::ActionType::LLM_WORKFLOW,
            data: serde_json::json!({"prompt": prompt}).to_string(),
            target: Some(LLMPromptType::BasicPrompt.into()), // Keep existing logic for data/target
            depends_on: None,
            expression: None,
        };

        self.steps.push(step);
        self.current_step_id = Some(id_str.clone());

        LLMStepBuilder {
            pipeline: self,
            step_id: id_str,
            // Keep internal state for builder methods
            prompt: prompt.to_string(),
            prompt_type: LLMPromptType::BasicPrompt,
        }
    }

    /*
    /// Add a container step to the pipeline
    pub fn container<S: Into<String>>(self, id: S, image: &str) -> ContainerStepBuilder {
        let id_str = id.into();
        // ContainerStepBuilder needs modification to create StepInput
        ContainerStepBuilder::new(self, id_str, image)
    }
    */

    // --- output, on_output remain the same ---
    pub fn output<S: Into<String>>(mut self, step_id: S) -> Self {
        let id = step_id.into();
        // Ensure output exists as a step
        if self.steps.iter().any(|s| s.id == id) {
            if !self.outputs.contains(&id) {
                self.outputs.push(id);
            }
        } else {
            // Optionally add a warning or error here
            eprintln!("Warning: Adding output for non-existent step ID: {}", id);
            if !self.outputs.contains(&id) {
                self.outputs.push(id); // Add anyway, maybe defined later? Or error?
            }
        }
        self
    }

    pub fn on_output<S: Into<String>, F>(mut self, step_id: S, callback: F) -> Self
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        let id = step_id.into();
        // Ensure output exists as a step and mark it for output
        if self.steps.iter().any(|s| s.id == id) {
            if !self.outputs.contains(&id) {
                self.outputs.push(id.clone());
            }
        } else {
            eprintln!("Warning: Adding callback for non-existent step ID: {}", id);
            if !self.outputs.contains(&id) {
                self.outputs.push(id.clone());
            }
        }
        self.callbacks.insert(id, Box::new(callback));
        self
    }

    /// Build the PipelineInput for GraphQL and callbacks
    pub fn build(self) -> (PipelineInput, HashMap<String, StepCallback>) {
        // Create the GraphQL PipelineInput struct
        let pipeline_input = PipelineInput {
            id: self.id,
            steps: self.steps,
            outputs: self.outputs.clone(), // Ensure outputs are up-to-date
        };
        (pipeline_input, self.callbacks)
    }

    // Helper method to find and update a StepInput by ID
    fn update_step<F>(&mut self, step_id: &str, updater: F)
    where
        F: FnOnce(&mut StepInput), // Update signature to use StepInput
    {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            updater(step);
        }
    }
}

// --- Step Builders (WebhookStepBuilder, LLMStepBuilder, ContainerStepBuilder) ---
// These need similar modifications:
// - The `pipeline` field holds `PipelineBuilder`.
// - Methods like `method`, `header`, `body`, `prompt_type`, `depends_on`, `when`
//   call `self.pipeline.update_step` to modify the correct `StepInput` in the `pipeline.steps` Vec.
// - Make sure the data field is updated correctly as a JSON string.
// - `ContainerStepBuilder` needs a significant change in its `into_step` or how it updates the pipeline step.

// Example modification for WebhookStepBuilder:
#[allow(dead_code)]
pub struct WebhookStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
    // Internal state for building data JSON
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Value>,
}

impl WebhookStepBuilder {
    /// Set the HTTP method for the webhook
    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self.pipeline.update_step(&self.step_id, |step| {
            let mut data: Value = serde_json::from_str(&step.data)
                .unwrap_or_else(|_| serde_json::json!({"url": &self.url}));
            data["method"] = String::from(method).into();
            step.data = data.to_string(); // Update the JSON string
        });
        self
    }

    // Add header... (update step.data JSON string similarly)
    pub fn header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        let key_str = key.into();
        let value_str = value.into();
        self.headers.insert(key_str.clone(), value_str.clone());

        self.pipeline.update_step(&self.step_id, |step| {
            let mut data: Value = serde_json::from_str(&step.data)
                .unwrap_or_else(|_| serde_json::json!({"url": &self.url}));
            if data.get("headers").is_none() {
                data["headers"] = serde_json::json!({});
            }
            // Ensure headers is an object before inserting
            if let Some(headers_obj) = data["headers"].as_object_mut() {
                headers_obj.insert(key_str, Value::String(value_str));
            }
            step.data = data.to_string();
        });
        self
    }

    // Set body... (update step.data JSON string similarly)
    pub fn body<B: Serialize>(mut self, body: &B) -> Self {
        let json_body = serde_json::to_value(body).unwrap_or(Value::Null);
        self.body = Some(json_body.clone());

        self.pipeline.update_step(&self.step_id, |step| {
            let mut data: Value = serde_json::from_str(&step.data)
                .unwrap_or_else(|_| serde_json::json!({"url": &self.url}));
            data["body"] = json_body;
            step.data = data.to_string();
        });

        self
    }

    /// Make this step depend on another step
    pub fn depends_on<S: Into<String>>(mut self, step_id: S) -> Self {
        let depends_on_id = step_id.into();
        self.pipeline.update_step(&self.step_id, |step| {
            step.depends_on = Some(depends_on_id);
        });
        self
    }

    /// Add a condition for executing this step
    pub fn when(mut self, expression: &str) -> Self {
        self.pipeline.update_step(&self.step_id, |step| {
            step.expression = Some(expression.to_string());
        });
        self
    }

    /// Continue building the pipeline
    pub fn then(self) -> PipelineBuilder {
        self.pipeline
    }
}

// --- LLMStepBuilder ---
#[allow(dead_code)]
pub struct LLMStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
    // Internal state
    prompt: String,
    prompt_type: LLMPromptType,
}

impl LLMStepBuilder {
    /// Set the prompt type for the LLM step
    pub fn prompt_type(mut self, prompt_type: LLMPromptType) -> Self {
        self.prompt_type = prompt_type;
        self.pipeline.update_step(&self.step_id, |step| {
            step.target = Some(String::from(prompt_type)); // Update target field
        });
        self
    }

    /// Make this step depend on another step
    pub fn depends_on<S: Into<String>>(mut self, step_id: S) -> Self {
        let depends_on_id = step_id.into();
        self.pipeline.update_step(&self.step_id, |step| {
            step.depends_on = Some(depends_on_id);
        });
        self
    }

    /// Add a condition for executing this step
    pub fn when(mut self, expression: &str) -> Self {
        self.pipeline.update_step(&self.step_id, |step| {
            step.expression = Some(expression.to_string());
        });
        self
    }

    /// Continue building the pipeline
    pub fn then(self) -> PipelineBuilder {
        self.pipeline
    }
}

// --- ContainerStepBuilder & ContainerStep ---
// These need more significant changes because the original `ContainerStep`
// directly created the gRPC `Step`. Now, `ContainerStepBuilder` needs
// to call `pipeline.update_step` to modify the `StepInput`'s data field
// with the serialized JSON representation of the container config.

/*
#[derive(Debug, Serialize, Clone)]
pub struct Bind {
    cache: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)] // Add derive Serialize
#[serde(rename_all = "camelCase")] // Match JSON conventions if needed
struct ContainerStepData {
    from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    publish: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    with_mounted_cache: HashMap<String, Bind>, // Renamed for clarity/JSON convention
    #[serde(skip_serializing_if = "Option::is_none")]
    with_registry_auth: Option<Value>, // Keep as JSON Value for flexibility
    #[serde(skip_serializing_if = "Option::is_none")]
    with_workdir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    with_file: Option<Value>, // Keep as JSON Value
    #[serde(skip_serializing_if = "Option::is_none")]
    with_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    with_volume: Option<Value>, // Keep as JSON Value
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
}

/// Builder for container steps
pub struct ContainerStepBuilder {
    pipeline: PipelineBuilder,
    step_id: String,
    // Store the configuration being built
    container_data: ContainerStepData,
}

impl ContainerStepBuilder {
    fn new(mut pipeline: PipelineBuilder, step_id: String, image: &str) -> Self {
        // Initialize container data
        let container_data = ContainerStepData {
            from: image.to_string(),
            publish: None,
            with_mounted_cache: HashMap::new(),
            with_registry_auth: None,
            with_workdir: None,
            with_file: None,
            with_dir: None,
            entrypoint: None,
            args: vec![],
            with_volume: None,
            platform: None,
            timeout: None,
        };

        // Create the initial StepInput
        let step = StepInput {
            id: step_id.clone(),
            type_: crate::client::run_pipeline::ActionType::NIMBUS_CONTAINER,
            data: serde_json::to_string(&container_data).unwrap_or_else(|_| "{}".to_string()), // Serialize initial data
            target: None,
            depends_on: None,
            expression: None,
        };

        pipeline.steps.push(step);
        pipeline.current_step_id = Some(step_id.clone());

        Self {
            pipeline,
            step_id,
            container_data,
        }
    }

    // Methods now modify self.container_data and then call self.update_step()

    pub fn entrypoint(mut self, entrypoint: &str) -> Self {
        self.container_data.entrypoint = Some(entrypoint.to_string());
        self.update_step();
        self
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.container_data.args = args.iter().map(|s| s.to_string()).collect();
        self.update_step();
        self
    }

    pub fn with_cache(mut self, cache: &str, path: &str) -> Self {
        let bind = Bind {
            cache: cache.to_string(),
            path: path.to_string(),
        };
        self.container_data
            .with_mounted_cache
            .insert(cache.to_string(), bind);
        self.update_step();
        self
    }

    pub fn workdir(mut self, workdir: &str) -> Self {
        self.container_data.with_workdir = Some(workdir.to_string());
        self.update_step();
        self
    }

    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        // Assuming you want {"path": "content"} structure in JSON
        let mut file_map = serde_json::Map::new();
        file_map.insert(path.to_string(), Value::String(content.to_string()));
        self.container_data.with_file = Some(Value::Object(file_map));
        self.update_step();
        self
    }

    pub fn platform(mut self, platform: Platform) -> Self {
        self.container_data.platform = Some(platform.to_string());
        self.update_step();
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.container_data.timeout = Some(seconds);
        self.update_step();
        self
    }

    /// Make this step depend on another step
    pub fn depends_on<S: Into<String>>(mut self, step_id: S) -> Self {
        let depends_on_id = step_id.into();
        // Update depends_on directly on the StepInput via pipeline
        self.pipeline.update_step(&self.step_id, |step| {
            step.depends_on = Some(depends_on_id);
        });
        // No need to call self.update_step() as container_data didn't change
        self
    }

    /// Add a condition for executing this step
    pub fn when(mut self, expression: &str) -> Self {
        // Update expression directly on the StepInput via pipeline
        self.pipeline.update_step(&self.step_id, |step| {
            step.expression = Some(expression.to_string());
        });
        // No need to call self.update_step() as container_data didn't change
        self
    }

    /// Continue building the pipeline
    pub fn then(self) -> PipelineBuilder {
        self.pipeline
    }

    /// Helper method to update the step's data field in the pipeline
    fn update_step(&mut self) {
        // Serialize the current container_data
        let new_data_str =
            serde_json::to_string(&self.container_data).unwrap_or_else(|_| "{}".to_string());
        // Update the data field of the corresponding StepInput
        self.pipeline.update_step(&self.step_id, |s| {
            s.data = new_data_str.clone();
        });
    }
}
*/
