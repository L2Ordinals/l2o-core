PROFILE    			:= lite
LOG_LEVEL  			:= info,r1cs=off
TRACE_ENABLED   := 1

.PHONY: check
check:
	@cargo check --all-targets

.PHONY: fix
fix:
	@cargo fix --all-targets --allow-dirty --allow-staged

.PHONY: format
format:
	@cargo fmt

.PHONY: test
test:
	@cargo test -- --nocapture

.PHONY: relaunch
relaunch: shutdown launch
	sleep 5
	@bitcoin-cli -regtest -rpcuser=devnet -rpcpassword=devnet createwallet "default" > /dev/null 2>&1 || true
	@ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet wallet create > /dev/null 2>&1 || true

.PHONY: launch
launch:
	@docker-compose \
		-f docker-compose.yml \
		--profile ${PROFILE} \
		up \
		--build \
		-d \
		--remove-orphans

.PHONY: shutdown
shutdown:
	@docker-compose \
		-f docker-compose.yml \
		down \
		--remove-orphans > /dev/null 2>&1 || true
	@sudo rm -fr chaindata || true
	@sudo rm -fr ordhook-data
	@sudo rm -fr db
	@rm -fr ~/.local/share/ord
	@rm -fr ~/.bitcoin

.PHONY: bitcoin-advance-block
bitcoin-advance-block:
	@bitcoin-cli -regtest -rpcwallet=default -rpcuser=devnet -rpcpassword=devnet -generate 2

.PHONY: bitcoin-latest-block
bitcoin-latest-block:
	@curl -s http://127.0.0.1:50000/blocks | jq '.[0]'

.PHONY: bitcoin-explorer
bitcoin-explorer:
	@(open http://localhost:8094 || xdg-open http://localhost:8094) > /dev/null 2>&1 || true

.PHONY: ord-explorer
ord-explorer:
	@(open http://localhost:1333 || xdg-open http://localhost:1333) > /dev/null 2>&1 || true
	@ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet --enable-save-ord-receipts --enable-index-bitmap --enable-index-brc20 server --http-port 1333

.PHONY: ord-init
ord-init:
	@bitcoin-cli -regtest -rpcuser=devnet -rpcpassword=devnet -rpcwallet=default settxfee 0.00001000 > /dev/null 2>&1 || true
	@bitcoin-cli -regtest -rpcuser=devnet -rpcpassword=devnet -rpcwallet=default generatetoaddress 101 $(shell bitcoin-cli -regtest -rpcuser=devnet -rpcpassword=devnet -rpcwallet=default getnewaddress) > /dev/null 2>&1 || true
	@bitcoin-cli -regtest -rpcwallet=default -rpcuser=devnet -rpcpassword=devnet sendtoaddress $(shell ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet wallet receive | jq '.address' | tr -d '"') 10 > /dev/null 2>&1 || true
	@bitcoin-cli -regtest -rpcwallet=default -rpcuser=devnet -rpcpassword=devnet -generate 2 > /dev/null 2>&1 || true

.PHONY: ord-inscriptions
ord-inscriptions:
	@ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet wallet inscriptions | sed 's/localhost/localhost:1333/g' | grep explorer | awk '{print $$2}'  | tr -d '",' | awk '{print $$1}'

.PHONY: ord-lastest-inscription
ord-lastest-inscription:
	@xdg-open $(shell ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet wallet inscriptions | sed 's/localhost/localhost:1333/g' | grep explorer | awk '{print $$2}'  | tr -d '",' | awk '{print $$1}' | head -n1) > /dev/null 2>&1 || true
	@ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet server --http-port 1333

.PHONY: ord-inscribe
ord-inscribe:
	@ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet wallet inscribe --file ${FILE} --fee-rate 1
	@bitcoin-cli -regtest -rpcwallet=default -rpcuser=devnet -rpcpassword=devnet -generate 2 > /dev/null 2>&1 || true

.PHONY: ord-reindex
ord-reindex:
	@ord -r --bitcoin-rpc-user=devnet --bitcoin-rpc-pass=devnet index run

.PHONY: run
run: run-indexer run-ordhook run-l2o-sequencer

.PHONY: run-l2o-sequencer
run-l2o-sequencer:
	@RUST_LOG=${LOG_LEVEL} RUST_BACKTRACE=${TRACE_ENABLED} cargo run --package l2o-cli -- sequencer

.PHONY: run-l2o-initializer
run-l2o-initializer:
	@RUST_LOG=${LOG_LEVEL} RUST_BACKTRACE=${TRACE_ENABLED} cargo run --package l2o-cli -- initializer

.PHONY: run-indexer
run-indexer:
	@RUST_LOG=${LOG_LEVEL} RUST_BACKTRACE=${TRACE_ENABLED} cargo run --package l2o-cli -- indexer-ord-hook --addr=0.0.0.0:3000

.PHONY: run-ordhook
run-ordhook:
	@ordhook service start --post-to=http://localhost:3000/api/events --config-path=./Ordhook.toml

.PHONY: l2o-last-block
l2o-last-block:
	curl http://localhost:3000 \
  	-X POST \
  	-H "Content-Type: application/json" \
		--data '{"method":"l2o_getLastBlockInscription","params":1,"id":1,"jsonrpc":"2.0"}' | jq

.PHONY: l2o-deploy-info
l2o-deploy-info:
	curl http://localhost:3000 \
		-X POST \
		-H "Content-Type: application/json" \
		--data '{"method":"l2o_getDeployInscription","params":1,"id":1,"jsonrpc":"2.0"}' | jq

.PHONY: l2o-superchain-root
l2o-superchain-root:
	curl http://localhost:3000 \
		-X POST \
		-H "Content-Type: application/json" \
		--data '{"method":"l2o_getSuperchainStateRootAtBlock","params":[1,"Sha256"],"id":1,"jsonrpc":"2.0"}' | jq

.PHONY: l2o-state-root
l2o-state-root:
	curl http://localhost:3000 \
		-X POST \
		-H "Content-Type: application/json" \
		--data '{"method":"l2o_getStateRootAtBlock","params":[1,0,"Sha256"],"id":1,"jsonrpc":"2.0"}' | jq

.PHONY: l2o-merkle-proof-state-root
l2o-merkle-proof-state-root:
	curl http://localhost:3000 \
		-X POST \
		-H "Content-Type: application/json" \
		--data '{"method":"l2o_getMerkleProofStateRootAtBlock","params":[1,0,"Sha256"],"id":1,"jsonrpc":"2.0"}' | jq

.PHONY: image
image:
	docker build \
		--build-arg PROFILE=debug \
		-c 512 \
		-t l2ordinals/l2o:latest \
		-f Dockerfile .
