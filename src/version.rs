// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The `version` module gives you tools to create and compare SemVer-compliant
//! versions.

use std::cmp::{self, Ordering};
use std::fmt;
use std::hash;
use std::error::Error;

use std::result;
use std::str;

use parser;

/// An identifier in the pre-release or build metadata.
///
/// See sections 9 and 10 of the spec for more about pre-release identifers and
/// build metadata.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Identifier {
    /// An identifier that's solely numbers.
    Numeric(u64),
    /// An identifier with letters and numbers.
    AlphaNumeric(String),
}

impl fmt::Display for Identifier {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Identifier::Numeric(ref n) => fmt::Display::fmt(n, f),
            Identifier::AlphaNumeric(ref s) => fmt::Display::fmt(s, f),
        }
    }
}

/// Represents a version number conforming to the semantic versioning scheme.
#[derive(Clone, Eq, Debug)]
pub struct Version {
    /// The major version, to be incremented on incompatible changes.
    pub major: u64,
    /// The minor version, to be incremented when functionality is added in a
    /// backwards-compatible manner.
    pub minor: u64,
    /// The patch version, to be incremented when backwards-compatible bug
    /// fixes are made.
    pub patch: u64,
    /// The pre-release version identifier, if one exists.
    pub pre: Vec<Identifier>,
    /// The build metadata, ignored when determining version precedence.
    pub build: Vec<Identifier>,
}

/// An error type for this crate
///
/// Currently, just a generic error. Will make this nicer later.
#[derive(Clone,PartialEq,Debug,PartialOrd)]
pub enum SemVerError {
    /// An error ocurred while parsing.
    ParseError(String),
}

impl fmt::Display for SemVerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SemVerError::ParseError(ref m) => write!(f, "{}", m),
        }
    }
}

impl Error for SemVerError {
    fn description(&self) -> &str {
        match self {
            &SemVerError::ParseError(ref m) => m,
        }
    }
}

/// A Result type for errors
pub type Result<T> = result::Result<T, SemVerError>;

impl Version {
    /// Parse a string into a semver object.
    pub fn parse(version: &str) -> Result<Version> {
        let res = parser::version::try_parse(version.trim().as_bytes(), false);

        match res {
            // Convert plain String error into proper ParseError
            Err(e) => Err(SemVerError::ParseError(e)),
            Ok(v) => Ok(v),
        }
    }

    /// Parse a string into a semver object.
    pub fn best_efford_parse(version: &str) -> Result<Version> {
        let res = parser::version::try_parse(version.trim().as_bytes(), true);

        match res {
            // Convert plain String error into proper ParseError
            Err(e) => Err(SemVerError::ParseError(e)),
            Ok(v) => Ok(v),
        }
    }

    /// Clears the build metadata
    fn clear_metadata(&mut self) {
        self.build = Vec::new();
        self.pre = Vec::new();
    }

    /// Increments the patch number for this Version (Must be mutable)
    pub fn increment_patch(&mut self) {
        self.patch += 1;
        self.clear_metadata();
    }

    /// Increments the minor version number for this Version (Must be mutable)
    ///
    /// As instructed by section 7 of the spec, the patch number is reset to 0.
    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
        self.clear_metadata();
    }

    /// Increments the major version number for this Version (Must be mutable)
    ///
    /// As instructed by section 8 of the spec, the minor and patch numbers are
    /// reset to 0
    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
        self.clear_metadata();
    }

    /// Checks to see if the current Version is in pre-release status
    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }
}

impl str::FromStr for Version {
    type Err = SemVerError;

    fn from_str(s: &str) -> Result<Version> {
        Version::parse(s)
    }
}

impl fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}.{}.{}", self.major, self.minor, self.patch));
        if !self.pre.is_empty() {
            try!(write!(f, "-"));
            for (i, x) in self.pre.iter().enumerate() {
                if i != 0 {
                    try!(write!(f, "."))
                }
                try!(write!(f, "{}", x));
            }
        }
        if !self.build.is_empty() {
            try!(write!(f, "+"));
            for (i, x) in self.build.iter().enumerate() {
                if i != 0 {
                    try!(write!(f, "."))
                }
                try!(write!(f, "{}", x));
            }
        }
        Ok(())
    }
}

