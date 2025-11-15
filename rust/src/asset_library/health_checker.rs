use super::asset_source_registry::{AssetSourceRegistry, SourceHealth};
use super::repository_adapter::RepositoryAdapter;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Health check result for a single source
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Source name
    pub source_name: String,
    /// Whether the check was successful
    pub success: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp of the check
    pub timestamp: SystemTime,
}

impl HealthCheckResult {
    /// Creates a successful health check result
    pub fn success(source_name: String, response_time_ms: u64) -> Self {
        Self {
            source_name,
            success: true,
            response_time_ms,
            error: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Creates a failed health check result
    pub fn failure(source_name: String, error: String) -> Self {
        Self {
            source_name,
            success: false,
            response_time_ms: 0,
            error: Some(error),
            timestamp: SystemTime::now(),
        }
    }
}

/// Health check history for a source
#[derive(Debug, Clone)]
pub struct HealthHistory {
    /// Recent health check results (limited to last N checks)
    results: Vec<HealthCheckResult>,
    /// Maximum number of results to keep
    max_results: usize,
    /// Running average response time
    avg_response_time: f64,
    /// Success rate (0.0 to 1.0)
    success_rate: f64,
}

impl HealthHistory {
    /// Creates a new health history tracker
    pub fn new(max_results: usize) -> Self {
        Self {
            results: Vec::new(),
            max_results,
            avg_response_time: 0.0,
            success_rate: 1.0,
        }
    }

    /// Adds a new health check result
    pub fn add_result(&mut self, result: HealthCheckResult) {
        self.results.push(result);

        // Keep only the most recent results
        if self.results.len() > self.max_results {
            self.results.remove(0);
        }

        // Recalculate metrics
        self.calculate_metrics();
    }

    /// Calculates average response time and success rate
    fn calculate_metrics(&mut self) {
        if self.results.is_empty() {
            return;
        }

        let successful_results: Vec<&HealthCheckResult> = self
            .results
            .iter()
            .filter(|r| r.success)
            .collect();

        // Calculate success rate
        self.success_rate = successful_results.len() as f64 / self.results.len() as f64;

        // Calculate average response time (only from successful checks)
        if !successful_results.is_empty() {
            let total_time: u64 = successful_results
                .iter()
                .map(|r| r.response_time_ms)
                .sum();
            self.avg_response_time = total_time as f64 / successful_results.len() as f64;
        }
    }

    /// Gets the most recent health check result
    pub fn latest(&self) -> Option<&HealthCheckResult> {
        self.results.last()
    }

    /// Gets the average response time
    pub fn average_response_time(&self) -> f64 {
        self.avg_response_time
    }

    /// Gets the success rate
    pub fn success_rate(&self) -> f64 {
        self.success_rate
    }

    /// Gets all results
    pub fn results(&self) -> &[HealthCheckResult] {
        &self.results
    }
}

/// Circuit breaker states for source health management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for a source
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state
    state: CircuitState,
    /// Number of consecutive failures
    failure_count: u32,
    /// Threshold for opening the circuit
    failure_threshold: u32,
    /// When the circuit was opened
    opened_at: Option<SystemTime>,
    /// How long to wait before trying half-open
    timeout: Duration,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            opened_at: None,
            timeout,
        }
    }

    /// Records a successful operation
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.opened_at = None;
    }

    /// Records a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            self.opened_at = Some(SystemTime::now());
        }
    }

    /// Checks if a request should be allowed
    pub fn should_allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(opened_at) = self.opened_at {
                    if let Ok(elapsed) = SystemTime::now().duration_since(opened_at) {
                        if elapsed >= self.timeout {
                            self.state = CircuitState::HalfOpen;
                            return true;
                        }
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Gets the current state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Gets the failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
}

/// Health checker service
pub struct HealthChecker {
    /// Health history for each source
    history: HashMap<String, HealthHistory>,
    /// Circuit breakers for each source
    circuit_breakers: HashMap<String, CircuitBreaker>,
    /// Default circuit breaker settings
    failure_threshold: u32,
    circuit_timeout: Duration,
    /// Maximum history to keep per source
    max_history: usize,
}

impl HealthChecker {
    /// Creates a new health checker
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            circuit_breakers: HashMap::new(),
            failure_threshold: 3,
            circuit_timeout: Duration::from_secs(60),
            max_history: 10,
        }
    }

    /// Configures the circuit breaker settings
    pub fn configure(
        mut self,
        failure_threshold: u32,
        circuit_timeout: Duration,
        max_history: usize,
    ) -> Self {
        self.failure_threshold = failure_threshold;
        self.circuit_timeout = circuit_timeout;
        self.max_history = max_history;
        self
    }

    /// Performs a health check on a source using its adapter
    pub fn check_source(
        &mut self,
        source_name: &str,
        adapter: &dyn RepositoryAdapter,
    ) -> HealthCheckResult {
        // Get or create circuit breaker
        let circuit_breaker = self.circuit_breakers.entry(source_name.to_string()).or_insert_with(|| {
            CircuitBreaker::new(self.failure_threshold, self.circuit_timeout)
        });

        // Check if circuit breaker allows request
        if !circuit_breaker.should_allow_request() {
            let result = HealthCheckResult::failure(
                source_name.to_string(),
                "Circuit breaker is open".to_string(),
            );
            self.record_result(result.clone());
            return result;
        }

        // Perform actual health check
        let start_time = SystemTime::now();
        let check_result = adapter.health_check();
        let response_time = SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        let result = match check_result {
            Ok(true) => {
                circuit_breaker.record_success();
                HealthCheckResult::success(source_name.to_string(), response_time)
            }
            Ok(false) => {
                circuit_breaker.record_failure();
                HealthCheckResult::failure(
                    source_name.to_string(),
                    "Health check returned false".to_string(),
                )
            }
            Err(e) => {
                circuit_breaker.record_failure();
                HealthCheckResult::failure(source_name.to_string(), e)
            }
        };

        self.record_result(result.clone());
        result
    }

    /// Records a health check result
    fn record_result(&mut self, result: HealthCheckResult) {
        let history = self
            .history
            .entry(result.source_name.clone())
            .or_insert_with(|| HealthHistory::new(self.max_history));

        history.add_result(result);
    }

    /// Gets health history for a source
    pub fn get_history(&self, source_name: &str) -> Option<&HealthHistory> {
        self.history.get(source_name)
    }

    /// Gets circuit breaker state for a source
    pub fn get_circuit_state(&self, source_name: &str) -> Option<CircuitState> {
        self.circuit_breakers
            .get(source_name)
            .map(|cb| cb.state())
    }

    /// Gets health statistics for all sources
    pub fn get_stats(&self) -> HashMap<String, SourceHealthStats> {
        self.history
            .iter()
            .map(|(name, history)| {
                let stats = SourceHealthStats {
                    source_name: name.clone(),
                    success_rate: history.success_rate(),
                    avg_response_time: history.average_response_time(),
                    last_check: history.latest().map(|r| r.timestamp),
                    circuit_state: self
                        .get_circuit_state(name)
                        .unwrap_or(CircuitState::Closed),
                };
                (name.clone(), stats)
            })
            .collect()
    }

    /// Clears history for a source
    pub fn clear_history(&mut self, source_name: &str) {
        self.history.remove(source_name);
        self.circuit_breakers.remove(source_name);
    }

    /// Resets circuit breaker for a source
    pub fn reset_circuit(&mut self, source_name: &str) {
        if let Some(circuit) = self.circuit_breakers.get_mut(source_name) {
            circuit.record_success();
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about source health
#[derive(Debug, Clone)]
pub struct SourceHealthStats {
    pub source_name: String,
    pub success_rate: f64,
    pub avg_response_time: f64,
    pub last_check: Option<SystemTime>,
    pub circuit_state: CircuitState,
}

/// Fallback orchestrator that coordinates health checking and source selection
pub struct FallbackOrchestrator {
    health_checker: HealthChecker,
    registry: AssetSourceRegistry,
}

impl FallbackOrchestrator {
    /// Creates a new fallback orchestrator
    pub fn new(registry: AssetSourceRegistry) -> Self {
        Self {
            health_checker: HealthChecker::new(),
            registry,
        }
    }

    /// Configures health checker settings
    pub fn configure_health_checker(
        mut self,
        failure_threshold: u32,
        circuit_timeout: Duration,
        max_history: usize,
    ) -> Self {
        self.health_checker = self
            .health_checker
            .configure(failure_threshold, circuit_timeout, max_history);
        self
    }

    /// Performs health checks on all enabled sources
    pub fn check_all_sources(&mut self, adapters: &HashMap<String, Box<dyn RepositoryAdapter>>) {
        // Collect source names first to avoid borrow checker issues
        let source_names: Vec<String> = self
            .registry
            .get_enabled_sources()
            .into_iter()
            .map(|s| s.source.name.clone())
            .collect();

        for source_name in source_names {
            if let Some(adapter) = adapters.get(&source_name) {
                let result = self
                    .health_checker
                    .check_source(&source_name, adapter.as_ref());

                // Update registry based on health check result
                if result.success {
                    self.registry.record_success(&source_name);
                } else {
                    self.registry.record_failure(&source_name);
                }
            }
        }
    }

    /// Gets the best available source for a request, using fallbacks if needed
    pub fn get_best_source(&self, preferred_source: Option<&str>) -> Option<String> {
        // If preferred source is specified and healthy, use it
        if let Some(pref) = preferred_source {
            if let Some(source) = self.registry.get_source(pref) {
                if source.health == SourceHealth::Healthy && source.source.enabled {
                    if let Some(CircuitState::Closed) =
                        self.health_checker.get_circuit_state(pref)
                    {
                        return Some(pref.to_string());
                    }
                }
            }
        }

        // Get healthy sources sorted by priority
        let healthy_sources = self.registry.get_healthy_sources();

        for source in healthy_sources {
            // Check circuit breaker state
            if let Some(state) = self.health_checker.get_circuit_state(&source.source.name) {
                if state == CircuitState::Closed || state == CircuitState::HalfOpen {
                    return Some(source.source.name.clone());
                }
            } else {
                // No circuit breaker info, assume healthy
                return Some(source.source.name.clone());
            }
        }

        None
    }

    /// Gets fallback sources when a specific source fails
    pub fn get_fallbacks(&self, failed_source: &str) -> Vec<String> {
        let fallback_sources = self.registry.get_fallback_sources(failed_source);

        fallback_sources
            .into_iter()
            .filter(|source| {
                // Only include sources with closed or half-open circuits
                if let Some(state) = self.health_checker.get_circuit_state(&source.source.name) {
                    state == CircuitState::Closed || state == CircuitState::HalfOpen
                } else {
                    true
                }
            })
            .map(|source| source.source.name.clone())
            .collect()
    }

    /// Gets the health checker
    pub fn health_checker(&self) -> &HealthChecker {
        &self.health_checker
    }

    /// Gets mutable health checker
    pub fn health_checker_mut(&mut self) -> &mut HealthChecker {
        &mut self.health_checker
    }

    /// Gets the registry
    pub fn registry(&self) -> &AssetSourceRegistry {
        &self.registry
    }

    /// Gets mutable registry
    pub fn registry_mut(&mut self) -> &mut AssetSourceRegistry {
        &mut self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result() {
        let success = HealthCheckResult::success("test".to_string(), 100);
        assert!(success.success);
        assert_eq!(success.response_time_ms, 100);

        let failure = HealthCheckResult::failure("test".to_string(), "error".to_string());
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[test]
    fn test_health_history() {
        let mut history = HealthHistory::new(5);

        history.add_result(HealthCheckResult::success("test".to_string(), 100));
        history.add_result(HealthCheckResult::success("test".to_string(), 200));
        history.add_result(HealthCheckResult::failure(
            "test".to_string(),
            "error".to_string(),
        ));

        assert_eq!(history.results().len(), 3);
        assert!(history.success_rate() > 0.6 && history.success_rate() < 0.7);
        assert!(history.average_response_time() > 0.0);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.should_allow_request());

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.should_allow_request());

        // Record success should close circuit
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.should_allow_request());
    }

    #[test]
    fn test_health_checker() {
        let mut checker = HealthChecker::new();

        let result = HealthCheckResult::success("source1".to_string(), 150);
        checker.record_result(result);

        let history = checker.get_history("source1");
        assert!(history.is_some());
        assert_eq!(history.unwrap().results().len(), 1);
    }
}
