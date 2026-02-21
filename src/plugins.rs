use crate::state::AppState;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    async fn handle(&self, input: &str, state: &AppState) -> Option<String>;
}

macro_rules! create_plugin {
    ($name:ident, $str_name:expr) => {
        pub struct $name;
        #[async_trait]
        impl Plugin for $name {
            fn name(&self) -> &'static str {
                $str_name
            }
            async fn handle(&self, _input: &str, _state: &AppState) -> Option<String> {
                None
            }
        }
    };
}

create_plugin!(Base64Plugin, "base64");
create_plugin!(CalcPlugin, "calc");
create_plugin!(ContactPlugin, "contact");
create_plugin!(ErrorPlugin, "error");
create_plugin!(HashPlugin, "hash");
create_plugin!(LogicPlugin, "logic");
create_plugin!(MemoryPlugin, "memory");
create_plugin!(NewsPlugin, "news");
create_plugin!(PasswordPlugin, "password");
create_plugin!(SummaryPlugin, "summary");
create_plugin!(SystemPlugin, "system");
create_plugin!(TimePlugin, "time");
create_plugin!(TodoPlugin, "todo");
create_plugin!(TranslatePlugin, "translate");
create_plugin!(WeatherPlugin, "weather");
create_plugin!(WebsiteStatusPlugin, "website_status");

pub fn load_dynamic_plugins(_dir: &str) -> Vec<Box<dyn Plugin>> {
    Vec::new()
}
