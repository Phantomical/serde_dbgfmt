# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.1.1
### Fixed
- Fixed an issue where `Option<T>` was not being parsed correctly (by @daladim).
- Fixed an issue where character escapes in string and character tokens were not
  being parsed correctly.
- Fixed an issue where `inf` was not being correctly parsed as a float value.
- Fixed an issue where signed integers would be parsed incorrectly if they had
  more digits than should be present in the decimal number value.
- Fixed an issue where signed integers would be parsed incorrectly if they had
  trailing `0` digits.

## 0.1.0
This is the initial release of `serde_dbgfmt`.
