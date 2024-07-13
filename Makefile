.PHONY: build test test-task1
test: test-task1 test-task2 test-task4

setup:
	pip3 install turnt
	pip3 install flit
	cd bril/bril-txt && flit install --symlink --env

build:
	cargo build

test-task1: build
	turnt test/task1/*.bril

test-task2: build
	turnt test/task2-dce/*.bril
	turnt test/task2-lvn/*.bril
	turnt --config brili.toml test/task2-lvn/*.bril

test-task4: build
	turnt --config is_ssa.toml test/task4-ssa/*.bril
	turnt --config to_ssa.toml test/task4-ssa/*.bril
	turnt --config to_ssa_brili.toml test/task4-ssa/*.bril
	turnt --config from_ssa_brili.toml test/task4-ssa/*.bril
	turnt --config from_ssa.toml test/task4-ssa/*.bril
	turnt --config turnt_to_ssa.toml benchmarks/*.bril
	turnt --config turnt_from_ssa.toml benchmarks/*.bril
