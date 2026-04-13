#[derive(Hash, Eq, PartialEq, Clone)]
pub(crate) struct PipelineKey {
    pub program_id: usize,
    pub layout_id: usize,
    pub attribute_hash: u64,
    pub depth_stencil_state_hash: u64,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub(crate) struct BindGroupKey {
    pub program_id: usize,
    pub layout_id: usize,
    pub buffer_ids: Vec<usize>,
    pub texture_ids: Vec<usize>,
}