impl cmp::PartialEq for Version {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        // We should ignore build metadata here, otherwise versions v1 and v2
        // can exist such that !(v1 < v2) && !(v1 > v2) && v1 != v2, which
        // violate strict total ordering rules.
        self.major == other.major && self.minor == other.minor && self.patch == other.patch &&
        self.pre == other.pre
    }
}

impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            r => return r,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            r => return r,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            r => return r,
        }

        // NB: semver spec says 0.0.0-pre < 0.0.0
        // but the version of ord defined for vec
        // says that [] < [pre] so we alter it here
        match (self.pre.len(), other.pre.len()) {
            (0, 0) => Ordering::Equal,
            (0, _) => Ordering::Greater,
            (_, 0) => Ordering::Less,
            (_, _) => self.pre.cmp(&other.pre),
        }
    }
}

impl hash::Hash for Version {
    fn hash<H: hash::Hasher>(&self, into: &mut H) {
        self.major.hash(into);
        self.minor.hash(into);
        self.patch.hash(into);
        self.pre.hash(into);
    }
}

#[cfg(test)]
mod tests {
    use std::result;
    use super::Version;
    use super::Identifier;
    use super::SemVerError;

    #[test]
    fn test_parse() {
        fn parse_error(e: &str) -> result::Result<Version, SemVerError> {
            return Err(SemVerError::ParseError(e.to_string()));
        }

        assert_eq!(Version::parse(""), parse_error("Parse error"));
        assert_eq!(Version::parse("  "), parse_error("Parse error"));
        assert_eq!(Version::parse("1"), parse_error("Parse error"));
        assert_eq!(Version::parse("1.2"), parse_error("Parse error"));
        assert_eq!(Version::parse("1.2.3-"),
                   parse_error("Failed with unparsed input: '-'"));
        assert_eq!(Version::parse("a.b.c"), parse_error("Parse error"));
        assert_eq!(Version::parse("1.2.3 abc"),
                   parse_error("Failed with unparsed input: ' abc'"));

        assert_eq!(Version::parse("1.2.3"),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: Vec::new(),
                   }));
        assert_eq!(Version::parse("  1.2.3  "),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: Vec::new(),
                   }));
        assert_eq!(Version::parse("1.2.3-alpha1"),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: Vec::new(),
                   }));
        assert_eq!(Version::parse("  1.2.3-alpha1  "),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: Vec::new(),
                   }));
        assert_eq!(Version::parse("1.2.3+build5"),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!(Version::parse("  1.2.3+build5  "),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!(Version::parse("1.2.3-alpha1+build5"),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!(Version::parse("  1.2.3-alpha1+build5  "),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!(Version::parse("1.2.3-1.alpha1.9+build5.7.3aedf  "),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::Numeric(1),
                      Identifier::AlphaNumeric(String::from("alpha1")),
                      Identifier::Numeric(9),
            ],
                       build: vec![Identifier::AlphaNumeric(String::from("build5")),
                        Identifier::Numeric(7),
                        Identifier::AlphaNumeric(String::from("3aedf")),
            ],
                   }));
        assert_eq!(Version::parse("0.4.0-beta.1+0851523"),
                   Ok(Version {
                       major: 0,
                       minor: 4,
                       patch: 0,
                       pre: vec![Identifier::AlphaNumeric(String::from("beta")),
                      Identifier::Numeric(1),
            ],
                       build: vec![Identifier::AlphaNumeric(String::from("0851523"))],
                   }));

    }

    #[test]
    fn test_increment_patch() {
        let mut buggy_release = Version::parse("0.1.0").unwrap();
        buggy_release.increment_patch();
        assert_eq!(buggy_release, Version::parse("0.1.1").unwrap());
    }

    #[test]
    fn test_increment_minor() {
        let mut feature_release = Version::parse("1.4.6").unwrap();
        feature_release.increment_minor();
        assert_eq!(feature_release, Version::parse("1.5.0").unwrap());
    }

    #[test]
    fn test_increment_major() {
        let mut chrome_release = Version::parse("46.1.246773").unwrap();
        chrome_release.increment_major();
        assert_eq!(chrome_release, Version::parse("47.0.0").unwrap());
    }

    #[test]
    fn test_increment_keep_prerelease() {
        let mut release = Version::parse("1.0.0-alpha").unwrap();
        release.increment_patch();

        assert_eq!(release, Version::parse("1.0.1").unwrap());

        release.increment_minor();

        assert_eq!(release, Version::parse("1.1.0").unwrap());

        release.increment_major();

        assert_eq!(release, Version::parse("2.0.0").unwrap());
    }


    #[test]
    fn test_increment_clear_metadata() {
        let mut release = Version::parse("1.0.0+4442").unwrap();
        release.increment_patch();

        assert_eq!(release, Version::parse("1.0.1").unwrap());
        release = Version::parse("1.0.1+hello").unwrap();

        release.increment_minor();

        assert_eq!(release, Version::parse("1.1.0").unwrap());
        release = Version::parse("1.1.3747+hello").unwrap();

        release.increment_major();

        assert_eq!(release, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_eq() {
        assert_eq!(Version::parse("1.2.3"), Version::parse("1.2.3"));
        assert_eq!(Version::parse("1.2.3-alpha1"),
                   Version::parse("1.2.3-alpha1"));
        assert_eq!(Version::parse("1.2.3+build.42"),
                   Version::parse("1.2.3+build.42"));
        assert_eq!(Version::parse("1.2.3-alpha1+42"),
                   Version::parse("1.2.3-alpha1+42"));
        assert_eq!(Version::parse("1.2.3+23"), Version::parse("1.2.3+42"));
    }

    #[test]
    fn test_ne() {
        assert!(Version::parse("0.0.0") != Version::parse("0.0.1"));
        assert!(Version::parse("0.0.0") != Version::parse("0.1.0"));
        assert!(Version::parse("0.0.0") != Version::parse("1.0.0"));
        assert!(Version::parse("1.2.3-alpha") != Version::parse("1.2.3-beta"));
    }

    #[test]
    fn test_show() {
        assert_eq!(format!("{}", Version::parse("1.2.3").unwrap()),
                   "1.2.3".to_string());
        assert_eq!(format!("{}", Version::parse("1.2.3-alpha1").unwrap()),
                   "1.2.3-alpha1".to_string());
        assert_eq!(format!("{}", Version::parse("1.2.3+build.42").unwrap()),
                   "1.2.3+build.42".to_string());
        assert_eq!(format!("{}", Version::parse("1.2.3-alpha1+42").unwrap()),
                   "1.2.3-alpha1+42".to_string());
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Version::parse("1.2.3").unwrap().to_string(),
                   "1.2.3".to_string());
        assert_eq!(Version::parse("1.2.3-alpha1").unwrap().to_string(),
                   "1.2.3-alpha1".to_string());
        assert_eq!(Version::parse("1.2.3+build.42").unwrap().to_string(),
                   "1.2.3+build.42".to_string());
        assert_eq!(Version::parse("1.2.3-alpha1+42").unwrap().to_string(),
                   "1.2.3-alpha1+42".to_string());
    }

    #[test]
    fn test_lt() {
        assert!(Version::parse("0.0.0") < Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.0.0") < Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.0") < Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3-alpha1") < Version::parse("1.2.3"));
        assert!(Version::parse("1.2.3-alpha1") < Version::parse("1.2.3-alpha2"));
        assert!(!(Version::parse("1.2.3-alpha2") < Version::parse("1.2.3-alpha2")));
        assert!(!(Version::parse("1.2.3+23") < Version::parse("1.2.3+42")));
    }

    #[test]
    fn test_le() {
        assert!(Version::parse("0.0.0") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.0.0") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.0") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3-alpha1") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3-alpha2") <= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3+23") <= Version::parse("1.2.3+42"));
    }

    #[test]
    fn test_gt() {
        assert!(Version::parse("1.2.3-alpha2") > Version::parse("0.0.0"));
        assert!(Version::parse("1.2.3-alpha2") > Version::parse("1.0.0"));
        assert!(Version::parse("1.2.3-alpha2") > Version::parse("1.2.0"));
        assert!(Version::parse("1.2.3-alpha2") > Version::parse("1.2.3-alpha1"));
        assert!(Version::parse("1.2.3") > Version::parse("1.2.3-alpha2"));
        assert!(!(Version::parse("1.2.3-alpha2") > Version::parse("1.2.3-alpha2")));
        assert!(!(Version::parse("1.2.3+23") > Version::parse("1.2.3+42")));
    }

    #[test]
    fn test_ge() {
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("0.0.0"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.0.0"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.2.0"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.2.3-alpha1"));
        assert!(Version::parse("1.2.3-alpha2") >= Version::parse("1.2.3-alpha2"));
        assert!(Version::parse("1.2.3+23") >= Version::parse("1.2.3+42"));
    }

    #[test]
    fn test_prerelease_check() {
        assert!(Version::parse("1.0.0").unwrap().is_prerelease() == false);
        assert!(Version::parse("0.0.1").unwrap().is_prerelease() == false);
        assert!(Version::parse("4.1.4-alpha").unwrap().is_prerelease());
        assert!(Version::parse("1.0.0-beta294296").unwrap().is_prerelease());
    }

    #[test]
    fn test_spec_order() {
        let vs = ["1.0.0-alpha",
                  "1.0.0-alpha.1",
                  "1.0.0-alpha.beta",
                  "1.0.0-beta",
                  "1.0.0-beta.2",
                  "1.0.0-beta.11",
                  "1.0.0-rc.1",
                  "1.0.0"];
        let mut i = 1;
        while i < vs.len() {
            let a = Version::parse(vs[i - 1]);
            let b = Version::parse(vs[i]);
            assert!(a < b, "nope {:?} < {:?}", a, b);
            i += 1;
        }
    }

    #[test]
    fn test_from_str() {
        assert_eq!("1.2.3".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: Vec::new(),
                   }));
        assert_eq!("  1.2.3  ".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: Vec::new(),
                   }));
        assert_eq!("1.2.3-alpha1".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: Vec::new(),
                   }));
        assert_eq!("  1.2.3-alpha1  ".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: Vec::new(),
                   }));
        assert_eq!("1.2.3+build5".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!("  1.2.3+build5  ".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: Vec::new(),
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!("1.2.3-alpha1+build5".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!("  1.2.3-alpha1+build5  ".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::AlphaNumeric(String::from("alpha1"))],
                       build: vec![Identifier::AlphaNumeric(String::from("build5"))],
                   }));
        assert_eq!("1.2.3-1.alpha1.9+build5.7.3aedf  ".parse(),
                   Ok(Version {
                       major: 1,
                       minor: 2,
                       patch: 3,
                       pre: vec![Identifier::Numeric(1),
                      Identifier::AlphaNumeric(String::from("alpha1")),
                      Identifier::Numeric(9),
            ],
                       build: vec![Identifier::AlphaNumeric(String::from("build5")),
                        Identifier::Numeric(7),
                        Identifier::AlphaNumeric(String::from("3aedf")),
            ],
                   }));
        assert_eq!("0.4.0-beta.1+0851523".parse(),
                   Ok(Version {
                       major: 0,
                       minor: 4,
                       patch: 0,
                       pre: vec![Identifier::AlphaNumeric(String::from("beta")),
                      Identifier::Numeric(1),
            ],
                       build: vec![Identifier::AlphaNumeric(String::from("0851523"))],
                   }));

    }

    #[test]
    fn test_from_str_errors() {
        fn parse_error(e: &str) -> result::Result<Version, SemVerError> {
            return Err(SemVerError::ParseError(e.to_string()));
        }

        assert_eq!("".parse(), parse_error("Parse error"));
        assert_eq!("  ".parse(), parse_error("Parse error"));
        assert_eq!("1".parse(), parse_error("Parse error"));
        assert_eq!("1.2".parse(), parse_error("Parse error"));
        assert_eq!("1.2.3-".parse(),
                   parse_error("Failed with unparsed input: '-'"));
        assert_eq!("a.b.c".parse(), parse_error("Parse error"));
        assert_eq!("1.2.3 abc".parse(),
                   parse_error("Failed with unparsed input: ' abc'"));
    }
}
