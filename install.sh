#!/usr/bin/bash

mkdir -p ~/.local/bin
ln -s "$(pwd)/target/release/watching_record" ~/.local/bin/watching_record
