use crate::{AppConfig, Config};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value, from_str};
use std::collections::HashMap;
use std::time::Duration;

pub struct ConfigClient {
    http_client: reqwest::Client,
    config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Res<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}
impl ConfigClient {
    pub fn new(config: &Config) -> Self {
        let http_client = reqwest::ClientBuilder::default()
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        ConfigClient {
            http_client,
            config: config.clone(),
        }
    }
    pub(crate) async fn load(&self) -> anyhow::Result<Configs> {
        let mut contents = vec![];
        for id in self.config.config_ids.iter() {
            contents.push(
                Self::load_config(
                    &self.http_client,
                    &self.config.server_addr,
                    &self.config.namespace,
                    id,
                )
                .await?,
            );
        }

        // 启动监听，监听配置变化
        self.start_watch().await?;

        // 启动补偿任务
        self.start_compensate().await?;

        Configs::from_contents(contents)
    }

    async fn load_config(
        http_client: &reqwest::Client,
        server_addr: &str,
        namespace: &str,
        config_id: &str,
    ) -> anyhow::Result<String> {
        let url = format!(
            "http://{}/config/get?namespace_id={}&id={}",
            server_addr, namespace, config_id
        );

        let response = http_client.get(url).send().await?;
        let result = response.json::<Res<HashMap<String, Value>>>().await?;
        if result.code != 0 {
            return Err(anyhow::anyhow!("fetch config failed: {}", result.msg));
        }

        match result.data {
            None => {
                bail!(
                    "no config id [{}] found in namespace [{}], please check if namespace or config id exists",
                    config_id,
                    namespace
                );
            }
            Some(data) => {
                let content = data.get("content").unwrap().as_str().unwrap();
                log::info!("config {} fetched", config_id);
                Ok(content.to_string())
            }
        }
    }

    async fn start_watch(&self) -> anyhow::Result<()> {
        let config_clone = self.config.clone();
        tokio::spawn(async move {
            log::info!(
                "start watch config changes in namespace: {}",
                config_clone.namespace
            );
            let hc = reqwest::ClientBuilder::default()
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap();
            let url = format!(
                "http://{}/config/watch?namespace_id={}",
                config_clone.server_addr, config_clone.namespace
            );
            loop {
                match hc.get(&url).send().await {
                    Ok(response) => {
                        if !response.status().is_success() {
                            log::error!("watch config failed, status: {}", response.status());
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                        let res = response.json::<Res<bool>>().await;
                        if let Err(e) = res.as_ref() {
                            log::error!("watch config changes error: {}", e.to_string());
                        }
                        let res = res.unwrap();
                        match res.data {
                            Some(true) => {
                                log::info!("config changed, reloading config");
                                let mut contents = vec![];
                                for id in config_clone.config_ids.iter() {
                                    contents.push(
                                        Self::load_config(
                                            &hc,
                                            &config_clone.server_addr,
                                            &config_clone.namespace,
                                            id,
                                        )
                                        .await
                                        .unwrap(),
                                    );
                                }
                                AppConfig::reload(Configs::from_contents(contents).unwrap());
                                log::info!("reloading config success");
                            }
                            Some(false) => {
                                log::info!("config no changed");
                            }
                            _ => {
                                log::error!(
                                    "expected server to return true or false, but actually got: {:?}",
                                    res.data
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("watch config changes error: {}", e.to_string());
                    }
                };
            }
        });
        Ok(())
    }

    async fn start_compensate(&self) -> anyhow::Result<()> {
        let config_clone = self.config.clone();
        let hc = self.http_client.clone();
        tokio::spawn(async move {
            log::info!(
                "start config compensate in namespace: {}",
                config_clone.namespace
            );

            loop {
                tokio::time::sleep(Duration::from_secs(45)).await;

                log::info!("starting sync config");
                let mut contents = vec![];
                for id in config_clone.config_ids.iter() {
                    contents.push(
                        Self::load_config(
                            &hc,
                            &config_clone.server_addr,
                            &config_clone.namespace,
                            id,
                        )
                        .await
                        .unwrap(),
                    );
                }
                AppConfig::reload(Configs::from_contents(contents).unwrap());
                log::info!("config sync success");
            }
        });
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    pub configs: HashMap<String, Value>,
}

impl Configs {
    pub fn from_contents(contents: Vec<String>) -> anyhow::Result<Self> {
        let mut merged_config = Value::Mapping(Mapping::new());

        // 依次解析并合并每个配置文件
        // 后面的配置会覆盖前面相同键的配置
        for content in contents {
            if !content.trim().is_empty() {
                let config_value: Value = from_str(&content)?;
                Self::merge_yaml_values(&mut merged_config, config_value);
            }
        }

        // 配置键展开
        let mut configs = HashMap::new();
        Self::flatten_yaml_value(&mut configs, "", &merged_config);

        Ok(Configs { configs })
    }

    /// 递归合并两个 YAML 值
    /// 后面的值会覆盖前面相同键的值
    fn merge_yaml_values(target: &mut Value, source: Value) {
        match (target, source) {
            // 当两个值都是映射类型时，递归合并
            (Value::Mapping(target_map), Value::Mapping(source_map)) => {
                for (key, value) in source_map {
                    if target_map.contains_key(&key) {
                        // 如果目标中已存在该key，则递归合并
                        Self::merge_yaml_values(target_map.get_mut(&key).unwrap(), value);
                    } else {
                        // 如果目标中不存在该key，则直接插入
                        target_map.insert(key, value);
                    }
                }
            }
            // 其他情况下，直接用源值覆盖目标值
            (target, source) => {
                *target = source;
            }
        }
    }

    /// 将嵌套的 YAML 值扁平化为键值对
    fn flatten_yaml_value(result: &mut HashMap<String, Value>, prefix: &str, value: &Value) {
        match value {
            Value::Mapping(mapping) => {
                for (key, val) in mapping {
                    let key_str = match key {
                        Value::String(s) => s.clone(),
                        Value::Number(num) => num.to_string(),
                        _ => "unknown".to_string(),
                    };

                    let new_prefix = if prefix.is_empty() {
                        key_str
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };

                    Self::flatten_yaml_value(result, &new_prefix, val);
                }
            }
            _ => {
                // 叶子节点
                result.insert(prefix.to_string(), value.clone());
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.configs.get(key)
    }

    #[allow(unused)]
    pub fn get_all(&self) -> &HashMap<String, Value> {
        &self.configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_config() {
        let contents = vec![
            r#"
            a: 1
            b: 2
            c:
              d: 3
              e: 4
              f:
                g: 0
            h:
              - 1
              - 2
            "#
            .to_string(),
            r#"
            a: 5
            b: 6
            c:
              d: 7
              e: 8
              f: x
            1: -1
            h:
              - 1
              - 3
            "#
            .to_string(),
        ];
        let config = Configs::from_contents(contents).unwrap();
        println!("{:?}", config);
        println!("{:?}", config.get("a"));
        println!("{:?}", config.get("h"));
    }
}
