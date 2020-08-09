.PHONY: help test

help: ## Display help information
	@printf 'usage: make [target] ...\n\ntargets:\n'
	@egrep '^(.+)\:\ .*##\ (.+)' ${MAKEFILE_LIST} | sed 's/:.*##/#/' | column -t -c 2 -s '#'

build: ## Build debug version
	@cargo build

release: ## Build release version and strip the binary
	@cargo build --release
	@strip target/release/vimwikid

install: ## Install release version locally (does not strip)
	@cargo install --path .

uninstall: ## Uninstalls release version built locally
	@cargo uninstall

test: ## Run all tests
	@cargo test
