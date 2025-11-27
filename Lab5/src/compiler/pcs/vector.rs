use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstNode, BinaryOperationKind, UnaryOperationKind,
};
use crate::compiler::reports::Reporter;
use crate::utils::StringBuffer;
use std::collections::{HashMap, HashSet};

// Configuration
pub mod time {
    pub const ADD: usize = 1;
    pub const SUB: usize = 1;
    pub const MUL: usize = 2;
    pub const DIV: usize = 4;
}

pub mod processors {
    pub const ADD: usize = 1;
    pub const SUB: usize = 1;
    pub const MUL: usize = 1;
    pub const DIV: usize = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    Add,
    Sub,
    Mul,
    Div,
    Load, // For variables and numbers (immediate execution)
}

impl OperationType {
    fn execution_time(&self) -> usize {
        match self {
            Self::Add => time::ADD,
            Self::Sub => time::SUB,
            Self::Mul => time::MUL,
            Self::Div => time::DIV,
            Self::Load => 0,
        }
    }

    fn from_binary(kind: &BinaryOperationKind) -> Self {
        match kind {
            BinaryOperationKind::Plus => Self::Add,
            BinaryOperationKind::Minus => Self::Sub,
            BinaryOperationKind::Multiply => Self::Mul,
            BinaryOperationKind::Divide => Self::Div,
            _ => OperationType::Load, // Logical ops are treated as immediate/loads for this lab
        }
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Add => "ADD_BLOCK",
            Self::Sub => "SUB_BLOCK",
            Self::Mul => "MUL_BLOCK",
            Self::Div => "DIV_BLOCK",
            Self::Load => "LOAD",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    operation_type: OperationType,
    dependencies: Vec<usize>,
    display_name: String,
}

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub task_id: usize,
    pub name: String,
    pub start_time: usize,
    pub end_time: usize,
    pub processor: OperationType,
}

#[derive(Debug, Clone)]
pub struct TickLog {
    pub tick: usize,
    // Map: Processor Name -> Task Name (or "Idle")
    pub processor_states: HashMap<String, String>,
    pub ready_queue: Vec<String>,
}

pub struct SimulationResult {
    pub schedule: Vec<ScheduledTask>,
    pub tick_logs: Vec<TickLog>,
    pub t1: usize,
    pub tp: usize,
    pub speedup: f64,
    pub efficiency: f64,
}

pub struct VectorSystemSimulator;

impl VectorSystemSimulator {
    pub fn simulate(ast: &AbstractSyntaxTree) -> SimulationResult {
        // Convert AST to Task Graph
        let (tasks, _) = Self::flatten_ast(ast);

        // Simulate execution
        let (schedule, tick_logs) = Self::run_list_scheduling(tasks.clone());

        // Calculate metrics
        // Tp = End time of the last task
        let tp = schedule.iter().map(|t| t.end_time).max().unwrap_or(0);

        // T1 = Sum of execution times of all computational tasks (excluding Loads)
        let t1: usize = schedule.iter().map(|t| t.end_time - t.start_time).sum();

        let total_processors =
            processors::ADD + processors::SUB + processors::MUL + processors::DIV;

        // Avoid division by zero
        let speedup = if tp > 0 { t1 as f64 / tp as f64 } else { 0.0 };
        let efficiency = if total_processors > 0 {
            speedup / total_processors as f64
        } else {
            0.0
        };

        SimulationResult {
            schedule,
            tick_logs,
            t1,
            tp,
            speedup,
            efficiency,
        }
    }

    /// Converts the recursive AST into a flat HashMap of Tasks with dependencies.
    fn flatten_ast(ast: &AbstractSyntaxTree) -> (HashMap<usize, Task>, usize) {
        let mut tasks = HashMap::new();
        let mut id_counter = 0;
        let root_id = Self::traverse_node(&ast.peek, &mut tasks, &mut id_counter);
        (tasks, root_id)
    }

