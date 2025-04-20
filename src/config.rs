use anyhow::Result;
use std::path::PathBuf;

#[derive(knus::Decode, Default, Debug)]
pub struct Config {
    #[knus(child)]
    pub options: Options,
    #[knus(children)]
    pub redis_list: Vec<Redis>,
}

#[derive(knus::Decode, Default, Debug)]
pub struct Options {
    #[knus(child)]
    pub debug: bool,
    #[knus(child, unwrap(argument))]
    pub listen: String,
    #[knus(child, unwrap(argument))]
    pub metric_prefix: String,
}

#[derive(knus::Decode, Default, Debug)]
pub struct Redis {
    #[knus(argument)]
    pub inst: String,
    #[knus(child, unwrap(argument))]
    pub host: String,
    #[knus(child, unwrap(argument))]
    pub port: u16,
    #[knus(child, unwrap(argument))]
    pub poll_interval: u32,
    #[knus(child)]
    pub tls: bool,
}

pub fn parse_config(config_path: &PathBuf) -> Result<Config> {
    let content = std::fs::read_to_string(config_path)?;
    let config = match knus::parse::<Config>(&config_path.display().to_string(), content.as_str()) {
        Ok(x) => x,
        Err(e) => {
            println!("{:?}", miette::Report::new(e));
            std::process::exit(1);
        }
    };
    Ok(config)
}
