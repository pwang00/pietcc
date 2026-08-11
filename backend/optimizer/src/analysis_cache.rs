use crate::result::ExecutionResult;

#[derive(Debug, Clone)]
pub struct AnalysisCache {
    pub(crate) result: Option<ExecutionResult>,
}

impl AnalysisCache {
    pub fn get_cached_result(&self) -> Option<&ExecutionResult> {
        self.result.as_ref()
    }

    pub fn update_result(&mut self, result: ExecutionResult) {
        self.result = Some(result)
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self { result: None }
    }
}
