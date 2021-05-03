#!/bin/bash

set -xe

export NODE_VERSIONS="14.16.1"

for VERSION in $NODE_VERSIONS; do
    curl -fSsL "https://nodejs.org/dist/v$VERSION/node-v$VERSION.tar.gz" -o /tmp/node-$VERSION.tar.gz
    mkdir /tmp/node-$VERSION
    tar -xf /tmp/node-$VERSION.tar.gz -C /tmp/node-$VERSION --strip-components=1
    rm /tmp/node-$VERSION.tar.gz
    cd /tmp/node-$VERSION
    ./configure --prefix=/usr/local/node-$VERSION
    make -j$(nproc)
    make -j$(nproc) install
    rm -rf /tmp/*;
done
