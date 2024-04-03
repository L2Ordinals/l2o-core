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

.PHONY: run-indexer
run-indexer:
	@cargo run --package l2o-cli -- indexer

.PHONY: run-indexer-poc
run-indexer-poc:
	@cargo run --package l2o-cli -- indexer-poc

.PHONY: run-ordhook
run-ordhook:
	@ordhook service start --post-to=http://localhost:1337/api/events --config-path=./Ordhook.toml

.PHONY: image
image:
	docker build \
		--build-arg PROFILE=debug \
		-c 512 \
		-t l2orinals/l2o:latest \
		-f Dockerfile .
