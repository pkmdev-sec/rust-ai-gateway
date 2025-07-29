use crate::{
    RouterConfig, RouterContext, RouterError, RouterResult, StrategyMode, Target,
    ConditionalRouter, LoadBalancer,
};

#[derive(Debug, Clone)]
pub struct RoutingResult {
    pub name: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl From<&Target> for RoutingResult {
    fn from(target: &Target) -> Self {
        Self {
            name: target.name.clone(),
            provider: target.provider.clone(),
            api_key: target.api_key.clone(),
            metadata: target.metadata.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Router {
    config: RouterConfig,
}

impl Router {
    pub fn new(config: RouterConfig) -> Self {
        Self { config }
    }

    pub fn route<'a>(&'a self, context: &'a RouterContext) -> RouterResult<RoutingResult> {
        if self.config.targets.is_empty() {
            return Err(RouterError::NoTargets);
        }

        let selected_target = match self.config.mode {
            StrategyMode::Single => self.route_single(),
            StrategyMode::LoadBalance => self.route_load_balance(),
            StrategyMode::Fallback => self.route_fallback(),
            StrategyMode::Conditional => self.route_conditional(context),
        }?;

        Ok(RoutingResult::from(selected_target))
    }

    fn route_single(&self) -> RouterResult<&Target> {
        self.config.targets.first()
            .ok_or(RouterError::NoTargets)
    }

    fn route_load_balance(&self) -> RouterResult<&Target> {
        LoadBalancer::select_target(&self.config.targets)
    }

    fn route_fallback(&self) -> RouterResult<&Target> {
        // For fallback, we return the first target
        // In a real implementation, you'd try each target until one succeeds
        self.config.targets.first()
            .ok_or(RouterError::NoTargets)
    }

    fn route_conditional<'a>(&'a self, context: &'a RouterContext) -> RouterResult<&'a Target> {
        let strategy = self.config.strategy.as_ref()
            .ok_or(RouterError::NoConditions)?;

        let router = ConditionalRouter::new(&self.config.targets, context);
        router.resolve_target(&strategy.conditions, strategy.default_target.as_deref())
    }

    pub fn get_targets(&self) -> &[Target] {
        &self.config.targets
    }

    pub fn get_mode(&self) -> &StrategyMode {
        &self.config.mode
    }
}

#[cfg(feature = "async")]
pub mod async_router {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;

    pub type AsyncRouterResult<T> = Pin<Box<dyn Future<Output = RouterResult<T>> + Send>>;

    pub struct AsyncRouter {
        router: Router,
    }

    impl AsyncRouter {
        pub fn new(config: RouterConfig) -> Self {
            Self {
                router: Router::new(config),
            }
        }

        pub async fn route(&self, context: &RouterContext) -> RouterResult<RoutingResult> {
            // For now, just delegate to sync router
            // In a real implementation, you might want to add async health checks, etc.
            self.router.route(context)
        }

        pub async fn route_with_fallback(&self, context: &RouterContext) -> RouterResult<RoutingResult> {
            match self.router.config.mode {
                StrategyMode::Fallback => {
                    // Try each target in sequence until one succeeds
                    for target in &self.router.config.targets {
                        // In a real implementation, you'd make an actual request here
                        // For now, just return the first target
                        return Ok(RoutingResult::from(target));
                    }
                    Err(RouterError::NoValidTarget)
                }
                _ => self.route(context).await,
            }
        }
    }
}