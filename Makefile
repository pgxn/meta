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
