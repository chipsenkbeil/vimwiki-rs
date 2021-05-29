.PHONY: help build clean test

help: ## Display help information
	@printf 'usage: make [target] ...\n\ntargets:\n'
	@egrep '^(.+)\:\ .*##\ (.+)' ${MAKEFILE_LIST} | sed 's/:.*##/#/' | column -t -c 2 -s '#'

build: ## Build debug version
	@cargo build

clean: ## Cleans build resources
	@cargo clean

test: ## Run all tests
	@cargo test
