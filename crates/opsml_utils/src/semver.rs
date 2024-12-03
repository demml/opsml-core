use core::num;
use opsml_error::error::VersionError;
use semver::{BuildMetadata, Prerelease, Version};
use std::str::FromStr;

pub struct VersionArgs {
    pub version: String,
    pub version_type: VersionType,
    pub pre: Option<String>,
    pub build: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum VersionType {
    Major,
    Minor,
    Patch,
    Pre,
    Build,
    PreBuild,
}

impl FromStr for VersionType {
    type Err = ();

    fn from_str(input: &str) -> Result<VersionType, Self::Err> {
        match input.to_lowercase().as_str() {
            "major" => Ok(VersionType::Major),
            "minor" => Ok(VersionType::Minor),
            "patch" => Ok(VersionType::Patch),
            "pre" => Ok(VersionType::Pre),
            "build" => Ok(VersionType::Build),
            "pre_build" => Ok(VersionType::PreBuild),
            _ => Err(()),
        }
    }
}

pub struct VersionValidator {}

impl VersionValidator {
    pub fn validate_version(version: &str) -> Result<(), VersionError> {
        match Version::parse(version) {
            Ok(_) => Ok(()),
            Err(e) => Err(VersionError::InvalidVersion(e.to_string())),
        }
    }

    pub fn bump_version(version_args: &VersionArgs) -> Result<String, VersionError> {
        // parse the version
        let mut version = match Version::parse(&version_args.version) {
            Ok(v) => v,
            Err(e) => return Err(VersionError::InvalidVersion(e.to_string())),
        };

        // get major minor patch
        let (major, minor, patch) = (version.major, version.minor, version.patch);

        // check if version type is major, minor, or patch. If not, return the version as is
        let mut new_version = match version_args.version_type {
            VersionType::Major => {
                version.major += 1;
                version.minor = 0;
                version.patch = 0;
                version
            }
            VersionType::Minor => {
                version.minor += 1;
                version.patch = 0;
                version
            }
            VersionType::Patch => {
                version.patch += 1;
                version
            }

            // we handle pre and build separately
            VersionType::Pre => Version::new(major, minor, patch),
            VersionType::Build => Version::new(major, minor, patch),
            VersionType::PreBuild => Version::new(major, minor, patch),
        };

        // its possible someone creates a major, minor, patch version with a pre or build, or both
        // in this case, we need to add the pre and build to the new version
        if let Some(pre) = &version_args.pre {
            new_version.pre = match Prerelease::new(pre) {
                Ok(p) => p,
                Err(e) => return Err(VersionError::InvalidPreRelease(e.to_string())),
            };
        }

        if let Some(build) = &version_args.build {
            new_version.build = match BuildMetadata::new(build) {
                Ok(b) => b,
                Err(e) => return Err(VersionError::InvalidPreRelease(e.to_string())),
            };
        }

        Ok(new_version.to_string())
    }

