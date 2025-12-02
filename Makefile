.PHONY: test test-all test-property test-snapshot test-parameterized test-coverage fuzz fuzz-all update-snapshots help

# Run all tests
test:
	cargo test

# Run all test suites
test-all: test-property test-snapshot test-parameterized
	cargo test

# Run property-based tests
test-property:
	cargo test --test property_tests

# Run snapshot tests
test-snapshot:
	cargo test --test snapshot_tests

# Update snapshots (interactive)
update-snapshots:
	cargo insta review

# Run parameterized tests
test-parameterized:
	cargo test --test parameterized || true

# Generate test coverage report
test-coverage:
	cargo tarpaulin --out Html --output-dir coverage/
	@echo "Coverage report generated in coverage/tarpaulin-report.html"

# Run fuzzing (requires cargo-fuzz)
fuzz:
	cd fuzz && cargo fuzz run fuzz_content_hash

# Run all fuzz targets
fuzz-all:
	cd fuzz && cargo fuzz run fuzz_content_hash -- -max_total_time=300
	cd fuzz && cargo fuzz run fuzz_version_parsing -- -max_total_time=300
	cd fuzz && cargo fuzz run fuzz_test_count_extraction -- -max_total_time=300

# Install testing tools
install-test-tools:
	cargo install cargo-fuzz cargo-tarpaulin cargo-insta

# Help
help:
	@echo "Testing Commands:"
	@echo "  make test              - Run all tests"
	@echo "  make test-all          - Run all test suites"
	@echo "  make test-property     - Run property-based tests"
	@echo "  make test-snapshot     - Run snapshot tests"
	@echo "  make update-snapshots  - Update snapshots (interactive)"
	@echo "  make test-coverage     - Generate coverage report"
	@echo "  make fuzz              - Run fuzzing"
	@echo "  make fuzz-all          - Run all fuzz targets"
	@echo "  make install-test-tools - Install testing tools"

