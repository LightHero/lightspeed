import "./just/build.just"
import "./just/code_check.just"
import "./just/test.just"

export RUST_BACKTRACE := "full"
export FEATURES := "test1;test2;test1,test2"

# Lists all the available commands
default:
  @just --list
