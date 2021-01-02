.PHONY: build test test-task1
test: test-task1 test-task2 test-task4

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
	turnt --config brili.toml test/task4-ssa/*.bril
