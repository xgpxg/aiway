use crate::{Plugin, PluginError};
use protocol::gateway::{HttpContext, RequestContext};
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}
impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Default::default(),
        }
    }
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn run(&self, name: &str, context: &HttpContext) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.execute(context)
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::SV;
    use protocol::gateway::request_context::Method;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::thread::spawn;

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();

        struct TestPlugin;

        impl Plugin for TestPlugin {
            fn name(&self) -> &'static str {
                "test"
            }

            fn execute(&self, context: &HttpContext) -> Result<(), PluginError> {
                println!("execute test plugin");
                Ok(())
            }
        }
        impl TestPlugin {
            pub fn new() -> Self {
                Self {}
            }
        }
        manager.register(Box::new(TestPlugin::new()));

        let context = HttpContext {
            request: RequestContext {
                request_id: "".to_string(),
                method: SV::new(Method::Get),
                headers: Default::default(),
                path: Default::default(),
                query: Default::default(),
                body: Default::default(),
                state: Default::default(),
                route: Default::default(),
                routing_url: SV::empty(),
            },
            response: Default::default(),
        };
        manager.run("test", &context).unwrap();
    }

    #[test]
    fn test_load() {
        let path = "/home/wxg/work/project/aiway/target/release/libdemo_plugin.so";

        let p: Box<dyn Plugin> = Path::new(path).to_path_buf().try_into().unwrap();
        let p = Arc::new(Mutex::new(p)); // 如果插件有内部状态需要同步

        let context = RequestContext {
            request_id: "".to_string(),
            method: SV::new(Method::Get),
            headers: Default::default(),
            path: Default::default(),
            query: Default::default(),
            body: Default::default(),
            state: Default::default(),
            route: SV::empty(),
            routing_url: SV::empty(),
        };
        let context = Arc::new(HttpContext {
            request: context,
            response: Default::default(),
        });

        //p.lock().unwrap().execute(&*context).unwrap();

        let mut tasks = vec![];

        for _ in 0..100 {
            let context_clone = Arc::clone(&context);
            let p_clone = Arc::clone(&p);
            tasks.push(spawn(move || {
                for _ in 0..100 {
                    p_clone.lock().unwrap().execute(&*context_clone).unwrap();
                }
            }));
        }

        for task in tasks {
            task.join().unwrap();
        }

        println!("{:?}", context.response.status.get());
    }
}
