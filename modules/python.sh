#!/bin/bash

set -xe

export PYTHON_VERSIONS="3.8.1 2.7.17"

for VERSION in $PYTHON_VERSIONS; do
    curl -fSsL "https://www.python.org/ftp/python/$VERSION/Python-$VERSION.tar.xz" -o /tmp/python-$VERSION.tar.xz
    mkdir /tmp/python-$VERSION
    tar -xf /tmp/python-$VERSION.tar.xz -C /tmp/python-$VERSION --strip-components=1
    rm /tmp/python-$VERSION.tar.xz
    cd /tmp/python-$VERSION
    ./configure --prefix=/usr/local/python-$VERSION
    make -j$(nproc)
    make -j$(nproc) install
    rm -rf /tmp/*
done