    fn traverse_node(
        node: &AstNode, tasks: &mut HashMap<usize, Task>, counter: &mut usize,
    ) -> usize {
        let current_id = *counter;
        *counter += 1;

        match node {
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let left_id = Self::traverse_node(left, tasks, counter);
                let right_id = Self::traverse_node(right, tasks, counter);

                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type: OperationType::from_binary(operation),
                        dependencies: vec![left_id, right_id],
                        display_name: operation.to_string(),
                    },
                );
            },
            AstNode::UnaryOperation {
                operation,
                expression,
            } => {
                let child_id = Self::traverse_node(expression, tasks, counter);

                // Map Unary Minus to Subtraction block (0 - expr)
                let operation_type = match operation {
                    UnaryOperationKind::Minus => OperationType::Sub,
                    _ => OperationType::Load, // Negation '!' treated as instant
                };

                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type,
                        dependencies: vec![child_id],
                        display_name: format!("Unary{}", operation),
                    },
                );
            },
            AstNode::Number(n) => {
                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type: OperationType::Load,
                        dependencies: vec![],
                        display_name: format!("{:.1}", n),
                    },
                );
            },
            AstNode::Identifier(s) => {
                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type: OperationType::Load,
                        dependencies: vec![],
                        display_name: s.clone(),
                    },
                );
            },
            // Function calls and Array access treated as Load (black box)
            AstNode::FunctionCall { name, .. } => {
                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type: OperationType::Load,
                        dependencies: vec![], // Simplifying: arguments handled inside but treated as one unit here
                        display_name: format!("{}()", name),
                    },
                );
            },
            AstNode::ArrayAccess { identifier, .. } => {
                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type: OperationType::Load,
                        dependencies: vec![],
                        display_name: format!("{}[..]", identifier),
                    },
                );
            },
            _ => {
                tasks.insert(
                    current_id,
                    Task {
                        id: current_id,
                        operation_type: OperationType::Load,
                        dependencies: vec![],
                        display_name: "?".to_string(),
                    },
                );
            },
        }
        current_id
    }

    /// List Scheduling Algorithm
    fn run_list_scheduling(
        tasks: HashMap<usize, Task>,
    ) -> (Vec<ScheduledTask>, Vec<TickLog>) {
        let mut final_schedule = Vec::new();
        let mut tick_logs = Vec::new();

        // Initialize Processor states: Map<OpType, Vec<BusyUntilTick>>
        // The Vec represents the pool of processors of that type.
        let mut processors: HashMap<OperationType, Vec<usize>> = HashMap::new();
        processors.insert(OperationType::Add, vec![0; processors::ADD]);
        processors.insert(OperationType::Sub, vec![0; processors::SUB]);
        processors.insert(OperationType::Mul, vec![0; processors::MUL]);
        processors.insert(OperationType::Div, vec![0; processors::DIV]);

        // "Load" operations don't need processors, they are instant.

        // Task states
        let mut task_finish_time: HashMap<usize, usize> = HashMap::new();
        let mut completed_tasks: HashSet<usize> = HashSet::new();

        // Pre-process "Load" tasks (variables/constants) as completed at T=0
        for task in tasks.values() {
            if task.operation_type == OperationType::Load {
                task_finish_time.insert(task.id, 0);
                completed_tasks.insert(task.id);
            }
        }

        let mut current_tick = 0;
        let mut active_tasks: HashMap<usize, (usize, OperationType)> = HashMap::new(); // TaskId -> (EndTime, ProcType)

        // Main Loop
        loop {
            // A. Check for completed tasks in this tick
            let mut just_finished = Vec::new();
            // We need to collect keys to remove to avoid borrowing issues
            let finished_ids: Vec<usize> = active_tasks
                .iter()
                .filter(|(_, (end_time, _))| *end_time <= current_tick)
                .map(|(id, _)| *id)
                .collect();

            for id in finished_ids {
                active_tasks.remove(&id);
                completed_tasks.insert(id);
                just_finished.push(id);
            }

            // If all tasks are done, break
            if completed_tasks.len() == tasks.len() {
                break;
            }

            // B. Find Ready Tasks
            let mut ready_queue: Vec<&Task> = tasks.values()
                .filter(|t| !completed_tasks.contains(&t.id)) // Not completed
                .filter(|t| !active_tasks.contains_key(&t.id)) // Not currently running
                .filter(|t| {
                    // All dependencies must be completed
                    t.dependencies.iter().all(|dep| completed_tasks.contains(dep))
                })
                .collect();

            // Heuristic: Sort by operation type cost (Longest Processing Time first) or just ID
            ready_queue.sort_by(|a, b| {
                let time_a = a.operation_type.execution_time();
                let time_b = b.operation_type.execution_time();
                if time_a != time_b {
                    time_b.cmp(&time_a) // Higher cost first
                } else {
                    a.id.cmp(&b.id)
                }
            });

            // Log snapshot preparation
            let mut current_tick_log = TickLog {
                tick: current_tick,
                processor_states: HashMap::new(),
                ready_queue: ready_queue.iter().map(|t| t.display_name.clone()).collect(),
            };

            // C. Assign Ready Tasks to Free Processors
            for task in ready_queue {
                if let Some(proc_pool) = processors.get_mut(&task.operation_type) {
                    // Find a processor that is free at current_tick
                    if let Some(proc_idx) = proc_pool
                        .iter()
                        .position(|&busy_until| busy_until <= current_tick)
                    {
                        // Schedule!
                        let duration = task.operation_type.execution_time();
                        let start = current_tick;
                        let end = start + duration;

                        // Mark processor busy
                        proc_pool[proc_idx] = end;

                        // Add to schedule
                        final_schedule.push(ScheduledTask {
                            task_id: task.id,
                            name: task.display_name.clone(),
                            start_time: start,
                            end_time: end,
                            processor: task.operation_type,
                        });

                        // Add to active tasks
                        active_tasks.insert(task.id, (end, task.operation_type));

                        // We also record finish time for dependency checking later
                        task_finish_time.insert(task.id, end);
                    }
                }
            }

            // D. Populate Tick Log with Processor Status
            // Helper to find what task is running on a specific processor type
            // Note: This simple model assumes 1 processor of each type.
            // If COUNT > 1, we would need to track which specific processor index is used.
            for operation_type in processors.keys() {
                if operation_type.eq(&OperationType::Load) {
                    continue;
                }

                let op_name = operation_type.to_string();
                let mut status = "Idle".to_string();

                // Find if any active task matches this op_type
                // Since we have 1 proc per type, we just check if there is ANY active task of this type
                if let Some((id, _)) =
                    active_tasks.iter().find(|(_, (_, t))| t == operation_type)
                    && let Some(task) = tasks.get(id)
                {
                    status = format!("Processing '{}'", task.display_name);
                }

                current_tick_log.processor_states.insert(op_name, status);
            }
            tick_logs.push(current_tick_log);

            // E. Advance Time
            current_tick += 1;

            // Safety break
            if current_tick > 10000 {
                break;
            }
        }

        (final_schedule, tick_logs)
    }
}

