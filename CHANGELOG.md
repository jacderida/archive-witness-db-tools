# CHANGELOG

All notable changes to this project will be documented in this file.

## [1.6.0] - 2024-06-22

* db 1.1.0
* tools 1.6.0

### 🚀 Features

- Provide `nist import document-numbers` cmd
- [**breaking**] `nist tapes ls` updated to display the document number

## [1.5.4] - 2024-06-13

* tools 1.5.4

### 🚀 Features

- Provide `--with-notes` arg for `nist tapes ls` cmd

## [1.5.3] - 2024-06-12

* tools 1.5.3

### 🐛 Bug Fixes

- Correct wrapping on additional notes field on `nist tapes ls` cmd

## [1.5.2] - 2024-06-12

* tools 1.5.2

### 🚀 Features

- Improve styling for `nist tapes ls` report

### 🐛 Bug Fixes

- Unallocated videos count does not include missing in `nist tapes ls`

## [1.5.1] - 2024-06-12

* db 1.0.6
* tools 1.5.1

### 🚀 Features

- Refine style for missing records in `nist tapes ls`
- Print summary on `nist tapes ls`
- Provide `--exclude-missing` for `nist tapes ls` cmd
- Align summaries for `nist tapes ls` and `releases reports nist-allocated-videos`

### 🐛 Bug Fixes

- `nist tapes ls` uses alt regex for allocated dirs

## [1.5.0] - 2024-06-11

* db 1.0.5
* tools 1.5.0

### 🚀 Features

- Mark nist videos as missing with notes
- [**breaking**] Change `--filter-found` to `--not-allocated`

## [1.4.1] - 2024-06-06

* tools 1.4.1

### 🐛 Bug Fixes

- `nist tapes ls --find` to use lowercase for tape records

## [1.4.0] - 2024-06-05

* db 1.0.4
* tools 1.4.0

### 🚀 Features

- [**breaking**] Move `releases files-ls` cmd to `releases files ls`
- [**breaking**] Move `releases ls-extensions` cmd to `releases files ls-extensions`
- [**breaking**] Make `tapes ls --find` case insensitive
- [**breaking**] Provide `--sum` arg on `releases ls-extensions` cmd
- `release ls-extensions` lists per-release for range
- Provide `releases reports nist-videos-allocated` cmd

### 🚜 Refactor

- Use `cmd` module to organise command processing

## [1.3.3] - 2024-05-26

* tools 1.3.3

### 🚀 Features

- Provide `--filter-found` arg for `nist tapes ls` cmd

## [1.3.2] - 2024-05-25

* tools 1.3.2

### 🚀 Features

- `nist tapes edit` cmd prompts for confirmation

## [1.3.1] - 2024-05-25

* tools 1.3.1

### 🚀 Features

- `nist tapes ls` cmd includes broadcast date

## [1.3.0] - 2024-05-25

* tools 1.3.0
* db 1.0.3

### 🚀 Features

- [**breaking**] `nist tapes ls` command groups tapes by video
- Provide `--find` arg for `nist tapes ls` cmd
- `nist tapes edit` can select tape with fuzzy find

## [1.2.0] - 2024-05-23

* tools 1.2.0
* db 1.0.2

### 🚀 Features

- Provide `nist videos ls` command
- Provide duration on `nist tapes ls` command
- Use yellow to indicate duplicates in `nist ls tapes` cmd
- Indicate batch/clips/timecode on `nist tapes ls` cmd
- Provide `--show-videos` on `nist tapes ls` cmd
- Support multiple release refs in `nist tapes ls` cmd
- Provide `nist tapes print` cmd
- [**breaking**] Rename `--filter-files` to `--filter-found`

### 🐛 Bug Fixes

- Change incorrect description on `nist tapes` subcommand

## [1.1.0] - 2024-05-21

* tools 1.1.0
* db 1.0.1

### 🚀 Features

- Provide `nist tapes ls` command
- Provide `nist tapes edit` command
- Provide `--filter-files` flag on `nist tapes ls` cmd
- Output release ref on `nist tapes ls` command

### 🐛 Bug Fixes

- Process nist files correctly when converting master

### 📚 Documentation

- Describe deployment on server
- Describe release process
- Update release process steps

### ⚙️ Miscellaneous Tasks

- Move `access-db` cmd to `nist import` cmd

## [1.0.1] - 2024-05-20

### 🐛 Bug Fixes

- Update incorrect torrent url

## [1.0.0] - 2024-05-20

### 🚀 Features

- Provide `import-releases` command
- Provide `images import` command
- Provide `releases ls-extensions` command
- Cumulus field commands
- Provide `images convert` command
- Provide `videos convert` command
- Import access database commands
- Provide `videos build-master` command
- List extensions for all releases
- List extensions for range of releases
- Provide `videos export` command
- Provide `videos export-master` command
- Script for generating video description
- Script to append time of day to timestamps
- Provide `add-master` command
- Provide `master-videos edit` command
- Provide `videos add` command
- Provide `--path` arg for `masters add` cmd
- Provide `releases ls-files` command
- Provide `releases find` command
- Extend `videos add` command to add files
- Provide `videos print` command
- Provide `videos edit` command
- Provide `news networks add` command
- Provide `news affiliates add` command
- Provide `news networks edit` command
- Provide `news networks print` command
- Provide `news networks ls` command
- Provide `news affiliates edit` command
- Provide `news affiliates print` command
- Provide `news affiliates ls` command
- Provide `news broadcasts add` command
- Provide `news broadcasts edit` command
- Provide `news broadcasts print` command
- Provide `news broadcasts ls` command
- Add video from youtube database
- Provide `masters ls` command
- Provide `videos ls` command

### 🐛 Bug Fixes

- Master video list tape reference

### 🚜 Refactor

- Use sqlx for `import-releases` command
- Store torrent content in database
- Use forms rather than ad-hoc parsing

### ⚙️ Miscellaneous Tasks

- Read assets from nist cumulus export
- Partially implement image import
- Move `download-torrents` command
- Include networks in the database schema
- Move scripts to directory
- Remove error type
- Store release files in the database
- Rename `master-videos` cmd to `masters`
- Convert from single crate to workspace
- New crate for youtube database
- Restructure videos representation in db
- Fix clippy warnings
- Rename crates
- Remove image content from the database
- Fix clippy warnings
- Use specific database var
- Provide changelog config

<!-- generated by git-cliff -->
