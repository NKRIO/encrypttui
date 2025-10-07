#!/bin/bash

echo "Make sure that ../src is Rust src directory."

if [ ! -d "../src/" ]; then
  echo "The directory, ../src/ does not exists"
  exit 1
fi

if [ ! -f "config.jsonc" ]; then
  echo "The file, config.jsonc dose not exists"
  exit 1
fi

node index.js config.jsonc -o ../src/cfg.rs