impl Reporter {
    pub fn pcs_simulation(&self, result: &SimulationResult) -> String {
        let mut buffer = StringBuffer::default();

        buffer.add_line("Parallel Computer System Simulation Results:".to_string());
        buffer.add_line("-".repeat(60));
        buffer.add_line(format!(
            "Configuration: Add({}), Sub({}), Mul({}), Div({})",
            processors::ADD,
            processors::SUB,
            processors::MUL,
            processors::DIV
        ));
        buffer.add_line(format!(
            "Costs (Ticks): Add={}, Sub={}, Mul={}, Div={}",
            time::ADD,
            time::SUB,
            time::MUL,
            time::DIV
        ));
        buffer.add_line("-".repeat(60));

        // Metrics
        buffer.add_line(format!("Sequential Time (T1): {}", result.t1));
        buffer.add_line(format!("Parallel Time (Tp):   {}", result.tp));
        buffer.add_line(format!("Speedup (Ky):         {:.4}", result.speedup));
        buffer.add_line(format!("Efficiency (E):       {:.4}", result.efficiency));
        buffer.add_line("-".repeat(60));

        // Schedule Table
        buffer.add_line(format!(
            "{:<12} | {:<10} | {:<10} | {:<15}",
            "Processor", "Start", "End", "Operation"
        ));
        buffer.add_line("-".repeat(60));

        // Sort by start time for better readability
        let mut sorted_schedule = result.schedule.clone();
        sorted_schedule.sort_by_key(|t| t.start_time);

        for task in sorted_schedule {
            buffer.add_line(format!(
                "{:<12} | {:<10} | {:<10} | {:<15}",
                task.processor.to_string(),
                task.start_time,
                task.end_time,
                task.name
            ));
        }

        buffer.add_line("\n".to_string());
        buffer.add_line("Tick-by-Tick Execution Log:".to_string());
        buffer.add_line("-".repeat(60));

        for log in &result.tick_logs {
            buffer.add_line(format!("Tick {:02}:", log.tick));

            // Show Queue
            if log.ready_queue.is_empty() {
                buffer.add_line("  Queue: [Empty]".to_string());
            } else {
                buffer.add_line(format!("  Queue: {:?}", log.ready_queue));
            }

            // Show Processors
            // Sort keys for consistent output
            let mut keys: Vec<&String> = log.processor_states.keys().collect();
            keys.sort();

            for proc in keys {
                let state = match log.processor_states.get(proc) {
                    Some(state) => state,
                    None => unreachable!("Non-existent processor in log"),
                };
                buffer.add_line(format!("  {:<10}: {}", proc, state));
            }
            buffer.add_line("".to_string());
        }

        buffer.get()
    }
}
