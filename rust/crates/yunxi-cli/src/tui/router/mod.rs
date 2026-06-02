pub type Route = String;

#[derive(Debug, Clone)]
pub enum RouteType {
    Home,
    Session(String),
    Workspace(String),
    Settings,
    Help,
}

pub struct Router {
    current_route: Route,
    history: Vec<Route>,
    history_index: usize,
}

impl Router {
    pub fn new() -> Self {
        Self {
            current_route: "/home".to_string(),
            history: vec!["/home".to_string()],
            history_index: 0,
        }
    }

    pub fn navigate(&mut self, route: Route) {
        if self.history_index < self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(route.clone());
        self.history_index = self.history.len() - 1;
        self.current_route = route;
    }

    pub fn go_back(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_route = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }

    pub fn go_forward(&mut self) -> bool {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.current_route = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }

    pub fn current_route(&self) -> &Route {
        &self.current_route
    }

    pub fn parse_route(&self) -> RouteType {
        match self.current_route.as_str() {
            "/home" => RouteType::Home,
            "/settings" => RouteType::Settings,
            "/help" => RouteType::Help,
            route if route.starts_with("/session/") => {
                let id = route.strip_prefix("/session/").unwrap_or("");
                RouteType::Session(id.to_string())
            }
            route if route.starts_with("/workspace/") => {
                let id = route.strip_prefix("/workspace/").unwrap_or("");
                RouteType::Workspace(id.to_string())
            }
            _ => RouteType::Home,
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
