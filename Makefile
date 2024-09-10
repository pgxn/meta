.PHONY: test # Run the full test suite.
test:
	@cargo test

.git/hooks/pre-commit:
	@printf "#!/bin/sh\nmake lint\n" > $@
	@chmod +x $@

.PHONY: lint # Lint the project
lint: .pre-commit-config.yaml
	@pre-commit run --show-diff-on-failure --color=always --all-files

.PHONY: cover # Run cover tests and generate & open a report.
cover:
	@./.ci/test-cover

.PHONY: docs # Generate and open cargo docs.
docs: target/doc/pgxn_meta/index.html
	open $<

target/doc/pgxn_meta/index.html: $(shell find . -name \*.rs)
	cargo doc

VERSION = $(shell perl -nE '/^version\s*=\s*"([^"]+)/ && do { say $$1; exit }' Cargo.toml)
.PHONY: release-notes # Show release notes for current version (must have `mknotes` in PATH).
release-notes: CHANGELOG.md
	mknotes -v v$(VERSION) -f $< -r https://github.com/$(or $(GITHUB_REPOSITORY),pgxn/meta)