    pub fn sort_versions(versions: Vec<String>) -> Result<Vec<String>, VersionError> {
        let mut versions: Vec<Version> = versions
            .iter()
            .map(|v| Version::parse(v).map_err(|e| VersionError::InvalidVersion(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;

        versions.sort();

        Ok(versions.iter().map(|v| v.to_string()).collect())
    }
}

pub struct VersionBounds {
    pub lower_bound: Version,
    pub upper_bound: Version,
    pub no_upper_bound: bool,
    pub parser_type: VersionParser,
}

#[derive(PartialEq, Debug)]
pub enum VersionParser {
    Star,
    Caret,
    Tilde,
    Exact,
}

impl VersionParser {
    pub fn new(version: &str) -> Result<VersionParser, VersionError> {
        // check if version contains
        if version.contains('*') {
            Ok(VersionParser::Star)
        } else if version.contains('^') {
            Ok(VersionParser::Caret)
        } else if version.contains('~') {
            Ok(VersionParser::Tilde)
        } else {
            Ok(VersionParser::Exact)
        }
    }

    pub fn remove_version_prefix(&self, version: &str) -> String {
        match self {
            VersionParser::Star => version.replace("*", ""),
            VersionParser::Caret => version.replace("^", ""),
            VersionParser::Tilde => version.replace("~", ""),
            VersionParser::Exact => version.to_string(),
        }
    }

    pub fn get_version_to_search(&self, version: &str) -> Result<VersionBounds, VersionError> {
        let parser = VersionParser::new(version)?;
        let cleaned_version = parser.remove_version_prefix(version);

        println!("cleaned_version: {}", cleaned_version);

        // determine number of "." in the version and split into int parts
        let version_parts = cleaned_version
            .split(".")
            .map(|v| v.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?;

        let num_parts = version_parts.len();

        match parser {
            VersionParser::Star => {
                if num_parts == 0 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!("{}.0.0", version_parts[0]))
                            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!("{}.0.0", version_parts[0]))
                            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: true,
                        parser_type: VersionParser::Star,
                    })
                } else if num_parts == 1 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!("{}.0.0", version_parts[0]))
                            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!("{}.0.0", version_parts[0] + 1))
                            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: false,
                        parser_type: VersionParser::Star,
                    })
                } else if num_parts == 2 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!(
                            "{}.{}.0",
                            version_parts[0], version_parts[1]
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!(
                            "{}.{}.0",
                            version_parts[0],
                            version_parts[1] + 1
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: false,
                        parser_type: VersionParser::Star,
                    })
                } else {
                    Err(VersionError::InvalidVersion(
                        "Invalid version provided with * syntax".to_string(),
                    ))
                }
            }
            VersionParser::Tilde => {
                if num_parts == 0 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!("{}.0.0", version_parts[0]))
                            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!("{}.0.0", version_parts[0] + 1))
                            .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: false,
                        parser_type: VersionParser::Tilde,
                    })
                } else if num_parts == 1 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!(
                            "{}.{}.0",
                            version_parts[0], version_parts[1]
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!(
                            "{}.{}.0",
                            version_parts[0],
                            version_parts[1] + 1
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: false,
                        parser_type: VersionParser::Tilde,
                    })
                } else if num_parts == 2 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!(
                            "{}.{}.{}",
                            version_parts[0], version_parts[1], version_parts[2]
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!(
                            "{}.{}.0",
                            version_parts[0],
                            version_parts[1] + 1,
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: false,
                        parser_type: VersionParser::Tilde,
                    })
                } else {
                    Err(VersionError::InvalidVersion(
                        "Invalid version provided with ~ syntax".to_string(),
                    ))
                }
            }
            VersionParser::Caret => {
                // must bea  full semver version for caret
                if num_parts >= 2 {
                    Ok(VersionBounds {
                        lower_bound: Version::parse(&format!(
                            "{}.{}.{}",
                            version_parts[0], version_parts[1], version_parts[2]
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        upper_bound: Version::parse(&format!(
                            "{}.{}.0",
                            version_parts[0],
                            version_parts[1] + 1,
                        ))
                        .map_err(|e| VersionError::InvalidVersion(e.to_string()))?,
                        no_upper_bound: false,
                        parser_type: VersionParser::Caret,
                    })
                } else {
                    Err(VersionError::InvalidVersion(
                        "Invalid version provided with ^ syntax".to_string(),
                    ))
                }
            }
            VersionParser::Exact => Version::parse(&cleaned_version)
                .map(|v| VersionBounds {
                    lower_bound: v.clone(),
                    upper_bound: v,
                    no_upper_bound: true,
                    parser_type: VersionParser::Exact,
                })
                .map_err(|e| VersionError::InvalidVersion(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn test_version_validator_validate_version() {
        assert!(VersionValidator::validate_version("1.2.3").is_ok());
        assert!(VersionValidator::validate_version("invalid.version").is_err());
    }

    #[test]
    fn test_version_validator_bump_version() {
        let args = VersionArgs {
            version: "1.2.3".to_string(),
            version_type: VersionType::Major,
            pre: None,
            build: None,
        };
        assert_eq!(VersionValidator::bump_version(&args).unwrap(), "2.0.0");

        let args = VersionArgs {
            version: "1.2.3".to_string(),
            version_type: VersionType::Minor,
            pre: None,
            build: None,
        };
        assert_eq!(VersionValidator::bump_version(&args).unwrap(), "1.3.0");

        let args = VersionArgs {
            version: "1.2.3".to_string(),
            version_type: VersionType::Patch,
            pre: None,
            build: None,
        };
        assert_eq!(VersionValidator::bump_version(&args).unwrap(), "1.2.4");

        let args = VersionArgs {
            version: "1.2.3".to_string(),
            version_type: VersionType::Pre,
            pre: Some("alpha".to_string()),
            build: None,
        };
        assert_eq!(
            VersionValidator::bump_version(&args).unwrap(),
            "1.2.3-alpha"
        );

        let args = VersionArgs {
            version: "1.2.3".to_string(),
            version_type: VersionType::Build,
            pre: None,
            build: Some("001".to_string()),
        };
        assert_eq!(VersionValidator::bump_version(&args).unwrap(), "1.2.3+001");

        let args = VersionArgs {
            version: "1.2.3".to_string(),
            version_type: VersionType::PreBuild,
            pre: Some("alpha".to_string()),
            build: Some("001".to_string()),
        };
        assert_eq!(
            VersionValidator::bump_version(&args).unwrap(),
            "1.2.3-alpha+001"
        );
    }

    #[test]
    fn test_version_validator_sort_versions() {
        let versions = vec![
            "1.2.1".to_string(),
            "1.3.0".to_string(),
            "1.2.2".to_string(),
            "1.2.3-alpha+001".to_string(),
            "1.2.3+001".to_string(),
            "1.2.3+0b1".to_string(),
            "1.2.3".to_string(),
        ];
        let sorted_versions = VersionValidator::sort_versions(versions).unwrap();
        assert_eq!(
            sorted_versions,
            vec![
                "1.2.1",
                "1.2.2",
                "1.2.3-alpha+001",
                "1.2.3",
                "1.2.3+001",
                "1.2.3+0b1",
                "1.3.0"
            ]
        );
    }

    #[test]
    fn test_version_parser_new() {
        assert_eq!(VersionParser::new("*").unwrap(), VersionParser::Star);
        assert_eq!(VersionParser::new("^1.2.3").unwrap(), VersionParser::Caret);
        assert_eq!(VersionParser::new("~1.2.3").unwrap(), VersionParser::Tilde);
        assert_eq!(VersionParser::new("1.2.3").unwrap(), VersionParser::Exact);
    }

    #[test]
    fn test_version_parser_remove_version_prefix() {
        assert_eq!(VersionParser::Star.remove_version_prefix("*"), "");
        assert_eq!(
            VersionParser::Caret.remove_version_prefix("^1.2.3"),
            "1.2.3"
        );
        assert_eq!(
            VersionParser::Tilde.remove_version_prefix("~1.2.3"),
            "1.2.3"
        );
        assert_eq!(VersionParser::Exact.remove_version_prefix("1.2.3"), "1.2.3");
    }

    #[test]
    fn test_version_parser_get_version_to_search() {
        let bounds = VersionParser::Star.get_version_to_search("*").unwrap();
        assert_eq!(bounds.lower_bound, Version::parse("0.0.0").unwrap());
        assert!(bounds.no_upper_bound);
    }
}
