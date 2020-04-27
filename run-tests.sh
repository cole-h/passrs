#!/bin/sh
env PASSWORD_STORE_DIR=./tests/test_repo cargo test --test util
