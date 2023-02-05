use std::env;
use std::path::PathBuf;

pub struct EnvConfig {
    pub dir_workspace: PathBuf,
    pub dir_pd2dsy: PathBuf,
}

pub fn get_env_config() -> EnvConfig {
    // TODO: lazy_static trickery

    let env_var_dir_workspace =
        env::var("DIR_WORKSPACE").expect("Missing required env var: DIR_WORKSPACE");
    let env_var_dir_pd2dsy = env::var("DIR_PD2DSY").expect("Missing required env var: DIR_PD2DSY");

    EnvConfig {
        dir_workspace: PathBuf::from(env_var_dir_workspace),
        dir_pd2dsy: PathBuf::from(env_var_dir_pd2dsy),
    }
}
