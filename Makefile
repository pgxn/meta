.PHONY: test # Validate the JSON schema.
test:
	@cargo test

.git/hooks/pre-commit:
	@printf "#!/bin/sh\nmake lint\n" > $@
	@chmod +x $@

.PHONY: lint # Lint the project
lint: .pre-commit-config.yaml
	@pre-commit run --show-diff-on-failure --color=always --all-files

brew-install-gfm:
	@brew install cmark-gfm

spec.html: spec.md
	cmark-gfm --to html --smart -e table -e footnotes -e strikethrough $< > $@
