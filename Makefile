ENDPOINT ?= mainnet.sol.streamingfast.io:443
SUBSTREAMS_API_TOKEN ?= 

.PHONY: token
token:
	export SUBSTREAMS_API_TOKEN=$(SUBSTREAMS_API_TOKEN)

.PHONY: build
build:
	LDFLAGS="-Wl,-no_compact_unwind" cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_block -s 355327313 -t +1 > trades.jsonl

.PHONY: protogen
protogen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: package
package:
	substreams pack ./substreams.yaml
