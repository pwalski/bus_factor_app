.DEFAULT_GOAL:= build

directory = clients/github/openapi

check-submodules:
	@if git submodule status | egrep -q '^[-]|^[+]'; then \
		git submodule update --init; \
	fi

gen: check-submodules
	rm -rf $(directory)
	cd clients/github && \
	npm exec @openapitools/openapi-generator-cli -- generate

$(directory):
	$(MAKE) gen

clean: | $(directory) 
	cargo clean

build: | $(directory)
	cargo build
