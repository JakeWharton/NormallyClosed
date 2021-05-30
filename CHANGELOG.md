# Change log

## [Unreleased]


## [0.2.0] - 2021-05-29

### Added

 * Output host and port on startup to indicate the app is successfully running.
 * ARMv6 build for Raspbery Pi Zero
 * Support aggregating buttons from secondary hosts using the top-level `secondary_hosts` array in the TOML. See the `README.md` for more information.

### Changed

 * Docker container is now a multi-architecture build supporting ARMv7 and ARMv6.
 * Docker container init now uses Tini instead of S6 (which saves nearly 50% on container size).


## [0.1.0] - 2021-04-28

Initial release


[Unreleased]: https://github.com/JakeWharton/NormallyClosed/compare/0.2.0...HEAD
[0.2.0]: https://github.com/JakeWharton/NormallyClosed/releases/tag/0.2.0
[0.1.0]: https://github.com/JakeWharton/NormallyClosed/releases/tag/0.1.0
