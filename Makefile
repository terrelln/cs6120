.PHONY: build test test-task1
test: test-task1

build:
	cargo build

test-task1: build
	cargo build
	turnt test/task1/*.bril

test-task2: build
	cargo build
	turnt test/task2-dce/*.bril
	turnt test/task2-lvn/*.bril
	turnt --config brili.toml test/task2-lvn/*.bril