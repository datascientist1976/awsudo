extern crate clap;

use self::clap::{App, AppSettings, Arg, ArgMatches};

const AWS_DEFAULT_CONFIG_PATH: &str = ".aws/config";
const AWS_DEFAULT_CACHE_DIR: &str = ".awsudo/";

pub struct CLI {
    pub user: String,
    pub command: String,
    pub config: std::path::PathBuf,
    pub cache_dir: std::path::PathBuf,
}

pub fn parse() -> CLI {
    from_args(default().get_matches())
}

fn from_args(matches: ArgMatches) -> CLI {
    let user = String::from(matches.value_of("user").unwrap_or("default"));
    let config = matches
        .value_of("config")
        .map(|s| std::path::PathBuf::from(s))
        .or(dirs::home_dir().map(|path| path.join(AWS_DEFAULT_CONFIG_PATH)))
        .expect("Something wrong with config");

    let cache_dir = matches
        .value_of("cache_dir")
        .map(|s| std::path::PathBuf::from(s))
        .or(dirs::runtime_dir().map(|path| path.join(AWS_DEFAULT_CACHE_DIR)))
        .or(dirs::home_dir().map(|path| path.join(AWS_DEFAULT_CACHE_DIR)))
        .expect("Something wrong with cache_dir");

    let command = match matches.subcommand() {
        (external, maybe_matches) => {
            let args = match maybe_matches {
                Some(external_matches) => match external_matches.values_of("") {
                    Some(values) => values.collect::<Vec<&str>>().join(" "),
                    None => String::from(""),
                },
                _ => String::from(""),
            };

            String::from(vec![String::from(external), args].join(" ").trim())
        }
    };

    CLI {
        user,
        config,
        command,
        cache_dir,
    }
}

fn default<'b, 'c>() -> App<'b, 'c> {
    App::new("awsudo - sudo-like behavior for role assumed access on AWS accounts")
        .version(clap::crate_version!())
        .setting(AppSettings::AllowExternalSubcommands)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Custom config file, defaults to: ~/.aws/config")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cache_dir")
                .long("cache-dir")
                .value_name("DIR")
                .help("Custom directory for credentials caching, defaults to ~/.awsudo/")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("user")
                .help("AWS profile name based on the config file")
                .required(true)
                .takes_value(true),
        )
}

#[cfg(test)]
mod tests {
    use awsudo::cli;
    use std::path::PathBuf;

    #[test]
    fn it_parses_user() {
        let result = cli::from_args(cli::default().get_matches_from(vec!["awsudo", "-u", "jeff"]));

        assert_eq!(result.user, "jeff");
    }

    #[test]
    fn it_sets_default_cache_dir() {
        let result = cli::from_args(cli::default().get_matches_from(vec!["awsudo", "-u", "jeff"]));

        let dir = match dirs::runtime_dir() {
            None => dirs::home_dir().unwrap().join(".awsudo/"),
            Some(p) => p.join(".awsudo/"),
        };

        assert_eq!(result.cache_dir, dir);
    }

    #[test]
    fn it_accepts_cache_dir_option() {
        let result = cli::from_args(cli::default().get_matches_from(vec![
            "awsudo",
            "-u",
            "jeff",
            "--cache-dir",
            "/foo/bar",
        ]));

        assert_eq!(result.cache_dir, PathBuf::from("/foo/bar"));
    }

    #[test]
    fn it_parses_config() {
        let result = cli::from_args(cli::default().get_matches_from(vec![
            "awsudo",
            "-u",
            "jeff",
            "-c",
            "/usr/specific/path",
        ]));

        assert_eq!(result.config, PathBuf::from("/usr/specific/path"));
    }

    #[test]
    fn it_parses_single_command() {
        let result =
            cli::from_args(cli::default().get_matches_from(vec!["awsudo", "-u", "jeff", "echo"]));

        assert_eq!(result.command, "echo");
    }

    #[test]
    fn it_parses_command_with_multiple_words() {
        let result = cli::from_args(
            cli::default().get_matches_from(vec!["awsudo", "-u", "jeff", "echo", "bezos", "aws"]),
        );

        assert_eq!(result.command, "echo bezos aws");
    }

    #[test]
    fn it_parses_command_with_attribute() {
        let result = cli::from_args(
            cli::default().get_matches_from(vec!["awsudo", "-u", "jeff", "ls", "-a"]),
        );

        assert_eq!(result.command, "ls -a");
    }

    #[test]
    fn it_parses_command_with_multiple_attributes() {
        let result = cli::from_args(
            cli::default().get_matches_from(vec!["awsudo", "-u", "jeff", "ls", "-a", "-l"]),
        );

        assert_eq!(result.command, "ls -a -l");
    }
}
