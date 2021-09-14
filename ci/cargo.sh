#!/bin/bash

# Find line where [[lib]] first appears
tmp=$(grep -n "\[lib\]" Cargo.toml)
line=$((${tmp%:*}-1))
# Remove lib and binaries at the end of the file
mv Cargo.toml Cargo.tmp
head Cargo.tmp -n $line > Cargo.toml
rm Cargo.tmp
