#!/bin/bash

set -e

cargo check

cargo build --release 
mkdir -p plugin
cp ./src/plugin.vim plugin/nvim-markers.vim
cp ./target/release/nvim-markers plugin
git commit -am 'install'

