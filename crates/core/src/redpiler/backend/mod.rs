pub mod direct;

use std::sync::Arc;

use super::compile_graph::CompileGraph;
use super::task_monitor::TaskMonitor;
use crate::world::World;
use enum_dispatch::enum_dispatch;
use mchprs_blocks::BlockPos;
use mchprs_world::TickEntry;

#[enum_dispatch]
pub trait JITBackend {
    fn compile(&mut self, graph: CompileGraph, ticks: Vec<TickEntry>, monitor: Arc<TaskMonitor>);
    fn tick(&mut self);
    fn on_use_block(&mut self, pos: BlockPos);
    fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool);
    fn flush<W: World>(&mut self, world: &mut W, io_only: bool);
    fn reset<W: World>(&mut self, world: &mut W, io_only: bool);
    /// Inspect block for debugging
    fn inspect(&mut self, pos: BlockPos);
}

#[cfg(feature = "jit_cranelift")]
use cranelift::CraneliftBackend;
use direct::DirectBackend;

#[enum_dispatch(JITBackend)]
pub enum BackendDispatcher {
    DirectBackend,
    #[cfg(feature = "jit_cranelift")]
    CraneliftBackend,
}

impl Default for BackendDispatcher {
    fn default() -> Self {
        Self::DirectBackend(Default::default())
    }
}
