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
    pub fn get_version_to_search(&self, version: &str) -> Result<String, VersionError> {
        match self {
            VersionParser::Star => {
                // remove star, create version and return major
                // *1.2.0 -> 1.2.0 -> 1
                let cleaned = version.replace("*", "");
                match Version::parse(&cleaned) {
                    Ok(_) => Ok("".to_string()),
                    Err(e) => Err(VersionError::SemVerError(e.to_string())),
                }
            }
            VersionParser::Tilde => {
                // remove star, create version and return major
                // *1.2.0 -> 1.2.0 -> 1
                let cleaned = version.replace("~", "");

                match Version::parse(&cleaned) {
                    Ok(v) => {
                        let version = format!("{}.{}", v.major, v.minor);
                        Ok(version)
                    }
                    Err(e) => Err(VersionError::SemVerError(e.to_string())),
                }
            }
            VersionParser::Caret => {
                // remove star, create version and return major
                // *1.2.0 -> 1.2.0 -> 1
                let cleaned = version.replace("^", "");

                match Version::parse(&cleaned) {
                    Ok(v) => {
                        let version = format!("{}", v.major);
                        Ok(version)
                    }
                    Err(e) => Err(VersionError::SemVerError(e.to_string())),
                }
            }
            VersionParser::Exact => match Version::parse(version) {
                Ok(v) => Ok(v.to_string()),
                Err(e) => Err(VersionError::SemVerError(e.to_string())),
            },
        }
    }
}
