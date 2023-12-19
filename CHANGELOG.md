# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased
- None.



## 0.5.0(2023-12-19)

- **feat:** support `axum 0.7`

## 0.4.0(2023-09-26)
- feat: update swagger `png`

- feat: replace `EmptyBodyRespons`e into `()`, update `readme.md` for usage
- feat: update swagger files and remove patch method
- feat: add check for operate `http`
- `refactor`: move `http` actions into `HTTPOperateCheck`
- feat: add more log for `http` operate, add more test check for `http` actor
- feat: add more fields for test
- feat: add snafu for better err display

## 0.3.0(2023-04-03)

- **added:** add initial support for swagger based on [aide](https://github.com/tamasfe/aide) project, swagger support still needs a lot of improvement


## 0.2.0(2023-04-01)

- **added:** Add views with create, update, query methods
- **added:** Add default 404 handler, graceful shutdown, `tls` support in `utils`
- **added:** Add crate `AppError impl` anyhow::Error
- **added:** Add `prometheus` metrics server
