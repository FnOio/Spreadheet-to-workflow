#!/usr/bin/env bash

# Exit if there are errors
set -e

echo 'Clearing previous install...'
cargo clean
rm -rf bin
mkdir bin

echo 'Installing YARRRML Parser...'
cd bin
npm install @rmlio/yarrrml-parser

echo 'Downloading RMLMapper...'
wget --output-document=rmlmapper.jar https://github.com/RMLio/rmlmapper-java/releases/download/v7.3.3/rmlmapper-7.3.3-r374-all.jar
cd ../

echo 'Building Spreadsheet-to-flow...'
cargo build --release
