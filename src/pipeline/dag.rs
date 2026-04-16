use std::collections::HashMap;
use std::marker::PhantomData;

use crate::pipeline::model::Pipeline;

// --- Typestate markers ---
pub struct Validated;
pub struct Scheduled;
pub struct Running;

/// Pipeline wrapped in a typestate — enforces Validated → Scheduled → Running at compile time
pub struct TypedPipeline<State> {
    pub inner: Pipeline,
    pub order: Vec<Vec<String>>, // populated after scheduling
    _state: PhantomData<State>,
}

impl TypedPipeline<Validated> {
    pub fn new(pipeline: Pipeline) -> Self {
        Self { inner: pipeline, order: vec![], _state: PhantomData }
    }

    /// Transition: Validated → Scheduled (builds DAG, returns execution waves)
    pub fn schedule(self) -> Result<TypedPipeline<Scheduled>, String> {
        let order = build_execution_waves(&self.inner)?;
        Ok(TypedPipeline { inner: self.inner, order, _state: PhantomData })
    }
}

impl TypedPipeline<Scheduled> {
    /// Transition: Scheduled → Running
    pub fn start(self) -> TypedPipeline<Running> {
        TypedPipeline { inner: self.inner, order: self.order, _state: PhantomData }
    }
}

/// Build waves of stages that can run in parallel.
/// Each wave contains stages whose `needs` are all satisfied by previous waves.
pub fn build_execution_waves(pipeline: &Pipeline) -> Result<Vec<Vec<String>>, String> {
    use petgraph::algo::toposort;
    use petgraph::graph::DiGraph;

    let stage_names: Vec<&String> = pipeline.stages.keys().collect();
    let mut graph = DiGraph::<&str, ()>::new();

    // Map stage name → node index
    let node_map: HashMap<&str, _> = stage_names
        .iter()
        .map(|name| (name.as_str(), graph.add_node(name.as_str())))
        .collect();

    // Add edges: needs[dep] → stage (dep must finish before stage)
    for (name, stage) in &pipeline.stages {
        if let Some(needs) = &stage.needs {
            for dep in needs {
                let from = node_map.get(dep.as_str()).ok_or_else(|| {
                    format!("Stage '{}' depends on unknown stage '{}'", name, dep)
                })?;
                let to = node_map[name.as_str()];
                graph.add_edge(*from, to, ());
            }
        }
    }

    // Detect cycles
    let sorted = toposort(&graph, None)
        .map_err(|e| format!("Circular dependency detected at stage '{}'", graph[e.node_id()]))?;

    // Group into waves: a stage goes in the earliest wave where all its deps are done
    let mut wave_of: HashMap<&str, usize> = HashMap::new();
    for node in &sorted {
        let name = graph[*node];
        let stage = &pipeline.stages[name];
        let wave = stage
            .needs
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|dep| wave_of.get(dep.as_str()).copied().unwrap_or(0) + 1)
            .max()
            .unwrap_or(0);
        wave_of.insert(name, wave);
    }

    let max_wave = wave_of.values().copied().max().unwrap_or(0);
    let mut waves: Vec<Vec<String>> = vec![vec![]; max_wave + 1];
    for (name, wave) in &wave_of {
        waves[*wave].push(name.to_string());
    }

    Ok(waves)
}

/// M4: Generic Scheduler<T> — generic over the executor type.
/// Demonstrates generics + monomorphization: Scheduler<ShellExecutor>, Scheduler<DockerExecutor>.
#[allow(dead_code)]
pub struct Scheduler<T> {
    pub waves: Vec<Vec<String>>,
    pub executor: T,
}

#[allow(dead_code)]
impl<T> Scheduler<T> {
    pub fn new(waves: Vec<Vec<String>>, executor: T) -> Self {
        Self { waves, executor }
    }

    pub fn wave_count(&self) -> usize {
        self.waves.len()
    }
}

/// Marker types for monomorphization
#[allow(dead_code)]
pub struct ShellExecutor;
#[allow(dead_code)]
pub struct DockerExecutor;
